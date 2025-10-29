use serde::Serialize;
use std::collections::HashMap;

use crate::post_note::{InternalLink, PostNote, Properties, Tag};

#[derive(Debug, Clone, Serialize)]
struct SearchProperties<'a> {
    tags: &'a Vec<Tag>,
    title: &'a str,
    description: &'a str,
}

impl<'a> From<&'a Properties> for SearchProperties<'a> {
    fn from(props: &'a Properties) -> Self {
        Self {
            tags: &props.tags,
            title: &props.title,
            description: &props.description,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentMap<'a>(HashMap<&'a InternalLink, SearchProperties<'a>>);

impl<'a> From<&'a Vec<PostNote>> for ContentMap<'a> {
    fn from(post_notes: &'a Vec<PostNote>) -> Self {
        let mut search_props = HashMap::new();

        for note in post_notes.iter() {
            search_props.insert(&note.file_name, SearchProperties::from(&note.properties));
        }

        Self(search_props)
    }
}
