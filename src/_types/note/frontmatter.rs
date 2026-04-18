use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub tags: Option<Vec<super::Tag>>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub public: bool,
}
