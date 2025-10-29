use anyhow::{Context, Result};
use rayon::prelude::*;
use std::{fs, path::PathBuf};

mod builder;
mod content_map;
mod navigation;
mod post_note;
mod settings;

use builder::build;
use content_map::ContentMap;
use navigation::Navigation;
use post_note::{PostNote, PostNoteEntry};

use crate::settings::get_settings;

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

    log::info!("=== Loading Settings ===");
    let settings = get_settings();

    println!();

    log::info!(
        "=== Starting to load content from {}. ===",
        &settings.path.input.display()
    );
    let post_notes = load_content(&settings.path.input).context("Failed to load content")?;

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
    build(&post_notes, content_map, navigation, &settings).context("Failed to build website")?;

    Ok(())
}

fn load_content(location: &PathBuf) -> Result<Vec<PostNote>> {
    Ok(fs::read_dir(location)?
        .par_bridge()
        .filter_map(|entry_result| match entry_result {
            Ok(entry) => Some(entry.path()),
            Err(err) => {
                log::error!("Could get directory entry: {err}");
                None
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
        .collect())
}
