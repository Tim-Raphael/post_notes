use anyhow::{Context, Result};
use comrak::nodes::NodeValue;
use comrak::{Arena, Options, format_html, parse_document};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Properties {
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub tags: Vec<Tag>,
    pub created: String,
    pub modified: Option<String>,
    pub public: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tag(String);

impl From<&str> for Tag {
    fn from(tag: &str) -> Self {
        Self(tag.trim().to_lowercase())
    }
}

impl From<String> for Tag {
    fn from(tag: String) -> Self {
        Self(tag.trim().to_lowercase())
    }
}

impl Deref for Tag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Html(String);

impl TryFrom<Vec<u8>> for Html {
    type Error = anyhow::Error;

    fn try_from(html_buf: Vec<u8>) -> Result<Self> {
        Ok(Self(String::from_utf8(html_buf)?))
    }
}

impl From<String> for Html {
    fn from(html: String) -> Self {
        Self(html.trim().to_string())
    }
}

impl Deref for Html {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InternalLink(String);

impl TryFrom<PathBuf> for InternalLink {
    type Error = anyhow::Error;

    fn try_from(mut path_buf: PathBuf) -> Result<Self> {
        path_buf.set_extension("html");

        let file_name = path_buf
            .file_name()
            .context("Could not determine file name")?
            .to_string_lossy()
            .to_string();

        Ok(Self(file_name))
    }
}

impl From<String> for InternalLink {
    fn from(link: String) -> Self {
        let (path_part, rest) = link
            .split_once(['#', '?'])
            .map(|(head, _tail)| (head, &link[head.len()..]))
            .unwrap_or((&link[..], ""));

        let mut full = path_part.trim_start_matches('/').to_string();

        if !full.ends_with(".html") {
            full.push_str(".html");
        }

        full.push_str(rest);

        Self(full)
    }
}

impl Deref for InternalLink {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaLink(String);

impl From<String> for MediaLink {
    fn from(image: String) -> Self {
        Self(image)
    }
}

impl Deref for MediaLink {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PostNote {
    pub file_name: InternalLink,
    pub properties: Properties,
    pub internal_links: Vec<InternalLink>,
    pub media_links: Vec<MediaLink>,
    pub html_content: Html,
}

impl PostNote {
    fn new(
        file_name: InternalLink,
        properties: Properties,
        internal_links: Vec<InternalLink>,
        media_links: Vec<MediaLink>,
        html_content: Html,
    ) -> Self {
        Self {
            file_name,
            properties,
            media_links,
            internal_links,
            html_content,
        }
    }
}

pub enum PostNoteEntry {
    Public(Box<PostNote>),
    Private,
}

impl PostNoteEntry {
    pub fn new(file_name: &Path, raw_md: &str) -> Result<PostNoteEntry> {
        let (pre_processed_raw_md, media) = match pre_process_media_wikilinks(raw_md) {
            Ok((md, media)) => (md, media),
            Err(err) => {
                log::warn!("Could not pre-process media wikilinks: {}", err);
                (Cow::from(raw_md), Vec::new())
            }
        };

        let arena = Arena::new();
        let mut options = Options::default();

        options.extension.table = true;
        options.extension.math_dollars = true;
        options.extension.wikilinks_title_after_pipe = true;
        options.extension.front_matter_delimiter = Some("---".to_owned());

        let root = parse_document(&arena, &pre_processed_raw_md, &options);

        let file_name = InternalLink::try_from(file_name.to_path_buf())?;
        let mut maybe_properties: Option<Properties> = Option::None;
        let mut links: Vec<InternalLink> = Vec::new();

        for node in root.descendants() {
            match &mut node.data.borrow_mut().value {
                NodeValue::FrontMatter(raw_front_matter) => {
                    let raw_yml = raw_front_matter.replace("---", "").replace("\\n", "");
                    let front_matter: Properties = serde_yaml::from_str(&raw_yml)?;

                    if !front_matter.public {
                        return Ok(Self::Private);
                    }

                    maybe_properties = Some(front_matter);
                }

                NodeValue::WikiLink(link) => {
                    let internal_link = InternalLink::from(link.url.to_owned());
                    link.url = internal_link.to_string();
                    links.push(internal_link);
                }

                // Clip everything that comes after `## Questions`. This is done because I'm to
                // busy to think of a propper way to render my anki cards.
                NodeValue::Heading(heading) => {
                    if heading.level == 2
                        && let Some(first_child) = node.first_child()
                    {
                        let borrowed = first_child.data.borrow();
                        if let NodeValue::Text(ref text) = borrowed.value
                            && text == "Questions"
                        {
                            let mut next_sibling = node.next_sibling();

                            while let Some(sibling) = next_sibling {
                                next_sibling = sibling.next_sibling();
                                sibling.detach();
                            }

                            if let Some(previous_sibling) = node.previous_sibling() {
                                previous_sibling.detach();
                            }

                            node.detach();

                            break;
                        }
                    }
                }

                _ => {}
            }
        }

        let properties = maybe_properties.context("Could not determine properties!")?;

        let mut html_buf = Vec::new();
        format_html(root, &options, &mut html_buf)?;

        let html = Html::try_from(html_buf)?;

        Ok(Self::Public(Box::new(PostNote::new(
            file_name, properties, links, media, html,
        ))))
    }
}

// This is probably going to be a temporary solution.
fn pre_process_media_wikilinks(raw_md: &str) -> Result<(Cow<'_, str>, Vec<MediaLink>)> {
    let re = Regex::new(r"!\[\[(media/[^|\]]+)(?:\|([^\[\]]+))?\]\]")?;
    let mut media_links = Vec::new();

    let pre_processed_raw_md = re.replace_all(raw_md, |caps: &regex::Captures| {
        let link = MediaLink::from(caps[1].to_string());
        let title = caps.get(2).map_or("", |m| m.as_str());

        media_links.push(link.clone());

        format!("![{}](./{})", title, link.to_string().replace(" ", "%20"))
    });

    Ok((pre_processed_raw_md, media_links))
}
