use std::io;
use std::sync;
use tokio;
use tokio_stream;

use crate::notes;
use crate::settings;

#[derive(Clone, Debug)]
pub struct Reader<Settings>
where
    Settings: settings::Provide,
{
    settings: sync::Arc<Settings>,
}

impl<S> Reader<S>
where
    S: settings::Provide,
{
    pub fn new(settings_provider: sync::Arc<S>) -> Self {
        Self {
            settings: settings_provider,
        }
    }
}

impl<S> notes::Read for Reader<S>
where
    S: settings::Provide + Sync + Send + 'static,
{
    async fn read(&self) -> impl tokio_stream::Stream<Item = io::Result<notes::types::RawNote>> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let settings = self.settings.clone();

        tokio::task::spawn(async move {
            let result = async {
                let input = settings.input();
                tracing::debug!("Starting to read from {input:?} dir");
                let mut read_dir = tokio::fs::read_dir(input).await?;

                while let Some(entry) = read_dir.next_entry().await? {
                    let tx = tx.clone();
                    let path = entry.path();

                    let Some(extension) = path.extension() else {
                        tracing::error!("Could not determine file extension: {path:?}");
                        continue;
                    };

                    if extension != "md" {
                        tracing::debug!("Skipping non-markdonw file: {path:?}");
                        continue;
                    }

                    tokio::task::spawn(async move {
                        let file_name = entry.file_name();
                        let raw_note = tokio::fs::read_to_string(entry.path())
                            .await
                            .map(|content| notes::types::RawNote::from((file_name, content)));

                        let _ = tx.send(raw_note).await;
                    });
                }

                Ok(())
            }
            .await;

            if let Err(err) = result {
                let _ = tx.send(Err(err)).await;
            }
        });

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}
