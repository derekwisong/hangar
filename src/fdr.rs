//! The X-Plane Flight Data Recorder (FDR) file format

// A .fdr file is a text files that contains lines of data in a csv-like format with a few header lines:
// - The first describes the line-ending types "A" (Apple) or "I" (IBM) and then the appropriate line ending.
// - The second line contains the file version: 3 or 4.
// - Subsequent lines are blank or contain data in the format of: FIELD, val1, val2, ..., valN (where N is the
//   number of values for that field).

// The following fields are recognized by X-Plane:
// COMM: any comment
// ACFT: the aircraft file to use, with full directory path from the X-Plane folder (ex: Aircraft/Laminar Research/Boeing B747-400/747-400.acf).
// TAIL: tail number of the aircraft (ex: N12345). Must come immediately after the ACFT line.
// TIME: ZULU time of the beginning of the flight (ex: 18:00:00).
// DATE: date of the flight (ex: 08/10/2004).
// PRES: sea-level pressure during the flight in inches HG (ex: 29.83).
// TEMP: sea-level temperature during the flight in degrees Fahrenheit (ex: 65).
// WIND: wind during the flight in degrees then knots (ex: 230,16).
// CALI: the actual takeoff or touchdown longitude, latitude, and elevation in feet for calibration to X-Plane scenery. (ex: -118.34, 34.57, 456).
// WARN: time to play a warning sound file, with full directory path from X-Plane itself to the .wav file (ex: 5,Resources/sounds/alert/1000ft.WAV).
// TEXT: time & text to be read aloud by computer speech synthesis software (5,This is a test of the spoken text.).
// MARK: time at which a text marker will appear in the time slider (ex: 5,Marker at 5 seconds).
// EVNT: highlights the flight path at the specified time, for a specified duration (ex: 10.5).
// DATA: comma-delimited floating-point numbers that make up the bulk of the .fdr data (ex: 5,10 & see explanation table below)

// In version 4 files, omit the DATA fields and instead the raw csv data will be added to the end of the file.
// In the csv data, the first 7 columns must be:
//      zulu time (hh:mm:ss), longitude, latitude, altitude (feet),
//      magnetic heading (degrees), pitch (degrees), roll (degrees)
// The remaining columns may be any data you wish to include. Each additional column you provide must be associated
// with a DREF entry higher up in the file. These DREF entries must be made in the order the columns appear.

