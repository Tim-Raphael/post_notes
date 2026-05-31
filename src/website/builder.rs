//! Build phase: turn parsed [`Note`]s into a ready-to-write [`Website`].
//!
//! Walks each note's `mdast` once to (a) rewrite wikilinks and media embeds into HTML-friendly
//! forms, (b) collect link/media inventories, and (c) render HTML. Then composes the site-wide
//! navigation and search map from the collected pages.

use anyhow::{Context as _, anyhow};
use markdown::mdast;
use rayon::prelude::*;
use tera::Tera;

use crate::Settings;
use crate::notes::Note;
use crate::website::content::Content;
use crate::website::html::{Html, Internal, Media};
use crate::website::navigation::Navigation;
use crate::website::page::{Page, PageFrontmatter};
use crate::website::{RenderedPage, Website};

/// Build a [`Website`] from parsed notes. Public entry point of the build phase.
///
/// Rendering per-page is embarrassingly parallel, so we hand it to rayon. Failures on individual
/// pages are logged and dropped rather than sinking the whole build — matches the read phase's
/// resilience policy.
#[tracing::instrument(name = "website::build", skip_all, fields(notes = notes.len()))]
pub fn build(notes: &[Note], settings: &Settings) -> anyhow::Result<Website> {
    let tera = load_templates(settings)?;

    let pages: Vec<Page> = notes
        .par_iter()
        .filter_map(|note| match build_page(note) {
            Ok(page) => Some(page),
            Err(err) => {
                tracing::warn!(file = %note.file_name, error = ?err, "skipping page");
                None
            }
        })
        .collect();

    // Deterministic page order for stable navigation/map output.
    let mut pages = pages;
    pages.sort_by(|a, b| a.link.cmp(&b.link));

    let navigation = Navigation::from_pages(&pages);
    let content = Content::from_pages(&pages);

    // Render templates in a second pass so every page has access to the full navigation and content
    // map.
    let rendered: Vec<RenderedPage> = pages
        .par_iter()
        .filter_map(|page| match render_page(&tera, page, &navigation) {
            Ok(html) => Some(RenderedPage {
                link: page.link.clone(),
                html,
                media_links: page.media_links.clone(),
            }),
            Err(err) => {
                tracing::error!(file = %page.file_name, error = ?err, "template render failed");
                None
            }
        })
        .collect();

    Ok(Website {
        pages: rendered,
        content,
        navigation,
    })
}

fn load_templates(settings: &Settings) -> anyhow::Result<Tera> {
    let glob = format!("{}/**/*.html", settings.templates.display());
    let mut tera = Tera::new();
    tera.load_from_glob(&glob)
        .with_context(|| format!("loading templates from {glob}"))?;
    Ok(tera)
}

fn build_page(note: &Note) -> anyhow::Result<Page> {
    let link = Internal::from_target(&note.file_name.inner);

    // Clone the tree — we mutate wikilink URLs in place before rendering.
    let mut tree = note.body.inner.clone();
    let mut internal_links = Vec::new();
    let mut media_links = Vec::new();
    collect_and_rewrite(&mut tree, &mut internal_links, &mut media_links);

    let html_str =
        render_mdast_to_html(&tree).with_context(|| format!("rendering {}", note.file_name))?;

    Ok(Page {
        link,
        file_name: note.file_name.inner.clone(),
        frontmatter: PageFrontmatter::from(&note.frontmatter),
        internal_links,
        media_links,
        html: Html::new(html_str),
    })
}

/// Walk the tree once to:
/// - rewrite `Link` nodes whose destination looks like another note
///   (`foo` or `foo.md`) to point at `foo.html`
/// - collect `Image` destinations into `media_links`
fn collect_and_rewrite(
    node: &mut mdast::Node,
    internal_links: &mut Vec<Internal>,
    media_links: &mut Vec<Media>,
) {
    match node {
        mdast::Node::Link(link) => {
            if is_internal_target(&link.url) {
                let internal = Internal::from_target(&link.url);
                link.url = internal.to_string();
                internal_links.push(internal);
            }
        }
        mdast::Node::Image(image) if !is_external(&image.url) => {
            // External URLs are left alone; anything relative is a media
            // asset we need to copy alongside the site output.
            media_links.push(Media::new(image.url.clone()));
        }
        _ => {}
    }

    if let Some(children) = node.children_mut() {
        for child in children {
            collect_and_rewrite(child, internal_links, media_links);
        }
    }
}

fn is_internal_target(url: &str) -> bool {
    !is_external(url) && !url.starts_with('#')
}

fn is_external(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("mailto:")
        || url.starts_with("//")
}

