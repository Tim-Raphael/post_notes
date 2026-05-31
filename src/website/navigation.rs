//! Navigation tree — a hierarchy of tags with the notes tagged at each level.
//!
//! A tag like `math/probability/independence` splits into a path of nested
//! `TagNode`s so the sidebar can render an expandable outline.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::notes;
use crate::website::html::Internal;

use super::Page;

/// One node in the navigation tree. Both `child_tags` and `files` are sorted
/// so serialization is deterministic across runs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TagNode {
    pub tag: String,
    pub child_tags: Vec<TagNode>,
    pub files: Vec<Internal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Navigation {
    pub root: TagNode,
}

impl Navigation {
    pub fn from_pages(pages: &[Page]) -> Self {
        // Build in a mutable intermediate form (BTreeMap for sort order),
        // then freeze into the sorted vector-of-structs shape the template
        // consumes.
        let mut root = RawTagNode::root();

        for page in pages {
            for tag in &page.frontmatter.tags {
                // Support hierarchical tags like `math/set/properties` by
                // splitting on `/`. Empty segments (leading/duplicate
                // slashes) are dropped.
                let parts: Vec<&str> = tag.split('/').filter(|p| !p.is_empty()).collect();
                if parts.is_empty() {
                    continue;
                }

                let mut current = &mut root;
                for part in &parts {
                    current = current
                        .child_tags
                        .entry(part.to_string())
                        .or_insert_with(|| RawTagNode::named(part));
                }
                current.files.insert(page.link.clone());
            }
        }

        Navigation { root: root.into() }
    }
}

/// Mutable builder for [`TagNode`]. Uses `BTreeMap`/`BTreeSet` so the
/// eventual conversion is naturally sorted.
struct RawTagNode {
    tag: String,
    child_tags: BTreeMap<String, RawTagNode>,
    files: std::collections::BTreeSet<Internal>,
}

impl RawTagNode {
    fn root() -> Self {
        Self::named("#")
    }

    fn named(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            child_tags: BTreeMap::new(),
            files: std::collections::BTreeSet::new(),
        }
    }
}

impl From<RawTagNode> for TagNode {
    fn from(raw: RawTagNode) -> Self {
        Self {
            tag: raw.tag,
            child_tags: raw.child_tags.into_values().map(TagNode::from).collect(),
            files: raw.files.into_iter().collect(),
        }
    }
}

// Compile-time nudge: `Tag` doesn't hand us the raw string directly, so we
// go through `Deref<Target = str>`.
#[allow(dead_code)]
fn _tag_is_str(tag: &notes::Tag) -> &str {
    tag
}
