use derive_more::Display;
use rayon::prelude::*;
use std::fs::{self, ReadDir};
use std::process;

const DEFAULT_CONTENT_FOLDER_LOCATION: &str = "./content";
const DEFAULT_TEMPLATE_FOLDER_LOCATION: &str = "./templates";

#[derive(Debug, Clone)]
struct PostNote {
    title: String,
    slug: String,
    tags: Option<Vec<String>>,
    content: String,
}

#[derive(Display, Debug, Clone, Copy)]
enum TryFromPathBufError {}

impl TryFrom<String> for PostNote {
    type Error = TryFromPathBufError;

    fn try_from(content: String) -> Result<Self, Self::Error> {
        dbg!(&content);
        Ok(Self {
            title: "".to_string(),
            slug: "".to_string(),
            tags: None,
            content: "".to_string(),
        })
    }
}

fn main() {
    let dir = fs::read_dir(DEFAULT_CONTENT_FOLDER_LOCATION).unwrap_or_else(|err| {
        eprintln!("Error: Could not load content - {err}");
        process::exit(1)
    });
    let post_notes: Vec<PostNote> = load_content(dir);
}

/// Parses directory and turn all into PostNotes
fn load_content(dir: ReadDir) -> Vec<PostNote> {
    dir.par_bridge()
        .filter_map(|entry_result| match entry_result {
            Ok(entry) => Some(entry.path()),
            Err(err) => {
                eprintln!("Warning: Could not read directory entry - {err}");
                None
            }
        })
        .filter(|path_buf| {
            path_buf
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext_str| ext_str.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
        })
        .filter_map(|path_buf| match fs::read_to_string(&path_buf) {
            Ok(string) => Some(string),
            Err(err) => {
                eprintln!("Warning: Could not read file {:?} - {err}", &path_buf);
                None
            }
        })
        .filter_map(|string| match PostNote::try_from(string) {
            Ok(post_note) => Some(post_note),
            Err(err) => {
                eprintln!("Warning: Could not parse content - {err}");
                None
            }
        })
        .collect()
}

fn load_templates() {}

fn build() {}
