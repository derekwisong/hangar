use criterion::{criterion_group, criterion_main, Criterion};

const SAMPLE_CSV: &str = "/mnt/d/Flying/Flight Logs/log_231104_084813_KPOU.csv";

pub fn read_csv_eager(c: &mut Criterion) {
    let p = std::path::Path::new(SAMPLE_CSV);
    c.bench_function("read_csv", |b| b.iter(|| hangar::read_csv(&p).unwrap()));
}


criterion_group!(benches, read_csv_eager);
criterion_main!(benches);
