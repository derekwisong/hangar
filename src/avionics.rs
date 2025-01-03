//! Detects the type of an avionics log file's type

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::fdr::FDRFileVersion4;
use crate::garmin::{GarminEISLog, GarminEISLogHeader, GarminToFDRBuilder};

/// The source of an avionics log
pub enum AvionicsLogSource {
    /// Log file from a Garmin product, such as the G500 TXi EIS
    Garmin(PathBuf),
}

impl AvionicsLogSource {
    pub fn to_fdr4(&self, aircraft: String, tail_number_override: Option<String>) -> Result<FDRFileVersion4, String> {
        const DEFAULT_TAIL_NUMBER: &str = "N12345";

        match self {
            AvionicsLogSource::Garmin(path) => match GarminEISLog::from_csv(&path) {
                Ok(data) => {
                    let mut builder = GarminToFDRBuilder::new(aircraft, DEFAULT_TAIL_NUMBER.to_string());

                    if let Some(tail_number) = tail_number_override {
                        builder = builder.with_tail_number_override(tail_number);
                    }

                    Ok(builder.build(data))
                },
                Err(e) => Err(format!("Error reading Garmin data file: {}", e)),
            },
        }
    }
}

/// Detect the source of an avionics log file. If the source is not recognized, returns None.
pub fn detect_source(path: &Path) -> Result<Option<AvionicsLogSource>, std::io::Error> {
    // Currently, only Garmin files are supported.
    // This is where future detection logic will go.
    match GarminEISLogHeader::from_csv(path) {
        Ok(_header) => Ok(Some(AvionicsLogSource::Garmin(path.to_path_buf()))),
        Err(e)
            if vec![
                ErrorKind::NotFound,
                ErrorKind::PermissionDenied,
                ErrorKind::IsADirectory,
                ErrorKind::NotSeekable,
            ]
            .contains(&e.kind()) =>
        {
            Err(e)
        }
        // other remaining errors indicate the format of the file was not recognized
        Err(e) => {
            eprintln!(
                "DEBUG: Error '{:?}' is being disregarded as the format being unrecognized.",
                e
            );
            Ok(None)
        } // the format is not recognized
    }
}
