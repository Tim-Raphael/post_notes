//! Efficiency benchmark for the read → build → write pipeline.
//!
//! Baselines the end-to-end time so regressions after refactors are obvious. Runs against
//! `tests/fixtures/notes` through the mock reader and the in-memory writer, so the numbers reflect
//! the pipeline itself rather than disk I/O.
//!
//! Run with `cargo bench --bench pipeline`.

use std::path::PathBuf;
use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use post_notes::Settings;
use post_notes::notes::{Notes, RawNote, ReadNotes};
use post_notes::website::{self, InMemoryWriter};
use tokio_stream::Stream;

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
        let items: Vec<_> = self.notes.iter().cloned().map(Ok).collect();
        tokio_stream::iter(items)
    }
}

fn fixture_settings() -> Settings {
    Settings {
        input: PathBuf::from("tests/fixtures/notes"),
        output: PathBuf::from("target/bench-output"),
        templates: PathBuf::from("templates"),
        assets: PathBuf::from("tests/fixtures/assets"),
    }
}

fn bench_pipeline(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build runtime");

    let settings = Arc::new(fixture_settings());
    let reader = MockReader::from_dir(&settings.input);

    c.bench_function("read_build_write_fixture", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let notes = Notes::new(reader.clone())
                    .sync(&settings.input)
                    .await
                    .expect("sync notes");
                let website = website::build(notes.notes(), &settings).expect("build website");
                let published = website::Publisher::new(InMemoryWriter::new())
                    .write(&website, &settings)
                    .expect("write");
                std::hint::black_box(published.writer().snapshot());
            });
        });
    });
}

criterion_group!(benches, bench_pipeline);
criterion_main!(benches);
