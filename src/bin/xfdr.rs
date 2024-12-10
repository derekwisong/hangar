//! Export an X-Plane Flight Data Recorder (FDR) file from an avionics log file.
//!
//! FDR files may be replayed in X-Plane to visualize flight path and telemetry data. This is useful as a post-flight
//! debriefing and analysis tool, for creating videos, or for sharing flight data with others.
//!
//! The FDR file format is a simple csv-like text format which is described inside example files in the "Instructions"
//! directory of the X-Plane installation.

use clap::Parser;
use hangar::{
    detect::{detect_source, AvionicsLogSource},
    fdr::{FDRFileVersion4, FDRWriter},
};
use std::{path::PathBuf, process::ExitCode};

/// Command line arguments for the binary
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to avionics log file
    input: PathBuf,

    /// Path to output FDR file
    output: Option<PathBuf>,
}


/// Parse an avionics log source into an FDR file format
fn parse_avionics_log(source: AvionicsLogSource) -> Result<FDRFileVersion4, String> {
    match source {
        AvionicsLogSource::Garmin(path) => match hangar::garmin::EISData::from_csv(&path) {
            Ok(data) => Ok(data.into()),
            Err(e) => Err(format!("Error reading Garmin data file: {}", e)),
        },
        AvionicsLogSource::Unknown => Err("Unable to parse an unknown avionics log file type".to_string()),
    }
}

/// Entrypoint for the xfdr binary.
fn main() -> ExitCode {
    // parse and validate args
    let args = Args::parse();

    // detect source and parse it into the FDR data structure
    let fdr = match detect_source(&args.input) {
        Ok(AvionicsLogSource::Unknown) => {
            eprintln!("Unable to recognize avionics log source: {}", args.input.display());
            return ExitCode::FAILURE;
        }
        // detected a known source, parse it
        Ok(p) => match parse_avionics_log(p) {
            Ok(fdr) => fdr, // return the parsed data
            Err(e) => {
                eprintln!("Parsing error: {}", e);
                return ExitCode::FAILURE;
            }
        },
        // The input file was not found
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("File not found: {}", args.input.display());
            return ExitCode::FAILURE;
        }
        // An error occured while detecting the source
        Err(e) => {
            eprintln!("Detection error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // if an output file is specified, create a writer using it, otherwise use stdout
    let writer = match hangar::fdr::get_writer(args.output.as_deref()) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Output error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // write data and exit
    return match fdr.write_fdr(writer) {
        Ok(_) => ExitCode::SUCCESS,
        // ignore broken pipe erorrs on stdout (as when on linux when piping output to head)
        Err(ref e) if args.output.is_none() && e.kind() == std::io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Writing error: {}", e);
            ExitCode::FAILURE
        }
    };
}
