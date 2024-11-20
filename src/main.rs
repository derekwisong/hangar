
fn main() {
    let path = "resources/log_231104_084813_KPOU.csv";
    let df = hangar::read_eis(&std::path::Path::new(path)).unwrap();
    println!("{:?}", df);
}
