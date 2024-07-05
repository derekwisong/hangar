use criterion::{criterion_group, criterion_main, Criterion};

pub fn hello(c: &mut Criterion) {
    const PATH: &str = "/mnt/d/Flying/Flight Logs/log_231104_084813_KPOU.csv";
    c.bench_function("read_csv", |b| b.iter(|| 
        hangar::read_csv(&std::path::Path::new(PATH)).unwrap()
    ));
}

criterion_group!(benches, hello);
criterion_main!(benches);
