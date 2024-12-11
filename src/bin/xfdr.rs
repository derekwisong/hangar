//! Export an X-Plane Flight Data Recorder (FDR) file from an avionics log file.
//!
//! FDR files may be replayed in X-Plane to visualize flight path and telemetry data. This is useful as a post-flight
//! debriefing and analysis tool, for creating videos, or for sharing flight data with others.
//!
//! The FDR file format is a simple csv-like text format which is described inside example files in the "Instructions"
//! directory of the X-Plane installation.

use clap::{Parser, ValueEnum};
use hangar::{
    avionics::{detect_source, AvionicsLogSource},
    fdr::FDRWriter,
};
use std::{path::PathBuf, process::ExitCode};

/// Export an X-Plane Flight Data Recorder (FDR) file from an avionics log file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The source of the avionics log file, otherwise auto-detect source
    #[arg(short, long, value_enum)]
    source: Option<AviationLogSourceOption>,

    /// The path to an aircraft file, relative to the X-Plane root, to use
    #[arg(short, long, default_value = "Aircraft/Laminar Research/Cirrus SR22/Cirrus SR22.acf")]
    aircraft: String,

    /// Optionally override any tail number discovered in the avionics log
    #[arg(short, long)]
    tail_number: Option<String>,

    /// Path to an avionics log file
    input: PathBuf,

    /// Path to output FDR file. If not specified, output is written to stdout
    output: Option<PathBuf>,
}

/// Supported avionics log sources that can be used as command line arguments
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum AviationLogSourceOption {
    /// Flight data logs from Garmin Engine Indication System (EIS) products. One such example is the G500 TXi EIS
    Garmin,
    // .. add more sources here as they become known (Avidyne, etc.)
}

impl AviationLogSourceOption {
    /// Create an AvionicsLogSource using the given args
    fn to_avionics_log_source(&self, args: &Args) -> AvionicsLogSource {
        match self {
            Self::Garmin => AvionicsLogSource::Garmin(args.input.clone()),
            // .. add more sources maps here as they become known
        }
    }
}

/// Entrypoint for the xfdr binary
fn main() -> ExitCode {
    // parse and validate args
    let args = Args::parse();

    // detect the source of the avionics log file
    let source = match args.source {
        // if the source was specified, map it to the appropriate source
        Some(source) => source.to_avionics_log_source(&args),
        // if the source was not specified, auto-detect it
        None => match detect_source(&args.input) {
            // a source was detected
            Ok(Some(source)) => source,
            // something unknown was detected
            Ok(None) => {
                eprintln!("Unable to recognize avionics log source: {}", args.input.display());
                return ExitCode::FAILURE;
            }
            // input file was not found
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("File not found: {}", args.input.display());
                return ExitCode::FAILURE;
            }
            // error occured while detecting the source
            Err(e) => {
                eprintln!("Detection error: {}", e);
                return ExitCode::FAILURE;
            }
        },
    };

    // parse the source data
    let fdr = match source.to_fdr4(args.aircraft, args.tail_number) {
        Ok(fdr) => fdr, // return the parsed data
        Err(e) => {
            eprintln!("Parsing error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // write data and exit
    return match fdr.write_fdr(&args.output) {
        Ok(_) => ExitCode::SUCCESS,
        // ignore broken pipe erorrs on stdout (as when on linux when piping output to head)
        Err(ref e) if args.output.is_none() && e.kind() == std::io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Writing error: {}", e);
            ExitCode::FAILURE
        }
    };
}