// For example:
// Each of these is a dataref in X-Plane, followed by a unit conversion to go from your input to the X-Plane units
// See the total list of datarefs at https://developer.x-plane.com/datarefs/
// DREF, sim/cockpit2/gauges/actuators/barometer_setting_in_hg_pilot	1.0			// comment:
// DREF, sim/cockpit2/gauges/indicators/airspeed_kts_pilot				1.0			// comment:
// DREF, sim/cockpit2/gauges/indicators/true_airspeed_kts_pilot		1.0			// comment:
// DREF, sim/cockpit2/gauges/indicators/ground_speed_kt				1.0			// comment:
// DREF, sim/cockpit2/gauges/indicators/altitude_ft_pilot				1.0			// comment:
// DREF, sim/cockpit2/gauges/indicators/vvi_fpm_pilot					1.0			// comment:
// DREF, sim/cockpit2/temperature/outside_air_temp_degc				1.0			// comment:
// DREF, sim/cockpit2/electrical/battery_voltage_indicated_volts[0]	1.0			// comment: [0] to indicate first bus. V
// DREF, sim/cockpit2/electrical/battery_voltage_indicated_volts[1]	1.0			// comment: [1] to indicate secnd bus. V
// DREF, sim/flightmodel/weight/m_fuel[0]								2.84		// comment: [0] to indicate first tank. constant to go from lb to X-Planes' kg
// DREF, sim/flightmodel/weight/m_fuel[1]								2.84		// comment: [1] to indicate secnd tank. constant to go from lb to X-Planes' kg
// DREF, sim/cockpit2/engine/indicators/fuel_flow_kg_sec[0]			0.0007396	// comment: [0] to indicate first engine. constant to go from gal/hr to X-Planes' kg/sec
// DREF, sim/cockpit2/engine/indicators/fuel_pressure_psi[0]			1.0			// comment: [0] to indicate first engine. psi
// DREF, sim/cockpit2/engine/indicators/oil_temperature_deg_C[0]		1.0			// comment: [0] to indicate first engine. C
// DREF, sim/cockpit2/engine/indicators/oil_pressure_psi[0]			1.0			// comment: [0] to indicate first engine. psi
// DREF, sim/cockpit2/engine/indicators/torque_n_mtr[0]				1.3558		// comment: [0] to indicate first engine. constant to go from ft-lb to X-Planes' newton-mtr
// DREF, sim/cockpit2/engine/indicators/prop_speed_rsc[0]				0.10472		// comment: [0] to indicate first engine. constant to go from RPM to X-Planes' rad/sec
// DREF, sim/cockpit2/engine/indicators/N1_percent[0]					1.0			// comment: [0] to indicate first engine. %
// DREF, sim/cockpit2/engine/indicators/ITT_deg_C[0]					1.0			// comment: [0] to indicate first engine. C
// DREF, sim/cockpit2/radios/actuators/nav1_frequency_hz				100.0		// comment: constant to do the whole mhz-khz-hz-decimal thing
// DREF, sim/cockpit2/radios/actuators/nav2_frequency_hz				100.0		// comment: constant to do the whole mhz-khz-hz-decimal thing
// DREF, sim/cockpit2/radios/actuators/com1_frequency_hz				100.0		// comment: constant to do the whole mhz-khz-hz-decimal thing
// DREF, sim/cockpit2/radios/actuators/com2_frequency_hz				100.0		// comment: constant to do the whole mhz-khz-hz-decimal thing

use polars::prelude::*;
use std::{io::Write, path::PathBuf};

pub trait FDRField {
    fn field_name(&self) -> &str;
    fn field_values(&self) -> Vec<String>;
}

pub struct AircraftField {
    pub aircraft: String,
}

pub struct CommentField {
    pub comment: String,
}

pub struct DrefField {
    pub dref: String,
    pub conversion_factor: f64,
}

pub struct TailNumberField {
    pub tail_number: String,
}

pub struct FlightTimeField {
    pub time: String,
}

pub struct CalibrationField {
    pub longitude: f64,
    pub latitude: f64,
    pub elevation: i32,
}

pub struct FlightDateField {
    pub date: String,
}

pub struct SeaLevelPressureField {
    pub pressure: f64,
}
pub struct SeaLevelTemperatureField {
    pub temperature: f64,
}

pub struct WindField {
    pub direction: i32,
    pub speed: i32,
}

pub struct WarningField {
    pub time: i32,
    pub sound: String,
}

pub struct MarkerField {
    pub time: i32,
    pub text: String,
}

pub struct EventField {
    pub time: f64,
}

pub struct TextField {
    pub time: i32,
    pub text: String,
}

impl FDRField for DrefField {
    fn field_name(&self) -> &str {
        "DREF"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.dref.clone(), self.conversion_factor.to_string()]
    }
}

impl FDRField for CommentField {
    fn field_name(&self) -> &str {
        "COMM"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.comment.clone()]
    }
}

impl FDRField for AircraftField {
    fn field_name(&self) -> &str {
        "ACFT"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.aircraft.clone()]
    }
}

impl FDRField for TailNumberField {
    fn field_name(&self) -> &str {
        "TAIL"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.tail_number.clone()]
    }
}

impl FDRField for FlightTimeField {
    fn field_name(&self) -> &str {
        "TIME"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.clone()]
    }
}

impl FDRField for FlightDateField {
    fn field_name(&self) -> &str {
        "DATE"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.date.clone()]
    }
}

impl FDRField for SeaLevelPressureField {
    fn field_name(&self) -> &str {
        "PRES"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.pressure.to_string()]
    }
}

