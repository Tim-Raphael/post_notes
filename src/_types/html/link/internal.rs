use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::path::PathBuf;

/// Internal output link to another rendered note.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Internal(String);

impl TryFrom<PathBuf> for Internal {
    type Error = anyhow::Error;

    fn try_from(mut path_buf: PathBuf) -> Result<Self> {
        path_buf.set_extension("html");

        let file_name = path_buf
            .file_name()
            .context("Could not determine file name")?
            .to_string_lossy()
            .to_string();

        Ok(Self(file_name))
    }
}

impl From<String> for Internal {
    fn from(link: String) -> Self {
        let (path_part, rest) = link
            .split_once(['#', '?'])
            .map(|(head, _tail)| (head, &link[head.len()..]))
            .unwrap_or((&link[..], ""));

        let mut full = path_part.trim_start_matches('/').to_string();

        if !full.ends_with(".html") {
            full.push_str(".html");
        }

        full.push_str(rest);

        Self(full)
    }
}

impl Deref for Internal {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
