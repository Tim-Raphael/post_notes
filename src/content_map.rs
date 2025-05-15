use serde::Serialize;
use std::collections::HashMap;

use crate::post_note::{InternalLink, PostNote, Properties, Tag};

#[derive(Debug, Clone, Serialize)]
struct SearchProperties {
    tags: Vec<Tag>,
    title: String,
    description: String,
}

impl From<Properties> for SearchProperties {
    fn from(props: Properties) -> Self {
        Self {
            tags: props.tags,
            title: props.title,
            description: props.description,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentMap(HashMap<InternalLink, SearchProperties>);

impl From<&Vec<PostNote>> for ContentMap {
    fn from(post_notes: &Vec<PostNote>) -> Self {
        let mut search_props = HashMap::new();

        for note in post_notes.iter() {
            search_props.insert(note.file_name.clone(), note.properties.clone().into());
            log::info!("Generated map entries for {}", note.file_name.to_string());
        }

        Self(search_props)
    }
}
