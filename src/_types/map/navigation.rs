use crate::types;

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawTagNode {
    pub tag: types::note::Tag,
    pub child_tags: HashMap<types::note::Tag, RawTagNode>,
    pub files: HashSet<types::html::link::Internal>,
}

impl Hash for RawTagNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag.hash(state);
    }
}

impl Default for RawTagNode {
    fn default() -> Self {
        RawTagNode {
            tag: types::note::Tag::from("#"),
            child_tags: HashMap::new(),
            files: HashSet::new(),
        }
    }
}

impl From<RawTagNode> for TagNode {
    fn from(raw_tag_node: RawTagNode) -> Self {
        let child_tags = {
            let mut child_tags = raw_tag_node
                .child_tags
                .into_iter()
                .map(|value| value.1.into())
                .collect::<Vec<TagNode>>();
            child_tags.sort_unstable();
            child_tags
        };

        let files = {
            let mut files = raw_tag_node
                .files
                .into_iter()
                .collect::<Vec<types::html::link::Internal>>();
            files.sort_unstable();
            files
        };

        Self {
            tag: raw_tag_node.tag,
            child_tags,
            files,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TagNode {
    pub tag: types::note::Tag,
    pub child_tags: Vec<TagNode>,
    pub files: Vec<types::html::link::Internal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Navigation {
    pub root: TagNode,
}

impl From<&[types::note::Note]> for Navigation {
    fn from(notes: &[types::note::Note]) -> Self {
        let mut root = RawTagNode::default();

        for note in notes {
            for tag in &note.frontmatter.tags {
                let parts: Vec<&str> = tag.split('/').filter(|p| !p.is_empty()).collect();

                if parts.is_empty() {
                    continue;
                }

                let mut current_node = &mut root;

                for part in &parts {
                    let tag_part = types::note::Tag::from(*part);

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
