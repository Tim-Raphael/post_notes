//! Build phase: compose [`crate::notes::Note`]s into a renderable [`Website`] and write it through
//! the [`WriteWebsite`] I/O boundary.
//!
//! The write side mirrors [`crate::notes`]: [`Publisher`] is a typestate-guarded pipeline that
//! wraps an implementation of [`WriteWebsite`]. Callers construct it in the [`Unwritten`] state,
//! call [`Publisher::write`], and get back a [`Written`] instance that exposes the writer.

pub mod builder;
pub mod content;
pub mod html;
pub mod navigation;
pub mod page;
pub mod writer;

pub use builder::build;
pub use content::Content;
pub use html::Internal;
pub use navigation::Navigation;
pub use page::Page;
pub use writer::{Artifact, FileSystemWriter, InMemoryWriter, WriteWebsite};

use anyhow::Context as _;

use crate::Settings;

use std::path::{Path, PathBuf};
use std::{fs, marker};

/// One rendered HTML file ready to hit the writer.
///
/// Separate from [`page::Page`] because a `Page` is what the *builder* composes (metadata +
/// un-templated HTML body), while a `RenderedPage` is what the *writer* consumes (fully-templated
/// HTML + the media the page owns).
#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub link: html::Internal,
    pub html: html::Html,
    pub media_links: Vec<html::Media>,
}

/// The output of the build phase: a self-contained site model that can be handed to any
/// [`WriteWebsite`] implementor.
#[derive(Debug, Clone)]
pub struct Website {
    pub pages: Vec<RenderedPage>,
    pub content: Content,
    pub navigation: Navigation,
}

#[derive(Clone, Debug)]
pub struct Unwritten;

#[derive(Clone, Debug)]
pub struct Written;

#[derive(Clone, Debug)]
pub struct Publisher<WebsiteWriter, State = Unwritten> {
    writer: WebsiteWriter,
    _state: marker::PhantomData<State>,
}

/// Preserves the original [`Publisher`] so callers can retry `write` after handling the underlying
/// I/O failure.
#[derive(Debug)]
pub struct WriteError<W>
where
    W: WriteWebsite,
{
    pub source: anyhow::Error,
    pub publisher: Publisher<W>,
}

impl<W> std::fmt::Display for WriteError<W>
where
    W: WriteWebsite,
{
    // Unlike `SyncError`, the source is not interpolated here: it carries an anyhow context chain
    // that `source()` already exposes, so printing it would repeat every line.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to write website")
    }
}

impl<W> std::error::Error for WriteError<W>
where
    W: WriteWebsite + std::fmt::Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl<W> Publisher<W, Written>
where
    W: WriteWebsite,
{
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Consume the pipeline and return the writer.
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<W> Publisher<W, Unwritten>
where
    W: WriteWebsite,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _state: marker::PhantomData,
        }
    }

    // `extra` is what tracing-scribe prints next to the span name, and it only reads string values,
    // so the path has to be rendered here rather than passed as a `Path`.
    #[tracing::instrument(
        name = "website::write",
        skip_all,
        fields(extra = settings.output.display().to_string())
    )]
    pub fn write(
        self,
        website: &Website,
        settings: &Settings,
    ) -> Result<Publisher<W, Written>, WriteError<W>> {
        if let Err(source) = Self::write_website(&self.writer, website, settings) {
            // Logged here, inside the span, so it closes as failed. `{:#}` flattens the anyhow
            // context chain onto the single line the console layer prints, and the `%` sigil is
            // required: the layer only reads `error` from `Debug`-recorded values.
            let error = format!("{source:#}");
            tracing::error!(error = %error, "could not write website");
            return Err(WriteError {
                source,
                publisher: self,
            });
        }

        Ok(Publisher {
            writer: self.writer,
            _state: marker::PhantomData::<Written>,
        })
    }

    /// Order:
    /// 1. static asset directory (CSS/JS)
    /// 2. media referenced by notes (copied from the input directory)
    /// 3. `map.json` (search index)
    /// 4. `{slug}.html` per rendered page
    ///
    /// Steps 3 and 4 are the only ones that produce artifacts; 1 and 2 use `copy_tree` so writers
    /// may implement them however they like.
    fn write_website(writer: &W, website: &Website, settings: &Settings) -> anyhow::Result<()> {
        writer
            .copy_tree(&settings.assets, Path::new(""))
            .context("copying static assets")?;

        Self::write_media(writer, website, settings)?;

        let map_json = serde_json::to_vec_pretty(website.content.as_map())
            .context("serializing content map")?;
        writer.write_artifact(Artifact {
            path: PathBuf::from("map.json"),
            bytes: map_json,
        })?;

        for page in &website.pages {
            writer.write_artifact(Artifact {
                path: PathBuf::from(&*page.link),
                bytes: page.html.as_bytes().to_vec(),
            })?;
        }

        Ok(())
    }

    /// Note media, copied from wherever it lives under the input directory.
    fn write_media(writer: &W, website: &Website, settings: &Settings) -> anyhow::Result<()> {
        for page in &website.pages {
            for media in &page.media_links {
                let rel = PathBuf::from(&**media);
                let src = settings.input.join(&rel);

                if let Some(parent) = rel.parent() {
                    writer
                        .copy_tree(&settings.input.join(parent), parent)
                        .with_context(|| format!("copying media parent {}", parent.display()))?;
                }

                // If the media file is a single file (common case), a full parent-tree copy already
                // grabbed it. But if the media path has no parent, fall through to a single-file
                // copy via an artifact write.
                if rel.parent().is_none() && src.exists() {
                    let bytes = fs::read(&src)
                        .with_context(|| format!("reading media {}", src.display()))?;
                    writer.write_artifact(Artifact { path: rel, bytes })?;
                }
            }
        }

        Ok(())
    }
}