/// Render an mdast node to HTML.
///
/// The `markdown` crate exposes `to_html_with_options` only for source
/// strings, not for pre-parsed trees. We serialize the tree back to
/// markdown and re-render — imperfect but avoids reimplementing an mdast
/// HTML renderer.
fn render_mdast_to_html(node: &mdast::Node) -> anyhow::Result<String> {
    // Skip the yaml frontmatter child so it doesn't render as a code
    // block in the output.
    let mut without_frontmatter = node.clone();
    if let Some(children) = without_frontmatter.children_mut() {
        children.retain(|c| !matches!(c, mdast::Node::Yaml(_)));
    }

    let md = mdast_to_markdown(&without_frontmatter);
    let mut options = markdown::Options::gfm();
    options.parse.constructs.math_flow = true;
    options.parse.constructs.math_text = true;
    options.compile.allow_dangerous_html = true;

    markdown::to_html_with_options(&md, &options).map_err(|err| anyhow!("{}", err.reason))
}

/// Best-effort mdast -> markdown serializer.
///
/// We only need to round-trip the constructs our notes use: paragraphs,
/// headings, lists, code, links, images, inline emphasis. This is
/// deliberately incomplete; extend as new constructs appear in the corpus.
fn mdast_to_markdown(node: &mdast::Node) -> String {
    let mut out = String::new();
    write_md(node, &mut out);
    out
}

fn write_md(node: &mdast::Node, out: &mut String) {
    use mdast::Node;

    match node {
        Node::Root(root) => {
            for (i, child) in root.children.iter().enumerate() {
                if i > 0 {
                    out.push_str("\n\n");
                }
                write_md(child, out);
            }
        }
        Node::Paragraph(p) => {
            for child in &p.children {
                write_md(child, out);
            }
        }
        Node::Heading(h) => {
            for _ in 0..h.depth {
                out.push('#');
            }
            out.push(' ');
            for child in &h.children {
                write_md(child, out);
            }
        }
        Node::Text(t) => out.push_str(&t.value),
        Node::Emphasis(e) => {
            out.push('*');
            for child in &e.children {
                write_md(child, out);
            }
            out.push('*');
        }
        Node::Strong(s) => {
            out.push_str("**");
            for child in &s.children {
                write_md(child, out);
            }
            out.push_str("**");
        }
        Node::InlineCode(c) => {
            out.push('`');
            out.push_str(&c.value);
            out.push('`');
        }
        Node::Code(c) => {
            out.push_str("```");
            if let Some(lang) = &c.lang {
                out.push_str(lang);
            }
            out.push('\n');
            out.push_str(&c.value);
            out.push_str("\n```");
        }
        Node::Link(l) => {
            out.push('[');
            for child in &l.children {
                write_md(child, out);
            }
            out.push_str("](");
            out.push_str(&l.url);
            out.push(')');
        }
        Node::Image(i) => {
            out.push_str("![");
            out.push_str(&i.alt);
            out.push_str("](");
            out.push_str(&i.url);
            out.push(')');
        }
        Node::List(list) => {
            for (idx, child) in list.children.iter().enumerate() {
                if idx > 0 {
                    out.push('\n');
                }
                if list.ordered {
                    out.push_str(&format!("{}. ", idx + 1));
                } else {
                    out.push_str("- ");
                }
                write_md(child, out);
            }
        }
        Node::ListItem(item) => {
            for (i, child) in item.children.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                write_md(child, out);
            }
        }
        Node::Blockquote(bq) => {
            out.push_str("> ");
            for child in &bq.children {
                write_md(child, out);
            }
        }
        Node::ThematicBreak(_) => out.push_str("---"),
        Node::Break(_) => out.push_str("  \n"),
        Node::Math(m) => {
            out.push_str("$$\n");
            out.push_str(&m.value);
            out.push_str("\n$$");
        }
        Node::InlineMath(m) => {
            out.push('$');
            out.push_str(&m.value);
            out.push('$');
        }
        Node::Html(h) => out.push_str(&h.value),
        Node::Yaml(_) => {} // Skip frontmatter.
        // For anything else, best-effort recurse and hope for readable output.
        other => {
            if let Some(children) = other.children() {
                for child in children {
                    write_md(child, out);
                }
            }
        }
    }
}

fn render_page(tera: &Tera, page: &Page, navigation: &Navigation) -> anyhow::Result<Html> {
    let mut ctx = tera::Context::new();
    ctx.insert("note", page);
    ctx.insert("navigation", navigation);
    let rendered = tera.render("base.html", &ctx)?;
    Ok(Html::new(rendered))
}
