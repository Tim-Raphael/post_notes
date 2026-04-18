use serde::Serialize;
use std::ops::Deref;

/// Media path referenced by a note.
#[derive(Debug, Clone, Serialize)]
pub struct Media(String);

impl From<String> for Media {
    fn from(image: String) -> Self {
        Self(image)
    }
}

impl Deref for Media {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
