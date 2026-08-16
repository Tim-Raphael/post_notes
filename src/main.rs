use std::io;
use std::sync;

use post_notes::notes::FileSystemReader;
use post_notes::{Notes, Settings, website};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        // TODO: Used for debugging stuff
        //.with(tracing_subscriber::fmt::Layer::default())
        // tracing-scribe builds each line from several `write!` calls, and the default unbuffered
        // stderr drops its lock between them. The reader's scan task closes its span while the
        // pipeline is still logging, so two threads splice into one line. Handing the layer an
        // already-locked stderr holds one lock per line instead of one per write.
        .with(tracing_scribe::ConsoleLayer::with_writer(|| {
            io::stderr().lock()
        }))
        .init();

    let settings = sync::Arc::new(Settings::new());

    let notes = Notes::new(FileSystemReader::new());
    // The reader already logs the failure inside its span, so logging again here would print a
    // stray child line after the root span has closed.
    let notes = notes
        .sync(&settings.input)
        .await
        .map_err(|err| anyhow::anyhow!("read phase failed: {err}"))?;

    let website = website::build(notes.notes(), &settings)?;

    let writer = website::FileSystemWriter::from_settings(&settings);
    website::Publisher::new(writer).write(&website, &settings)?;

    Ok(())
}
