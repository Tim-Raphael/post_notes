use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::post_note::{InternalLink, PostNote, Tag};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagNode {
    pub tag: Tag,
    pub child_tags: HashMap<Tag, TagNode>,
    pub files: HashSet<InternalLink>,
}

impl Hash for TagNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag.hash(state);
    }
}

impl Default for TagNode {
    fn default() -> Self {
        TagNode {
            tag: Tag::from("#"),
            child_tags: HashMap::new(),
            files: HashSet::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Navigation {
    pub root: TagNode,
}

impl From<&Vec<Box<PostNote>>> for Navigation {
    fn from(notes: &Vec<Box<PostNote>>) -> Self {
        let mut root = TagNode::default();

        for note in notes {
            for tag in &note.properties.tags {
                let parts: Vec<&str> = tag.split('/').filter(|p| !p.is_empty()).collect();

                if parts.is_empty() {
                    continue;
                }

                let mut current_node = &mut root;

                for part in &parts {
                    let tag_part = Tag::from(*part);

                    current_node = current_node
                        .child_tags
                        .entry(tag_part.clone())
                        .or_insert_with(|| TagNode {
                            tag: tag_part,
                            ..Default::default()
                        });
                }

                current_node.files.insert(note.file_name.clone());

                log::info!(
                    "Inserted {} under the tag {}",
                    note.file_name.to_string(),
                    tag.to_string()
                );
            }
        }

        Navigation { root }
    }
}
