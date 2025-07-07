use anyhow::{Context, Result};
use rayon::prelude::*;
use std::fs;

mod builder;
mod content_map;
mod navigation;
mod post_note;

use builder::Builder;
use content_map::ContentMap;
use navigation::Navigation;
use post_note::{PostNote, PostNoteEntry};

const DEFAULT_CONTENT_FOLDER: &str = "../notes";
const DEFAULT_OUTPUT_FOLDER: &str = "./output/";
const DEFAULT_TEMPLATE_FOLDER: &str = "./assets/templates";
const DEFAULT_STATIC_FOLDER: &str = "./assets/static";

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
    let args = Args::parse_arguments();

    println!();

    log::info!(
        "=== Starting to load content from {}. ===",
        &args.content_folder
    );
    let post_notes = load_content(&args.content_folder).context("Failed to load content")?;

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
    Builder::new(post_notes, content_map, navigation, &args)?.build()?;

    Ok(())
}

pub struct Args {
    output_folder: String,
    content_folder: String,
    template_folder: String,
    static_folder: String,
}

impl Args {
    fn parse_arguments() -> Self {
        let mut output_folder = None;
        let mut content_folder = None;
        let mut template_folder = None;
        let mut static_folder = None;

        for arg in std::env::args() {
            if let Some(value) = arg.strip_prefix("--output-folder=") {
                output_folder = Some(value.to_string());
                log::info!("Found output folder argument: {}", value);
            } else if let Some(value) = arg.strip_prefix("--content-folder=") {
                content_folder = Some(value.to_string());
                log::info!("Found content folder argument: {}", value);
            } else if let Some(value) = arg.strip_prefix("--template-folder=") {
                template_folder = Some(value.to_string());
                log::info!("Found template folder argument: {}", value);
            } else if let Some(value) = arg.strip_prefix("--static-folder=") {
                static_folder = Some(value.to_string());
                log::info!("Found static folder argument: {}", value);
            }
        }

        Self {
            output_folder: output_folder.unwrap_or_else(|| DEFAULT_OUTPUT_FOLDER.to_string()),
            content_folder: content_folder.unwrap_or_else(|| DEFAULT_CONTENT_FOLDER.to_string()),
            template_folder: template_folder.unwrap_or_else(|| DEFAULT_TEMPLATE_FOLDER.to_string()),
            static_folder: static_folder.unwrap_or_else(|| DEFAULT_STATIC_FOLDER.to_string()),
        }
    }
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
