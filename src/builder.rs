use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde_json::json;
use tera::{Context, Tera};

use crate::content_map::ContentMap;
use crate::navigation::Navigation;
use crate::post_note::PostNote;

pub struct Builder {
    notes: Vec<PostNote>,
    navigation: Navigation,
    content_map: ContentMap,
    tera: Tera,
    content_path: PathBuf,
    output_path: PathBuf,
    static_path: PathBuf,
}

impl Builder {
    pub fn new(
        notes: Vec<PostNote>,
        content_map: ContentMap,
        navigation: Navigation,
        content_location: &str,
        output_location: &str,
        template_location: &str,
        static_location: &str,
    ) -> anyhow::Result<Self> {
        let tera = Tera::new(&format!("{}/**/*.html", template_location))?;

        let content_path = PathBuf::from(content_location);
        let output_path = PathBuf::from(output_location);
        let static_path = PathBuf::from(static_location);

        Ok(Self {
            notes,
            navigation,
            content_map,
            tera,
            content_path,
            output_path,
            static_path,
        })
    }

    pub fn build(self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.output_path)?;
        self.copy_static_dir(&self.static_path, &self.output_path)?;
        self.copy_media_files(&self.content_path, &self.output_path)?;
        self.write_content_map()?;
        self.render_notes()?;

        Ok(())
    }

    fn copy_static_dir(
        &self,
        src: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> io::Result<()> {
        fs::create_dir_all(&destination)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                self.copy_static_dir(entry.path(), destination.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
            }
        }

        Ok(())
    }

    fn copy_media_files(
        &self,
        src: impl AsRef<Path> + Sync,
        destination: impl AsRef<Path> + Sync,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(&destination)?;

        self.notes.par_iter().for_each(|note| {
            note.media_links.par_iter().for_each(|media_link| {
                // TODO!
                let media_path = PathBuf::from(media_link.to_string());
                let output_media_path = PathBuf::from(media_link.to_string().replace(" ", "%20"));

                if let Some(parent) = media_path.parent() {
                    if let Err(err) = fs::create_dir_all(destination.as_ref().join(parent)) {
                        log::warn!("Could not create parent directory: {}", err);
                    };
                }

                if let Err(err) = fs::copy(
                    src.as_ref().join(&media_path),
                    destination.as_ref().join(&output_media_path),
                ) {
                    log::warn!(
                        "Could not copy file {:?} into output directory: {}",
                        src.as_ref().join(&media_path),
                        err
                    );
                }
            })
        });

        Ok(())
    }

    fn write_content_map(&self) -> anyhow::Result<()> {
        let map_json = serde_json::to_string(&json!(self.content_map))?;
        let path = self.output_path.join("map.json");

        fs::write(&path, map_json)?;
        log::info!("Created the content map at: {}", path.display());

        Ok(())
    }

    fn render_notes(&self) -> anyhow::Result<()> {
        self.notes.par_iter().for_each(|note| {
            let mut context = Context::new();

            if let Err(err) = context.try_insert("note", note) {
                log::error!("Failed to insert note for {:?}: {}", &note.file_name, err);
                return;
            }

            if let Err(err) = context.try_insert("navigation", &self.navigation) {
                log::error!(
                    "Failed to insert navigation for {:?}: {}",
                    &note.file_name,
                    err
                );
                return;
            }

            let content = match self.tera.render("base.html", &context) {
                Ok(content) => content,
                Err(err) => {
                    log::error!("Rendering failed for {:?}: {}", note.file_name, err);
                    return;
                }
            };

            let path = self.output_path.join(note.file_name.to_string());
            if let Err(err) = fs::write(&path, content) {
                log::error!("Writing failed for {}: {}", path.display(), err);
            } else {
                log::info!("Rendered: {}", path.display());
            }
        });

        Ok(())
    }
}
