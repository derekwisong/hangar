use hangar::garmin;

fn main() {
    let path = "resources/log_231104_084813_KPOU.csv";
    let data = crate::garmin::EISData::from_csv(&std::path::Path::new(path)).unwrap();
    println!("{:?}", data.data);
}
