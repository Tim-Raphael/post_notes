use tokio_stream::StreamExt as _;

use std::{io, marker, sync};

use crate::notes;
use crate::settings;

#[derive(Clone, Debug)]
pub struct Unsynced;
#[derive(Clone, Debug)]
pub struct Synced;

#[derive(Clone, Debug)]
pub struct Provider<Reader, State = Unsynced>
where
    Reader: notes::Read,
{
    notes: Vec<notes::types::Note>,
    reader: Reader,
    // TODO: add watcher
    _state: marker::PhantomData<State>,
}

// TODO: will be used to continuously watch directory
#[derive(Debug)]
pub struct SyncError<R>
where
    R: notes::Read,
{
    source: io::Error,
    provider: Provider<R>,
}

impl<R> Provider<R, Unsynced>
where
    R: notes::Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            notes: vec![],
            _state: marker::PhantomData,
        }
    }

    pub async fn sync(self) -> Result<Provider<R, Synced>, SyncError<R>> {
        let mut notes = Vec::new();
        let mut sync_err = None;

        {
            let raw_notes = self.reader.read().await;
            tokio::pin!(raw_notes);

            while let Some(raw_note) = raw_notes.next().await {
                // If an error occurs while reading a file abort, because we don't know if the
                // note would've been something we wanted to render.
                let Ok(raw_note) = raw_note else {
                    let err = raw_note.as_ref().unwrap_err();
                    tracing::error!("Could not read note: {err}");
                    sync_err = Some(raw_note.unwrap_err());
                    break;
                };

                // If an error occurs while parsing the note content we know that the note itself
                // is invalid and probably can't be rendered anyway.
                match notes::types::Note::try_from(raw_note) {
                    Ok(note) => notes.push(note),
                    Err(err) => {
                        tracing::warn!("Could not parse note: {err}")
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

        Ok(Provider {
            notes,
            reader: self.reader,
            _state: marker::PhantomData::<Synced>,
        })
    }
}

impl<R> notes::Provide for Provider<R, Synced>
where
    R: notes::Read,
{
    fn notes(&self) -> &[notes::types::Note] {
        &self.notes
    }
}

impl From<sync::Arc<settings::Provider>> for Provider<notes::Reader<settings::Provider>> {
    fn from(settings: sync::Arc<settings::Provider>) -> Self {
        let reader = notes::Reader::new(settings.clone());
        Self::new(reader)
    }
}

///// Reads markdown note files from the input directory.
/////
///// Non-markdown files are ignored. Directory entry and file-read failures are
///// logged and skipped to preserve the current resilient behavior.
//pub fn notes(path: &std::path::Path) -> Result<Vec<types_::note::Source>> {
//    Ok(fs::read_dir(path)?
//        .par_bridge()
//        .filter_map(|entry_result| match entry_result {
//            Ok(entry) => Some(entry.path()),
//            Err(err) => {
//                log::error!("Could get directory entry: {err}");
//                None
//            }
//        })
//        .filter(|path_buf| {
//            path_buf
//                .extension()
//                .and_then(|ext| ext.to_str())
//                .map(|ext_str| ext_str == "md")
//                .unwrap_or(false)
//        })
//        .filter_map(|path_buf| {
//            let raw_content = match fs::read_to_string(&path_buf) {
//                Ok(raw_content) => raw_content,
//                Err(err) => {
//                    log::error!(
//                        "Could not read content of {:?}: {}",
//                        path_buf.display(),
//                        err
//                    );
//                    return None;
//                }
//            };
//
//            Some((path_buf, raw_content))
//        })
//        .collect())
//}
