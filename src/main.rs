
fn main() {
    let path = "/mnt/d/Flying/Flight Logs/log_231104_084813_KPOU.csv";
    let df = hangar::read_csv(&std::path::Path::new(path)).unwrap();
    println!("{:?}", df);
}
