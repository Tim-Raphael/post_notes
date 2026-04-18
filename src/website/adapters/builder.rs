use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde_json::json;
use tera::{Context, Tera};

use crate::types;

/// Builds the static site by rendering templates and copying assets.
///
/// Steps:
/// - Initializes the Tera template engine with HTML templates
/// - Creates the output directory structure
/// - Copies all static asset directories to output
/// - Copies media files referenced in notes
/// - Writes the content map index
/// - Renders all notes using templates
///
/// # Errors
///
/// Returns an error if template loading, directory creation, file copying, or rendering fails.
pub fn website(
    notes: &[types::note::Note],
    content_map: types::map::Content,
    navigation: types::map::Navigation,
    settings: &types::settings::Settings,
) -> anyhow::Result<()> {
    let template_pattern = format!("{}/**/*.html", settings.path.template.display());
    let tera = Tera::new(&template_pattern)?;
    for asset_path in &settings.path.assets {
        static_dir(asset_path, &settings.path.output)?;
    }
    media(notes, &settings.path.input, &settings.path.output)?;
    map(content_map, &settings.path.output)?;
    pages(notes, &navigation, &tera, &settings.path.output)?;

    Ok(())
}

fn pages(
    notes: &[types::note::Note],
    navigation: &types::map::Navigation,
    tera: &Tera,
    output_path: &Path,
) -> anyhow::Result<()> {
    notes.par_iter().for_each(|note| {
        let mut context = Context::new();

        if let Err(err) = context.try_insert("note", note) {
            log::error!("Failed to insert note for {:?}: {}", &note.file_name, err);
            return;
        }

        if let Err(err) = context.try_insert("navigation", navigation) {
            log::error!(
                "Failed to insert navigation for {:?}: {}",
                &note.file_name,
                err
            );
            return;
        }

        let content = match tera.render("base.html", &context) {
            Ok(content) => content,
            Err(err) => {
                log::error!("Rendering failed for {:?}: {}", note.file_name, err);
                return;
            }
        };

        let path = output_path.join(note.file_name.to_string());
        if let Err(err) = fs::write(&path, content) {
            log::error!("Writing failed for {}: {}", path.display(), err);
        } else {
            log::info!("Rendered: {}", path.display());
        }
    });

    Ok(())
}

/// Recursively copies a directory tree from source to destination.
///
/// Creates the destination directory if it doesn't exist. For each entry in the source:
/// - Directories are recursively copied
/// - Files are copied directly
///
/// If destination already exists, contents are merged (existing files are overwritten).
///
/// # Errors
///
/// Returns an error if any filesystem operation fails (reading, creating directories, copying).
fn static_dir(from: &Path, to: &Path) -> io::Result<()> {
    // Ensure the destination directory exists before copying contents.
    fs::create_dir_all(to)?;
    // Iterate through all entries in the source directory.
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let from = entry.path();
        let to = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            // Recursively copy subdirectories.
            static_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }

    Ok(())
}

fn media(notes: &[types::note::Note], src: &Path, out: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(out)?;
    notes.par_iter().for_each(|note| {
        note.media_links.par_iter().for_each(|media_link| {
            let media_path = PathBuf::from(media_link.to_string());
            let output_media_path = PathBuf::from(media_link.to_string());
            if let Some(parent) = media_path.parent()
                && let Err(err) = fs::create_dir_all(out.join(parent))
            {
                log::warn!("Could not create parent directory: {}", err);
            };
            if let Err(err) = fs::copy(src.join(&media_path), out.join(&output_media_path)) {
                log::warn!(
                    "Could not copy file {:?} into output directory: {}",
                    &src.join(&media_path),
                    err
                );
            }
        })
    });

    Ok(())
}

fn map(content: types::map::Content, out: &Path) -> anyhow::Result<()> {
    let map_json = serde_json::to_string(&json!(content))?;
    let path = out.join("map.json");

    fs::write(&path, map_json)?;
    log::info!("Created the content map at: {}", path.display());

    Ok(())
}

/// Parses raw markdown sources into public notes.
///
/// Notes that fail parsing or are marked as private are skipped while logging
/// errors, preserving the current behavior.
pub fn notes(raw: Vec<types::note::Source>) -> Vec<types::note::Note> {
    raw.into_iter()
        .filter_map(|(path_buf, raw_md)| {
            let post_note_entry = match PostNoteEntry::new(&path_buf, &raw_md) {
                Ok(post_note_entry) => post_note_entry,
                Err(err) => {
                    log::error!(
                        "Something went wrong while parsing post note {:?}: {}",
                        &path_buf,
                        err
                    );
                    return None;
                }
            };

            let post_note = match post_note_entry {
                PostNoteEntry::Public(post_note) => post_note,
                PostNoteEntry::Private => {
                    log::info!("Skipping private note: {:?}", &path_buf);
                    return None;
                }
            };

            log::info!("Loaded public note: {:?}", &path_buf);

            Some(*post_note)
        })
        .collect()
}

/// Builds the content search map from parsed notes.
pub fn content(notes: &[types::note::Note]) -> types::map::Content<'_> {
    types::map::Content::from(notes)
}

/// Builds the navigation tree from parsed notes.
pub fn navigation(notes: &[types::note::Note]) -> types::map::Navigation {
    types::map::Navigation::from(notes)
}

enum PostNoteEntry {
    Public(Box<types::note::Note>),
    Private,
}

impl PostNoteEntry {
    fn new(file_name: &Path, raw_md: &str) -> Result<PostNoteEntry> {
        let (md, media) = match media(raw_md) {
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

        let root = parse_document(&arena, &md, &options);

        let file_name = types::html::link::Internal::try_from(file_name.to_path_buf())?;
        let mut maybe_properties: Option<types::note::Frontmatter> = Option::None;
        let mut links: Vec<types::html::link::Internal> = Vec::new();

        for node in root.descendants() {
            match &mut node.data.borrow_mut().value {
                NodeValue::FrontMatter(raw_front_matter) => {
                    let raw_yml = raw_front_matter.replace("---", "").replace("\\n", "");
                    let front_matter: types::note::Frontmatter = serde_yaml::from_str(&raw_yml)?;

                    if !front_matter.public {
                        return Ok(Self::Private);
                    }

                    maybe_properties = Some(front_matter);
                }

                NodeValue::WikiLink(link) => {
                    let internal_link = types::html::link::Internal::from(link.url.to_owned());
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

        let html = types::html::Html::try_from(html_buf)?;

        Ok(Self::Public(Box::new(types::note::Note::new(
            file_name, properties, links, media, html,
        ))))
    }
}

// This is probably going to be a temporary solution.
fn media(raw: &str) -> Result<(Cow<'_, str>, Vec<types::html::link::Media>)> {
    let re = Regex::new(r"!\[\[(media/[^|\]]+)(?:\|([^\[\]]+))?\]\]")?;
    let mut media_links = Vec::new();

    let md = re.replace_all(raw, |caps: &regex::Captures| {
        let link = types::html::link::Media::from(caps[1].to_string());
        let title = caps.get(2).map_or("", |m| m.as_str());

        media_links.push(link.clone());

        format!("![{}](./{})", title, link.to_string().replace(" ", "%20"))
    });

    Ok((md, media_links))
}
