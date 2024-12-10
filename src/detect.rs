//! Detects the type of an avionics log file's type

use std::path::{Path, PathBuf};

use crate::garmin::EISHeader;

/// The source of an avionics log
pub enum AvionicsLogSource {
    Garmin(PathBuf),  /// Log file from a Garmin product, such as the G500 TXi EIS
    Unknown,
}

/// Detect the source of an avionics log file.
pub fn detect_source(path: &Path) -> Result<AvionicsLogSource, std::io::Error> {
    // Currently, only Garmin files are supported.
    match EISHeader::from_csv(path) {
        Ok(_header) => Ok(AvionicsLogSource::Garmin(path.to_path_buf())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(e),
        Err(_) => Ok(AvionicsLogSource::Unknown), // if the file is found, we dont understand the format
    } 
}
