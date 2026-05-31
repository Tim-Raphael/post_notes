use std::sync;

use post_notes::{Notes, Settings, website};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
#[tracing::instrument(name = "main")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        // TODO: used for debugging stuff
        //.with(tracing_subscriber::fmt::Layer::default())
        .with(tracing_scribe::ConsoleLayer::default())
        .init();

    let settings = sync::Arc::new(Settings::new());

    let notes = Notes::from(settings.clone());
    let notes = match notes.sync().await {
        Ok(notes) => notes,
        Err(err) => {
            tracing::error!(error = %err, "failed to sync notes");
            return Err(anyhow::anyhow!("read phase failed"));
        }
    };

    let website = website::build(notes.notes(), &settings)?;

    let writer = website::FileSystemWriter::from_settings(&settings);
    website::write(&website, &settings, &writer)?;

    Ok(())
}
