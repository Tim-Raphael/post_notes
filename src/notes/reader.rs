//! I/O boundary for the read phase.
//!
//! [`ReadNotes`] is the only trait the pipeline in [`super`] speaks to. Tests provide an in-memory
//! implementation; production uses [`FileSystemReader`].

use std::ffi;
use std::io;
use std::sync;

use tracing_scribe::console;

use crate::Settings;

/// The raw bytes of a single note, before parsing.
#[derive(Clone, Debug)]
pub struct RawNote {
    pub file_name: ffi::OsString,
    pub content: String,
}

impl From<(ffi::OsString, String)> for RawNote {
    fn from(value: (ffi::OsString, String)) -> Self {
        Self {
            file_name: value.0,
            content: value.1,
        }
    }
}

/// Streams raw notes from some backing store. Yielding a `Result` per item (rather than a single
/// `Result<Stream, _>`) lets the pipeline surface partial failures.
///
/// The `async fn` return type is intentionally auto-trait-bound-free: tests use single-threaded
/// runtimes and don't need `Send`.
#[allow(async_fn_in_trait)]
pub trait ReadNotes {
    async fn read_notes(&self) -> impl tokio_stream::Stream<Item = io::Result<RawNote>>;
}

#[derive(Clone, Debug)]
pub struct FileSystemReader {
    settings: sync::Arc<Settings>,
}

impl FileSystemReader {
    pub fn new(settings: sync::Arc<Settings>) -> Self {
        Self { settings }
    }
}

impl ReadNotes for FileSystemReader {
    async fn read_notes(&self) -> impl tokio_stream::Stream<Item = io::Result<RawNote>> {
        // Channel width is a soft cap on how many `read_to_string` completions can be queued before
        // we backpressure the reader.
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let settings = self.settings.clone();

        tokio::task::spawn(async move {
            // Reader-level skip counters: subdirectories and non-markdown files are filesystem
            // noise (`.obsidian/`, images, etc.), not notes, so we don't log them individually.
            let mut markdown_count: usize = 0;
            let mut skipped_subdirs: usize = 0;
            let mut skipped_non_markdown: usize = 0;

            let result = async {
                let input = &settings.input;
                tracing::debug!(dir = ?input, "reading notes directory");
                let mut read_dir = tokio::fs::read_dir(input).await?;

                while let Some(entry) = read_dir.next_entry().await? {
                    let path = entry.path();

                    if path.is_dir() {
                        tracing::debug!(?path, "skipping subdirectory");
                        skipped_subdirs += 1;
                        continue;
                    }

                    let is_markdown = path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext == "md");

                    if !is_markdown {
                        tracing::trace!(?path, "skipping non-markdown file");
                        skipped_non_markdown += 1;
                        continue;
                    }

                    let file_name = entry.file_name();
                    let tx = tx.clone();

                    markdown_count += 1;

                    tokio::task::spawn(async move {
                        let raw_note = tokio::fs::read_to_string(path)
                            .await
                            .map(|content| RawNote::from((file_name, content)));
                        let _ = tx.send(raw_note).await;
                    });
                }

                Ok(())
            }
            .await;

            if result.is_ok() {
                console!(
                    info,
                    "scanned input directory: markdown files = {markdown_count}, skipped non-markdown files = {skipped_non_markdown}, skipped subdirectories = {skipped_subdirs}"
                );
            }

            if let Err(err) = result {
                let _ = tx.send(Err(err)).await;
            }
        });

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}
