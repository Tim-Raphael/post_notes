//! Build phase: compose [`crate::notes::Note`]s into a renderable [`Website`] and write it through
//! the [`WriteWebsite`] I/O boundary.

pub mod builder;
pub mod content;
pub mod html;
pub mod navigation;
pub mod page;
pub mod writer;

pub use builder::build;
pub use content::Content;
pub use html::Internal;
pub use navigation::Navigation;
pub use page::Page;
pub use writer::{Artifact, FileSystemWriter, InMemoryWriter, WriteWebsite, write};

/// One rendered HTML file ready to hit the writer.
///
/// Separate from [`page::Page`] because a `Page` is what the *builder* composes (metadata +
/// un-templated HTML body), while a `RenderedPage` is what the *writer* consumes (fully-templated
/// HTML + the media the page owns).
#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub link: html::Internal,
    pub html: html::Html,
    pub media_links: Vec<html::Media>,
}

/// The output of the build phase: a self-contained site model that can be handed to any
/// [`WriteWebsite`] implementor.
#[derive(Debug, Clone)]
pub struct Website {
    pub pages: Vec<RenderedPage>,
    pub content: Content,
    pub navigation: Navigation,
}
