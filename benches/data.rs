use criterion::{criterion_group, criterion_main, Criterion};

const SAMPLE_CSV: &str = "log_231104_084813_KPOU.csv";
fn resource_path(filename: &str) -> std::path::PathBuf {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources");
    d.push(filename);
    d
}

pub fn read_csv_eager(c: &mut Criterion) {
    let p = resource_path(SAMPLE_CSV);
    c.bench_function("read_csv", |b| b.iter(|| hangar::read_eis(&p).unwrap()));
}


criterion_group!(benches, read_csv_eager);
criterion_main!(benches);
