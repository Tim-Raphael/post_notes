#[derive(Clone, Debug)]
pub struct Frontmatter {
    pub title: Option<super::Title>,
    pub description: Option<super::Description>,
    pub image: Option<super::Image>,
    pub tags: Option<Vec<super::Tag>>,
    pub created: Option<super::Timestamp>,
    pub modified: Option<super::Timestamp>,
    pub public: bool,
}
