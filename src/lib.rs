pub mod garmin;
pub mod data;
pub mod fdr;
pub mod avionics;

#[doc(hidden)]
pub fn resource_path(filename: &str) -> std::path::PathBuf {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources");
    d.push(filename);
    d
}
