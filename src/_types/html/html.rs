use anyhow::Result;
use serde::Serialize;
use std::ops::Deref;

/// HTML document generated from markdown content.
#[derive(Debug, Clone, Serialize)]
pub struct Html(String);

impl TryFrom<Vec<u8>> for Html {
    type Error = anyhow::Error;

    fn try_from(html_buf: Vec<u8>) -> Result<Self> {
        Ok(Self(String::from_utf8(html_buf)?))
    }
}

impl From<String> for Html {
    fn from(html: String) -> Self {
        Self(html.trim().to_string())
    }
}

impl Deref for Html {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
