use serde::Serialize;

use crate::types;

/// Fully parsed and render-ready note model.
#[derive(Debug, Clone, Serialize)]
pub struct Note {
    pub file_name: types::html::link::Internal,
    pub frontmatter: super::Frontmatter,
    pub internal_links: Vec<types::html::link::Internal>,
    pub media_links: Vec<types::html::link::Media>,
    pub html: types::html::Html,
}

impl Note {
    /// Creates a new parsed note.
    pub fn new(
        file_name: types::html::link::Internal,
        frontmatter: super::Frontmatter,
        internal_links: Vec<types::html::link::Internal>,
        media_links: Vec<types::html::link::Media>,
        html: types::html::Html,
    ) -> Self {
        Self {
            file_name,
            frontmatter,
            media_links,
            internal_links,
            html,
        }
    }
}
