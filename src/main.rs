use anyhow::{Context, Result};
use rayon::prelude::*;
use std::env::args;
use std::fs;

mod builder;
mod content_map;
mod navigation;
mod post_note;

use builder::Builder;
use content_map::ContentMap;
use navigation::Navigation;
use post_note::{PostNote, PostNoteEntry};

const DEFAULT_CONTENT_FOLDER_LOCATION: &str = "../notes";
const DEFAULT_OUTPUT_FOLDER_LOCATION: &str = "./output/";
const DEFAULT_TEMPLATE_FOLDER_LOCATION: &str = "./assets/templates";
const DEFAULT_STATIC_FOLDER_LOCATION: &str = "./assets/static";

fn main() -> Result<()> {
    print!(
        r#"
       .~@`,
      (__,  \
          \' \
           \  \
            \  \
             \  `._            __.__
              \    ~-._  _.==~~     ~~--.._
               \        '                  ~-.
                \      _-   -_                `.
                 \    /       )        .-    .  \
                  `. |      /  )      (       ;  \
                    `|     /  /       (       :   '\
                     \    |  /        |      /       \
                      |     /`-.______.\     |~-.      \
                      |   |/           (     |   `.      \_
                      |   ||            ~\   \      '._    `-.._____..----..___
                      |   |/             _\   \         ~-.__________.-~~~~~~~~~'''
      post_notes    .o'___/            .o______)


        "#
    );

    colog::init();

    log::info!("=== Parsing arguments. ===");
    let parsed_arguments = parse_arguments();
    let content_folder_location = parsed_arguments
        .0
        .unwrap_or(DEFAULT_CONTENT_FOLDER_LOCATION.to_string());
    let template_folder_location = parsed_arguments
        .1
        .unwrap_or(DEFAULT_TEMPLATE_FOLDER_LOCATION.to_string());
    let output_folder_location = parsed_arguments
        .2
        .unwrap_or(DEFAULT_OUTPUT_FOLDER_LOCATION.to_string());
    let static_folder_location = parsed_arguments
        .3
        .unwrap_or(DEFAULT_STATIC_FOLDER_LOCATION.to_string());

    println!();

    log::info!(
        "=== Starting to load content from {}. ===",
        &content_folder_location
    );
    let post_notes = load_content(&content_folder_location).context("Failed to laod content")?;

    println!();

    log::info!(
        "=== Starting to generate content map with {} entrie(s). ===",
        post_notes.len()
    );
    let content_map = ContentMap::from(&post_notes);

    println!();

    log::info!("=== Starting to generate navigation. ===");
    let navigation = Navigation::from(&post_notes);

    println!();

    log::info!("=== Starting to build website. ===");
    Builder::new(
        post_notes,
        content_map,
        navigation,
        &content_folder_location,
        &output_folder_location,
        &template_folder_location,
        &static_folder_location,
    )?
    .build()?;

    Ok(())
}

// TODO: The function signature should be more clear (typed return value)
fn parse_arguments() -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut output_folder_location = None;
    let mut content_folder_location = None;
    let mut template_folder_location = None;
    let mut static_folder_location = None;

    for arg in args() {
        if let Some(value) = arg.strip_prefix("--output-folder-location=") {
            output_folder_location = Some(value.to_string());
            log::info!("Found argument: {}", &value);
        } else if let Some(value) = arg.strip_prefix("--content-folder-location=") {
            content_folder_location = Some(value.to_string());
            log::info!("Found argument: {}", &value);
        } else if let Some(value) = arg.strip_prefix("--template-folder-location=") {
            template_folder_location = Some(value.to_string());
            log::info!("Found argument: {}", &value);
        } else if let Some(value) = arg.strip_prefix("--static-folder-location=") {
            static_folder_location = Some(value.to_string());
            log::info!("Found argument: {}", &value);
        }
    }

    (
        content_folder_location,
        template_folder_location,
        output_folder_location,
        static_folder_location,
    )
}

fn load_content(location: &str) -> Result<Vec<PostNote>> {
    Ok(fs::read_dir(location)?
        .par_bridge()
        .filter_map(|entry_result| match entry_result {
            Ok(entry) => Some(entry.path()),
            Err(err) => {
                log::error!("Could get derectory entry: {}", err);
                return None;
            }
        })
        .filter(|path_buf| {
            path_buf
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext_str| ext_str == "md")
                .unwrap_or(false)
        })
        .filter_map(|path_buf| {
            let raw_content = match fs::read_to_string(&path_buf) {
                Ok(raw_content) => raw_content,
                Err(err) => {
                    log::error!(
                        "Could not read content of {:?}: {}",
                        path_buf.display(),
                        err
                    );
                    return None;
                }
            };

            Some((path_buf, raw_content))
        })
        .filter_map(|(path_buf, raw_md)| {
            let post_note_visibility = match PostNoteEntry::new(&path_buf, &raw_md) {
                Ok(post_note_visibility) => post_note_visibility,
                Err(err) => {
                    log::error!(
                        "Something went wrong while parsing post note {:?}: {}",
                        &path_buf,
                        err
                    );
                    return None;
                }
            };

            let post_note = match post_note_visibility {
                PostNoteEntry::Public(post_note) => post_note,
                PostNoteEntry::Private => {
                    log::info!("Skipping private note: {:?}", &path_buf);
                    return None;
                }
            };

            log::info!("Loaded public note: {:?}", &path_buf);

            Some(post_note)
        })
        .collect())
}
