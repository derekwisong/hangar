use criterion::{criterion_group, criterion_main, Criterion};
use hangar::resource_path;

const SAMPLE_CSV: &str = "log_231104_084813_KPOU.csv";

pub fn read_csv_eager(c: &mut Criterion) {
    let p = resource_path(SAMPLE_CSV);
    c.bench_function("read_csv", |b| b.iter(|| hangar::read_eis(&p).unwrap()));
}


criterion_group!(benches, read_csv_eager);
criterion_main!(benches);
