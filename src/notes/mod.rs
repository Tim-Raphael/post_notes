//! Read phase: turn a directory of markdown files into structured [`Note`]s.
//!
//! The public entry point is [`Notes`], a typestate-guarded pipeline that
//! wraps an implementation of [`ReadNotes`]. Users construct it in the
//! [`Unsynced`] state, call [`Notes::sync`], and get back a [`Synced`]
//! instance that exposes the parsed notes via [`Notes::notes`].
//!
//! The `ReadNotes` trait is the I/O boundary — swap in a mock for tests to
//! exercise the pipeline without touching disk.

mod reader;
mod types;

pub use reader::{FileSystemReader, RawNote, ReadNotes};
use tracing_scribe::console;
pub use types::{Body, FileName, Frontmatter, Note, Tag};

use anyhow::{Context as _, anyhow};
use tokio_stream::StreamExt as _;

use std::{io, marker, sync};

use crate::Settings;

#[derive(Clone, Debug)]
pub struct Unsynced;

#[derive(Clone, Debug)]
pub struct Synced;

#[derive(Clone, Debug)]
pub struct Notes<NotesReader, State = Unsynced> {
    notes: Vec<Note>,
    reader: NotesReader,
    // TODO: add watcher
    _state: marker::PhantomData<State>,
}

/// Preserves the original [`Notes`] so callers can retry `sync` after handling the underlying I/O
/// failure.
#[derive(Debug)]
pub struct SyncError<R>
where
    R: ReadNotes,
{
    pub source: io::Error,
    pub provider: Notes<R>,
}

impl<R> std::fmt::Display for SyncError<R>
where
    R: ReadNotes,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to read notes: {}", self.source)
    }
}

impl<R> std::error::Error for SyncError<R>
where
    R: ReadNotes + std::fmt::Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl<R> Notes<R, Synced>
where
    R: ReadNotes,
{
    pub fn notes(&self) -> &[Note] {
        self.notes.as_ref()
    }

    /// Consume the pipeline and return the parsed notes.
    pub fn into_notes(self) -> Vec<Note> {
        self.notes
    }
}

impl<R> Notes<R, Unsynced>
where
    R: ReadNotes,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            notes: vec![],
            _state: marker::PhantomData,
        }
    }

    #[tracing::instrument(name = "notes::sync", skip_all)]
    pub async fn sync(self) -> Result<Notes<R, Synced>, SyncError<R>> {
        let mut notes = Vec::new();
        let mut sync_err = None;
        // Per-phase skip tallies. `parse_raw_note` returns `Ok(None)` only for private notes today,
        // so `Ok(None)` is a safe proxy for "skipped because non-public".
        let mut private: usize = 0;
        let mut parse_errors: usize = 0;

        {
            let raw_notes = self.reader.read_notes().await;
            tokio::pin!(raw_notes);

            while let Some(raw_note) = raw_notes.next().await {
                // If an I/O error occurs while reading a file we abort: we don't know whether the
                // missing note would've been renderable, and silently dropping it would corrupt the
                // navigation tree.
                let raw_note = match raw_note {
                    Ok(raw_note) => raw_note,
                    Err(err) => {
                        tracing::error!(error = %err, "could not read note");
                        sync_err = Some(err);
                        break;
                    }
                };

                // Parse failures are non-fatal: a malformed note can't be rendered anyway, so skip
                // it and keep going. The file name is embedded in the error's context chain (see
                // `parse_raw_note`), so no separate field is needed.
                match Self::parse_raw_note(raw_note) {
                    Ok(Some(note)) => notes.push(note),
                    Ok(None) => private += 1,
                    Err(err) => {
                        parse_errors += 1;
                        tracing::warn!(error = %err, "could not parse note");
                    }
                }
            }
        }

        if let Some(source) = sync_err {
            return Err(SyncError {
                source,
                provider: self,
            });
        }

        let kept = notes.len();

        console!(
            info,
            "processed notes: parsed = {kept}, private = {private}, parse errors = {parse_errors}",
        );

        // Sort for determinism: two runs with the same input must produce the same output (see
        // `tests/consistency.rs`). The reader emits notes in filesystem order, which is not stable
        // across platforms.
        notes.sort_by(|a, b| a.file_name.inner.cmp(&b.file_name.inner));

        Ok(Notes {
            notes,
            reader: self.reader,
            _state: marker::PhantomData::<Synced>,
        })
    }

    fn parse_raw_note(RawNote { file_name, content }: RawNote) -> anyhow::Result<Option<Note>> {
        let file_name = FileName {
            inner: file_name.to_string_lossy().to_string(),
        };

        let mut options = markdown::ParseOptions::gfm();
        options.constructs.frontmatter = true;
        options.constructs.math_flow = true;
        options.constructs.math_text = true;

        let mdast = markdown::to_mdast(&content, &options)
            .map_err(|err| anyhow!("failed to parse {file_name}: {}", err.reason))?;

        let children = mdast
            .children()
            .with_context(|| format!("{file_name} is empty"))?;

        let frontmatter = children
            .iter()
            .find_map(|node| {
                if let markdown::mdast::Node::Yaml(yaml) = node {
                    Some(&yaml.value)
                } else {
                    None
                }
            })
            .with_context(|| format!("could not find frontmatter in {file_name}"))?;

        // `public` gates rendering entirely, so it lives on a wrapper rather than leaking into the
        // downstream `Frontmatter`.
        #[derive(serde::Deserialize)]
        struct RawFrontmatter {
            #[serde(flatten)]
            frontmatter: Frontmatter,
            #[serde(default)]
            public: bool,
        }

        let raw_frontmatter = serde_yaml::from_str::<RawFrontmatter>(frontmatter)
            .with_context(|| format!("failed to deserialize frontmatter of {file_name}"))?;

        if !raw_frontmatter.public {
            tracing::trace!(%file_name, "skipping private note");
            return Ok(None);
        }

        Ok(Some(Note {
            file_name,
            frontmatter: raw_frontmatter.frontmatter,
            body: Body { inner: mdast },
        }))
    }
}

impl From<sync::Arc<Settings>> for Notes<FileSystemReader, Unsynced> {
    fn from(settings: sync::Arc<Settings>) -> Self {
        let reader = FileSystemReader::new(settings.clone());
        Self::new(reader)
    }
}
