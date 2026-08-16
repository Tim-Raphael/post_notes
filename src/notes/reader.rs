//! I/O boundary for the read phase.
//!
//! [`ReadNotes`] is the only trait the pipeline in [`super`] speaks to. Tests provide an in-memory
//! implementation; production uses [`FileSystemReader`].

use std::ffi;
use std::io;
use std::path;

use tracing::Instrument as _;
use tracing_scribe::console;

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
    async fn read(
        &self,
        input: &path::Path,
    ) -> impl tokio_stream::Stream<Item = io::Result<RawNote>>;
}

#[derive(Clone, Debug, Default)]
pub struct FileSystemReader;

impl FileSystemReader {
    pub fn new() -> Self {
        Self
    }
}

impl ReadNotes for FileSystemReader {
    // `extra` is what tracing-scribe prints next to the span name, and it only reads string
    // values, so the path has to be rendered here rather than passed as a `Path`.
    #[tracing::instrument(skip_all, fields(extra = input.display().to_string()))]
    async fn read(
        &self,
        input: &path::Path,
    ) -> impl tokio_stream::Stream<Item = io::Result<RawNote>> {
        // Channel width is a soft cap on how many `read_to_string` completions can be queued before
        // we backpressure the reader.
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        // The scan outlives the borrow of `input`, so the spawned task needs its own copy.
        let input = input.to_path_buf();

        // The scan runs after `read` returns, so it needs the span attached explicitly to keep its
        // events nested under the directory being read.
        let scan = async move {
            match scan_directory(&input, &tx).await {
                Ok(ScanStats {
                    markdown,
                    skipped_subdirs,
                    skipped_non_markdown,
                }) => console!(
                    info,
                    "scanned input directory: markdown files = {markdown}, skipped non-markdown files = {skipped_non_markdown}, skipped subdirectories = {skipped_subdirs}"
                ),
                // Logged here, inside the `read` span, so the span closes as failed and the error
                // nests under it. Reporting it only through the channel would let `read` close as
                // a success before the pipeline gets around to logging.
                Err(err) => {
                    tracing::error!(error = %err, "could not scan input directory");
                    let _ = tx.send(Err(err)).await;
                }
            }
        };

        tokio::task::spawn(scan.instrument(tracing::Span::current()));

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}

type NoteSender = tokio::sync::mpsc::Sender<io::Result<RawNote>>;

/// Subdirectories and non-markdown files are filesystem noise (`.obsidian/`, images, etc.), not
/// notes, so we tally them instead of logging each one.
#[derive(Debug, Default)]
struct ScanStats {
    markdown: usize,
    skipped_subdirs: usize,
    skipped_non_markdown: usize,
}

/// Spawns one read task per markdown file in `input` and reports what the directory held.
async fn scan_directory(input: &path::Path, tx: &NoteSender) -> io::Result<ScanStats> {
    let mut stats = ScanStats::default();
    let mut read_dir = tokio::fs::read_dir(input).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();

        if path.is_dir() {
            tracing::trace!(?path, "skipping subdirectory");
            stats.skipped_subdirs += 1;
            continue;
        }

        if !is_markdown(&path) {
            tracing::trace!(?path, "skipping non-markdown file");
            stats.skipped_non_markdown += 1;
            continue;
        }

        stats.markdown += 1;
        spawn_read(path, entry.file_name(), tx.clone());
    }

    Ok(stats)
}

fn is_markdown(path: &path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "md")
}

fn spawn_read(path: path::PathBuf, file_name: ffi::OsString, tx: NoteSender) {
    tokio::task::spawn(async move {
        let raw_note = tokio::fs::read_to_string(path)
            .await
            .map(|content| RawNote::from((file_name, content)));
        let _ = tx.send(raw_note).await;
    });
}
