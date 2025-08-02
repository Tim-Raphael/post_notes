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

impl From<&Vec<PostNote>> for Navigation {
    fn from(notes: &Vec<PostNote>) -> Self {
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

                log::info!("Inserted {} under the tag {}", &*note.file_name, &**tag);
            }
        }

        sort_node(&mut root);

        Navigation { root }
    }
}

fn sort_node(node: &mut TagNode) {
    let mut sorted_children: Vec<_> = node.child_tags.drain().collect();
    sorted_children.sort_by(|a, b| a.0.cmp(&b.0));
    node.child_tags = sorted_children.into_iter().collect();

    for child_node in node.child_tags.values_mut() {
        sort_node(child_node);
    }
}
