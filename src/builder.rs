use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde_json::json;
use tera::{Context, Tera};

use crate::Args;
use crate::content_map::ContentMap;
use crate::navigation::Navigation;
use crate::post_note::PostNote;

pub fn build(
    notes: &Vec<PostNote>,
    content_map: ContentMap,
    navigation: Navigation,
    args: &Args,
) -> anyhow::Result<()> {
    let tera = Tera::new(&format!("{}/**/*.html", &args.template_folder))?;
    let content_path = PathBuf::from(&args.content_folder);
    let output_path = PathBuf::from(&args.output_folder);
    let static_path = PathBuf::from(&args.static_folder);

    fs::create_dir_all(&output_path)?;
    copy_static_dir(&static_path, &output_path)?;
    copy_media_files(notes, &content_path, &output_path)?;
    write_content_map(content_map, &output_path)?;
    render_notes(notes, &navigation, &tera, &output_path)?;

    Ok(())
}

fn render_notes(
    notes: &Vec<PostNote>,
    navigation: &Navigation,
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

fn copy_static_dir(src: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(&destination)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_static_dir(&entry.path(), &destination.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), &destination.join(entry.file_name()))?;
        }
    }

    Ok(())
}

fn copy_media_files(notes: &Vec<PostNote>, src: &Path, destination: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(&destination)?;

    notes.par_iter().for_each(|note| {
        note.media_links.par_iter().for_each(|media_link| {
            let media_path = PathBuf::from(media_link.to_string());
            let output_media_path = PathBuf::from(media_link.to_string().replace(" ", "%20"));

            if let Some(parent) = media_path.parent() {
                if let Err(err) = fs::create_dir_all(&destination.join(parent)) {
                    log::warn!("Could not create parent directory: {}", err);
                };
            }

            if let Err(err) = fs::copy(
                &src.join(&media_path),
                &destination.join(&output_media_path),
            ) {
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

fn write_content_map(content_map: ContentMap, output_path: &Path) -> anyhow::Result<()> {
    let map_json = serde_json::to_string(&json!(content_map))?;
    let path = output_path.join("map.json");

    fs::write(&path, map_json)?;
    log::info!("Created the content map at: {}", path.display());

    Ok(())
}
