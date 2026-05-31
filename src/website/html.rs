//! HTML/link primitives shared by the build and write phases.

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use std::{ops, path};

/// Fully-rendered HTML. Kept as a newtype so we can't accidentally feed a
/// raw markdown string to a template as if it were safe HTML.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Html(String);

impl Html {
    pub fn new(html: impl Into<String>) -> Self {
        Self(html.into().trim().to_string())
    }
}

impl ops::Deref for Html {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Html {
    fn from(html: String) -> Self {
        Self::new(html)
    }
}

/// A link to another rendered note within the site (e.g. `math-set.html`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Internal(String);

impl Internal {
    /// Build an internal link from a raw target that may or may not
    /// already include an `.html` extension or a fragment/query.
    pub fn from_target(raw: impl AsRef<str>) -> Self {
        let raw = raw.as_ref();
        let (path_part, tail) = raw
            .split_once(['#', '?'])
            .map(|(head, _)| (head, &raw[head.len()..]))
            .unwrap_or((raw, ""));

        let mut full = path_part.trim_start_matches('/').to_string();
        if !full.ends_with(".html") {
            // Strip a trailing `.md` first so `foo.md` becomes `foo.html`,
            // not `foo.md.html`.
            if let Some(stripped) = full.strip_suffix(".md") {
                full = stripped.to_string();
            }
            full.push_str(".html");
        }
        full.push_str(tail);
        Self(full)
    }
}

impl ops::Deref for Internal {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<path::PathBuf> for Internal {
    type Error = anyhow::Error;

    fn try_from(mut path_buf: path::PathBuf) -> anyhow::Result<Self> {
        path_buf.set_extension("html");
        let file_name = path_buf
            .file_name()
            .context("path has no file name")?
            .to_string_lossy()
            .to_string();
        Ok(Self(file_name))
    }
}

impl std::fmt::Display for Internal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A media asset referenced by a note (e.g. `media/diagram.png`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Media(String);

impl Media {
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }
}

impl ops::Deref for Media {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Media {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
