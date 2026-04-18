use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Normalized tag value used for navigation and indexing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tag(String);

impl From<&str> for Tag {
    fn from(tag: &str) -> Self {
        Self(tag.trim().to_lowercase())
    }
}

impl From<String> for Tag {
    fn from(tag: String) -> Self {
        Self(tag.trim().to_lowercase())
    }
}

impl Deref for Tag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
