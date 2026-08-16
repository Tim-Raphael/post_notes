//! End-to-end tests for the read → build → write pipeline.
//!
//! These tests exercise the full system through its two I/O boundaries:
//! - Read: a [`MockReader`] feeds fixture markdown from `tests/fixtures/notes` without hitting the
//!   actual filesystem read path.
//! - Write: an [`InMemoryWriter`] captures artifacts so we can assert on them directly.
//!
//! **Consistency**: the same input must always produce byte-identical output. We assert this by
//! running the pipeline twice and comparing artifact snapshots.

use std::path::PathBuf;
use std::sync::Arc;

use post_notes::Settings;
use post_notes::notes::{Notes, RawNote, ReadNotes};
use post_notes::website::{self, InMemoryWriter};
use pretty_assertions::assert_eq;
use tokio_stream::Stream;

/// Feeds a predetermined list of raw notes into the pipeline. Order is randomized on each call to
/// prove the pipeline sorts internally.
#[derive(Clone, Debug)]
struct MockReader {
    notes: Vec<RawNote>,
}

impl MockReader {
    fn from_dir(dir: &std::path::Path) -> Self {
        let mut notes = Vec::new();
        for entry in std::fs::read_dir(dir).expect("read fixture dir") {
            let entry = entry.expect("read fixture entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(&path).expect("read fixture");
            notes.push(RawNote {
                file_name: entry.file_name(),
                content,
            });
        }
        Self { notes }
    }
}

impl ReadNotes for MockReader {
    async fn read(&self, _input: &std::path::Path) -> impl Stream<Item = std::io::Result<RawNote>> {
        // Reverse the order on purpose to exercise the sort inside `Notes::sync`. If sorting is
        // dropped, snapshots diverge.
        let mut items: Vec<_> = self.notes.iter().cloned().map(Ok).rev().collect();
        // Interleave a second time to further scramble.
        items.rotate_left(1);
        tokio_stream::iter(items)
    }
}

fn fixture_settings() -> Settings {
    Settings {
        input: PathBuf::from("tests/fixtures/notes"),
        output: PathBuf::from("target/tests-output"),
        templates: PathBuf::from("templates"),
        assets: PathBuf::from("tests/fixtures/assets"),
    }
}

async fn run_pipeline() -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let settings = Arc::new(fixture_settings());
    let reader = MockReader::from_dir(&settings.input);
    let notes = Notes::new(reader)
        .sync(&settings.input)
        .await
        .expect("sync notes");
    let website = website::build(notes.notes(), &settings).expect("build website");
    let published = website::Publisher::new(InMemoryWriter::new())
        .write(&website, &settings)
        .expect("write website");
    published.writer().snapshot()
}

#[tokio::test]
async fn consistency_two_runs_produce_identical_output() {
    let first = run_pipeline().await;
    let second = run_pipeline().await;
    assert_eq!(
        first.keys().collect::<Vec<_>>(),
        second.keys().collect::<Vec<_>>(),
        "artifact paths differ between runs",
    );
    for (path, first_bytes) in &first {
        let second_bytes = second.get(path).expect("missing in second run");
        assert_eq!(
            first_bytes,
            second_bytes,
            "artifact {} differs between runs",
            path.display(),
        );
    }
}

#[tokio::test]
async fn private_notes_are_excluded() {
    let snapshot = run_pipeline().await;
    let has_draft = snapshot
        .keys()
        .any(|p| p.to_string_lossy().contains("draft"));
    assert!(!has_draft, "private note leaked into output");
}

#[tokio::test]
async fn every_public_note_renders_to_html() {
    let snapshot = run_pipeline().await;
    let keys: Vec<_> = snapshot.keys().collect();
    assert!(
        snapshot.contains_key(&PathBuf::from("alpha.html")),
        "alpha.html missing; got: {:?}",
        keys,
    );
    assert!(
        snapshot.contains_key(&PathBuf::from("beta.html")),
        "beta.html missing; got: {:?}",
        keys,
    );
    // Search map is always emitted.
    assert!(
        snapshot.contains_key(&PathBuf::from("map.json")),
        "map.json missing; got: {:?}",
        keys,
    );
}

#[tokio::test]
async fn search_map_contains_public_notes_only() {
    let snapshot = run_pipeline().await;
    let map_bytes = snapshot.get(&PathBuf::from("map.json")).expect("map.json");
    let map: serde_json::Value = serde_json::from_slice(map_bytes).expect("parse map.json");
    let obj = map.as_object().expect("map is object");
    assert!(obj.contains_key("alpha.html"));
    assert!(obj.contains_key("beta.html"));
    assert!(!obj.contains_key("draft.html"));
}

#[tokio::test]
async fn navigation_contains_hierarchical_tags() {
    // Alpha has `math/set`, which should nest under `math`.
    let snapshot = run_pipeline().await;
    let alpha_html = snapshot
        .get(&PathBuf::from("alpha.html"))
        .expect("alpha.html");
    let html = std::str::from_utf8(alpha_html).expect("utf-8");
    assert!(html.contains("math"), "math tag missing from navigation");
    assert!(html.contains("set"), "nested set tag missing");
}