impl FDRField for SeaLevelTemperatureField {
    fn field_name(&self) -> &str {
        "TEMP"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.temperature.to_string()]
    }
}

impl FDRField for WindField {
    fn field_name(&self) -> &str {
        "WIND"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.direction.to_string(), self.speed.to_string()]
    }
}

impl FDRField for CalibrationField {
    fn field_name(&self) -> &str {
        "CALI"
    }

    fn field_values(&self) -> Vec<String> {
        vec![
            self.longitude.to_string(),
            self.latitude.to_string(),
            self.elevation.to_string(),
        ]
    }
}

impl FDRField for WarningField {
    fn field_name(&self) -> &str {
        "WARN"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.to_string(), self.sound.clone()]
    }
}

impl FDRField for TextField {
    fn field_name(&self) -> &str {
        "TEXT"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.to_string(), self.text.clone()]
    }
}

impl FDRField for MarkerField {
    fn field_name(&self) -> &str {
        "MARK"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.to_string(), self.text.clone()]
    }
}

impl FDRField for EventField {
    fn field_name(&self) -> &str {
        "EVNT"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.to_string()]
    }
}

pub trait FDRWriter {
    fn serialize_field(&self, field: &dyn FDRField) -> String {
        let mut line = field.field_name().to_string();
        for value in field.field_values() {
            line.push_str(",");
            line.push_str(&value);
        }
        line
    }

    fn write_fdr(&self, destination: &Option<PathBuf>) -> std::io::Result<()>;
}

// create an alias for Box<dyn Write>
pub type Writer = Box<dyn Write>;

/// Construct a writer for the given path, or stdout if no path is provided.
fn get_writer(path: Option<&std::path::Path>) -> std::io::Result<Writer> {
    match path {
        Some(p) => std::fs::File::create(p).map(|f| Box::new(f) as Writer),
        None => Ok(Box::new(std::io::stdout())),
    }
}

pub struct FDRFileVersion4 {
    pub fields: Vec<Box<dyn FDRField>>,
    pub data: DataFrame,
}

impl FDRFileVersion4 {
    /// Create a new FDR file version 4
    pub fn new(data: DataFrame, fields: Option<Vec<Box<dyn FDRField>>>) -> Self {
        FDRFileVersion4 {
            fields: match fields {
                Some(fields) => fields,
                None => Vec::new(),
            },
            data: data,
        }
    }

    /// Add a field to the FDR
    pub fn add_field(&mut self, field: Box<dyn FDRField>) {
        self.fields.push(field);
    }
}

impl FDRWriter for FDRFileVersion4 {
    fn write_fdr(&self, destination: &Option<PathBuf>) -> std::io::Result<()> {
        // if an output file is specified, create a writer for it, otherwise stdout
        let mut writer = match get_writer(destination.as_deref()) {
            Ok(writer) => writer,
            Err(e) => return Err(e),
        };

        writeln!(writer, "A")?;
        writeln!(writer, "4")?;

        for field in &self.fields {
            writeln!(writer, "{}", self.serialize_field(&**field))?;
        }

        const REQUIRED_COLS: [&str; 7] = ["Timestamp", "Longitude", "Latitude", "AltB", "HDG", "Pitch", "Roll"];

        let mut df = self
            .data
            .select(REQUIRED_COLS)
            .expect("Missing required columns")
            .drop_nulls::<String>(None)
            .expect("Unable to shape the data");

        // convert Timestamp to hh:mm:ss
        let ts = df.column("Timestamp").unwrap().datetime().unwrap().strftime("%H:%M:%S");
        if let Ok(ts) = ts {
            df.with_column(ts).unwrap();
        }

        let result = CsvWriter::new(writer).include_header(false).finish(&mut df);

        match result {
            Ok(_) => Ok(()),
            Err(PolarsError::IO { error, msg }) => Err(std::io::Error::new(
                error.kind(),
                msg.map_or_else(|| error.to_string(), |m| m.to_string()),
            )),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        }
    }
}
