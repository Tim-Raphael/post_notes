//! A rendered page: metadata + HTML body + links extracted from the note.
//!
//! Produced by [`crate::website::Builder`] from a parsed [`crate::notes::Note`].

use serde::Serialize;

use crate::notes;
use crate::website::html::{Html, Internal, Media};

/// Everything the writer needs to emit one `<slug>.html`.
///
/// Serialized field names are chosen to match the existing templates: `properties` for frontmatter
/// and `html_content` for the rendered body. Keeping the Rust field names ergonomic (`frontmatter`,
/// `html`) and the template names historical (`properties`, `html_content`) is done via
/// `serde(rename = ...)`.
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    /// Site-relative link to this page (`{slug}.html`).
    pub link: Internal,
    /// Retained for logging and template display.
    pub file_name: String,
    #[serde(rename = "properties")]
    pub frontmatter: PageFrontmatter,
    pub internal_links: Vec<Internal>,
    pub media_links: Vec<Media>,
    /// Rendered HTML body, without the surrounding template chrome.
    #[serde(rename = "html_content")]
    pub html: Html,
}

/// Serialization-friendly mirror of [`notes::Frontmatter`]: all optional fields are unwrapped to
/// sensible defaults so templates don't need to null-check every access.
#[derive(Debug, Clone, Serialize)]
pub struct PageFrontmatter {
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub tags: Vec<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

impl From<&notes::Frontmatter> for PageFrontmatter {
    fn from(fm: &notes::Frontmatter) -> Self {
        Self {
            title: fm.title.as_ref().map(|t| t.0.clone()).unwrap_or_default(),
            description: fm
                .description
                .as_ref()
                .map(|d| d.0.clone())
                .unwrap_or_default(),
            image: fm.image.as_ref().map(|i| i.0.to_string_lossy().to_string()),
            tags: fm.tags.iter().map(|t| (**t).to_string()).collect(),
            created: fm.created.as_ref().map(|t| t.0.clone()),
            modified: fm.modified.as_ref().map(|t| t.0.clone()),
        }
    }
}
