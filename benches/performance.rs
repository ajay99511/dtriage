use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;

// Import from the crate
use dtriage::triage::compute_file_hash;
use camino::Utf8Path;

fn bench_file_hashing_1kb(c: &mut Criterion) {
    let temp_file = create_temp_file(1024); // 1KB file
    let path = Utf8Path::from_path(temp_file.path()).unwrap();

    let rt = Runtime::new().unwrap();

    c.bench_function("hash_1kb_file", |b| {
        b.iter(|| {
            rt.block_on(compute_file_hash(black_box(path)))
                .unwrap()
        })
    });
}

fn bench_file_hashing_1mb(c: &mut Criterion) {
    let temp_file = create_temp_file(1024 * 1024); // 1MB file
    let path = Utf8Path::from_path(temp_file.path()).unwrap();

    let rt = Runtime::new().unwrap();

    c.bench_function("hash_1mb_file", |b| {
        b.iter(|| {
            rt.block_on(compute_file_hash(black_box(path)))
                .unwrap()
        })
    });
}

fn bench_file_hashing_10mb(c: &mut Criterion) {
    let temp_file = create_temp_file(10 * 1024 * 1024); // 10MB file
    let path = Utf8Path::from_path(temp_file.path()).unwrap();

    let rt = Runtime::new().unwrap();

    c.bench_function("hash_10mb_file", |b| {
        b.iter(|| {
            rt.block_on(compute_file_hash(black_box(path)))
                .unwrap()
        })
    });
}

fn create_temp_file(size: usize) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    let data = vec![0u8; size];
    temp_file.write_all(&data).unwrap();
    temp_file.flush().unwrap();
    temp_file
}

fn bench_categorizer(c: &mut Criterion) {
    use dtriage::config::CategorizationRule;
    use dtriage::triage::Categorizer;

    let rules = CategorizationRule::defaults();
    let categorizer = Categorizer::new(rules);
    let path = Utf8Path::new("/downloads/test.pdf");

    c.bench_function("categorize_pdf", |b| {
        b.iter(|| categorizer.categorize(black_box(path)))
    });
}

criterion_group!(
    benches,
    bench_file_hashing_1kb,
    bench_file_hashing_1mb,
    bench_file_hashing_10mb,
    bench_categorizer
);

criterion_main!(benches);
