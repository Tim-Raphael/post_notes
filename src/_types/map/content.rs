use crate::types;

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
struct SearchProperties<'a> {
    tags: &'a Vec<types::note::Tag>,
    title: &'a str,
    description: &'a str,
}

impl<'a> From<&'a types::note::Frontmatter> for SearchProperties<'a> {
    fn from(props: &'a types::note::Frontmatter) -> Self {
        Self {
            tags: &props.tags,
            title: &props.title,
            description: &props.description,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Content<'a>(HashMap<&'a types::html::link::Internal, SearchProperties<'a>>);

impl<'a> From<&'a [types::note::Note]> for Content<'a> {
    fn from(post_notes: &'a [types::note::Note]) -> Self {
        let mut search_props = HashMap::new();

        for note in post_notes.iter() {
            search_props.insert(&note.file_name, SearchProperties::from(&note.frontmatter));
        }

        Self(search_props)
    }
}
