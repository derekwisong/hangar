
fn main() {
    let path = "resources/log_231104_084813_KPOU.csv";
    let data = hangar::EISData::from_csv(&std::path::Path::new(path)).unwrap();
    println!("{:?}", data.data);
}
