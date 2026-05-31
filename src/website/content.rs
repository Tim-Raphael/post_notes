//! The `Content` map — a flat search index of every rendered note.
//!
//! Serialized as `map.json` at the site root and consumed by the search
//! widget in the browser.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::website::html::Internal;
use crate::website::page::PageFrontmatter;

use super::Page;

/// Search-relevant metadata for a single note.
#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}

impl From<&PageFrontmatter> for Entry {
    fn from(fm: &PageFrontmatter) -> Self {
        Self {
            title: fm.title.clone(),
            description: fm.description.clone(),
            tags: fm.tags.clone(),
        }
    }
}

/// A `link -> entry` map. Ordered so the serialized JSON is stable across
/// runs (required for the consistency test).
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Content(BTreeMap<Internal, Entry>);

impl Content {
    pub fn from_pages(pages: &[Page]) -> Self {
        let mut map = BTreeMap::new();
        for page in pages {
            map.insert(page.link.clone(), Entry::from(&page.frontmatter));
        }
        Self(map)
    }

    pub fn as_map(&self) -> &BTreeMap<Internal, Entry> {
        &self.0
    }
}
