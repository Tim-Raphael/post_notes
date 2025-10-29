use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::post_note::{InternalLink, PostNote, Tag};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawTagNode {
    pub tag: Tag,
    pub child_tags: HashMap<Tag, RawTagNode>,
    pub files: HashSet<InternalLink>,
}

impl Hash for RawTagNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag.hash(state);
    }
}

impl Default for RawTagNode {
    fn default() -> Self {
        RawTagNode {
            tag: Tag::from("#"),
            child_tags: HashMap::new(),
            files: HashSet::new(),
        }
    }
}

impl From<RawTagNode> for TagNode {
    fn from(raw_tag_node: RawTagNode) -> Self {
        let mut child_tags = raw_tag_node
            .child_tags
            .into_iter()
            .map(|value| value.1.into())
            .collect::<Vec<TagNode>>();
        child_tags.sort_unstable();
        let mut files = raw_tag_node
            .files
            .into_iter()
            .collect::<Vec<InternalLink>>();
        files.sort_unstable();
        Self {
            tag: raw_tag_node.tag,
            child_tags,
            files,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TagNode {
    pub tag: Tag,
    pub child_tags: Vec<TagNode>,
    pub files: Vec<InternalLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Navigation {
    pub root: TagNode,
}

impl From<&Vec<PostNote>> for Navigation {
    fn from(notes: &Vec<PostNote>) -> Self {
        let mut root = RawTagNode::default();

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
                        .or_insert_with(|| RawTagNode {
                            tag: tag_part,
                            ..Default::default()
                        });
                }

                current_node.files.insert(note.file_name.clone());

                log::info!("Inserted {} under the tag {}", &*note.file_name, &**tag);
            }
        }

        Navigation { root: root.into() }
    }
}
