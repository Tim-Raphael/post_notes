//! I/O boundary for the build phase.
//!
//! [`WriteWebsite`] is the only trait the top-level pipeline speaks to when emitting the site.
//! Tests can capture writes in memory via [`InMemoryWriter`]; production uses [`FileSystemWriter`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::{fs, io};

use crate::Settings;

/// A single blob-to-be-written. `path` is site-relative — the writer is responsible for anchoring
/// it under an output root (or an in-memory map).
#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

/// Sink for a built [`super::Website`]. Implementors decide what "write" means: filesystem,
/// in-memory buffer, tar archive, etc.
pub trait WriteWebsite {
    fn write_artifact(&self, artifact: Artifact) -> io::Result<()>;

    /// Copy a directory tree from `from` (on the local filesystem) into the writer's target space.
    /// Used for static assets and note media, which we don't need to buffer in memory.
    fn copy_tree(&self, from: &Path, to: &Path) -> io::Result<()>;
}

/// Writes to a directory on the local filesystem.
#[derive(Debug, Clone)]
pub struct FileSystemWriter {
    root: PathBuf,
}

impl FileSystemWriter {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn from_settings(settings: &Settings) -> Self {
        Self::new(&settings.output)
    }

    fn abs(&self, rel: &Path) -> PathBuf {
        self.root.join(rel)
    }
}

impl WriteWebsite for FileSystemWriter {
    fn write_artifact(&self, artifact: Artifact) -> io::Result<()> {
        let target = self.abs(&artifact.path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, &artifact.bytes)?;
        tracing::trace!(path = %target.display(), "wrote artifact");
        Ok(())
    }

    fn copy_tree(&self, from: &Path, to: &Path) -> io::Result<()> {
        let target = self.abs(to);
        copy_dir_recursive(from, &target)
    }
}

fn copy_dir_recursive(from: &Path, to: &Path) -> io::Result<()> {
    if !from.exists() {
        // Missing source directories are common in mixed setups (e.g. `assets/media` may not exist
        // for every project). Silently succeed rather than making callers pre-check.
        tracing::debug!(source = %from.display(), "copy_tree source missing, skipping");
        return Ok(());
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src, &dst)?;
        } else {
            fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

/// In-memory writer used by tests and the consistency benchmark.
///
/// Records every artifact and copy request; nothing hits the disk.
#[derive(Debug, Default)]
pub struct InMemoryWriter {
    pub artifacts: Mutex<BTreeMap<PathBuf, Vec<u8>>>,
    pub copies: Mutex<Vec<(PathBuf, PathBuf)>>,
}

impl InMemoryWriter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of all written artifacts, path-sorted.
    pub fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        self.artifacts
            .lock()
            .expect("artifacts mutex poisoned")
            .clone()
    }
}

impl WriteWebsite for InMemoryWriter {
    fn write_artifact(&self, artifact: Artifact) -> io::Result<()> {
        self.artifacts
            .lock()
            .expect("artifacts mutex poisoned")
            .insert(artifact.path, artifact.bytes);
        Ok(())
    }

    fn copy_tree(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.copies
            .lock()
            .expect("copies mutex poisoned")
            .push((from.to_path_buf(), to.to_path_buf()));
        Ok(())
    }
}
