// Build X-Plane flight data recorder (FDR) file from a Garmin EIS data file.
use hangar::garmin;
use std::{fs::File, io::{self, Write}};
use polars::prelude::*;

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

pub trait FDRField {
    fn field_name(&self) -> &str;
    fn field_values(&self) -> Vec<String>;
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
    // fn write_fdr(&self, path: &std::path::Path) -> std::io::Result<()>;
    fn write_fdr(&self, writer: Writer, data: &DataFrame) -> std::io::Result<()>;

    fn print_fdr(&self);
}

pub struct FDRFileVersion4 {
    pub fields: Vec<Box<dyn FDRField>>,
}

impl FDRWriter for FDRFileVersion4 {
    // fn write_fdr(&self, path: &std::path::Path) -> std::io::Result<()> {
    //     let mut file = std::fs::File::create(path)?;
    //     writeln!(file, "A")?;
    //     writeln!(file, "4")?;
    //     for field in &self.fields {
    //         writeln!(file, "{}", self.serialize_field(&**field))?;
    //     }
    //     Ok(())
    // }

    fn write_fdr(&self, mut writer: Writer, data: &DataFrame) -> std::io::Result<()> {
        writeln!(writer, "A")?;
        writeln!(writer, "4")?;
        for field in &self.fields {
            writeln!(writer, "{}", self.serialize_field(&**field))?;
        }
        let mut data = data.clone().lazy()
            .select(vec![
                col("Timestamp").dt().strftime("%H:%M:%S").alias("time"),
                col("Longitude"),
                col("Latitude"),
                col("AltB"),
                col("HDG"),
                col("Pitch"),
                col("Roll")])
            .drop_nans(None)
            .collect().unwrap();

        CsvWriter::new(writer)
            .include_header(false)
            .finish(&mut data).unwrap();
            
        Ok(())
    }

    fn print_fdr(&self) {
        println!("A");
        println!("4");
        for field in &self.fields {
            println!("{}", self.serialize_field(&**field));
        }
    }
}

pub struct CommentField {
    pub comment: String,
}

impl FDRField for CommentField {
    fn field_name(&self) -> &str {
        "COMM"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.comment.clone()]
    }
}

pub struct AircraftField {
    pub aircraft: String,
}

impl FDRField for AircraftField {
    fn field_name(&self) -> &str {
        "ACFT"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.aircraft.clone()]
    }
}

pub struct TailNumberField {
    pub tail_number: String,
}

impl FDRField for TailNumberField {
    fn field_name(&self) -> &str {
        "TAIL"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.tail_number.clone()]
    }
}

pub struct FlightTimeField {
    pub time: String,
}

impl FDRField for FlightTimeField {
    fn field_name(&self) -> &str {
        "TIME"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.clone()]
    }
}

pub struct FlightDateField {
    pub date: String,
}

impl FDRField for FlightDateField {
    fn field_name(&self) -> &str {
        "DATE"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.date.clone()]
    }
}

pub struct SeaLevelPressureField {
    pub pressure: f64,
}

impl FDRField for SeaLevelPressureField {
    fn field_name(&self) -> &str {
        "PRES"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.pressure.to_string()]
    }
}

pub struct SeaLevelTemperatureField {
    pub temperature: f64,
}

impl FDRField for SeaLevelTemperatureField {
    fn field_name(&self) -> &str {
        "TEMP"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.temperature.to_string()]
    }
}

pub struct WindField {
    pub direction: i32,
    pub speed: i32,
}

impl FDRField for WindField {
    fn field_name(&self) -> &str {
        "WIND"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.direction.to_string(), self.speed.to_string()]
    }
}

pub struct CalibrationField {
    pub longitude: f64,
    pub latitude: f64,
    pub elevation: i32,
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

pub struct WarningField {
    pub time: i32,
    pub sound: String,
}

impl FDRField for WarningField {
    fn field_name(&self) -> &str {
        "WARN"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.to_string(), self.sound.clone()]
    }
}

pub struct TextField {
    pub time: i32,
    pub text: String,
}

impl FDRField for TextField {
    fn field_name(&self) -> &str {
        "TEXT"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.to_string(), self.text.clone()]
    }
}

pub struct MarkerField {
    pub time: i32,
    pub text: String,
}

impl FDRField for MarkerField {
    fn field_name(&self) -> &str {
        "MARK"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.to_string(), self.text.clone()]
    }
}

pub struct EventField {
    pub time: f64,
}

impl FDRField for EventField {
    fn field_name(&self) -> &str {
        "EVNT"
    }

    fn field_values(&self) -> Vec<String> {
        vec![self.time.to_string()]
    }
}

// create an alias for Box<dyn Write>
type Writer = Box<dyn Write>;

fn console_writer() -> Writer {
    Box::new(std::io::stdout())
}

fn file_writer(path: &std::path::Path) -> io::Result<Writer> {
    File::create(path).map(|f| Box::new(f) as Writer)
}

fn main() {
    let path = std::env::args().nth(1).expect("No file path provided");
    let output = std::env::args().nth(2);

    let data = garmin::EISData::from_csv(&std::path::Path::new(path.as_str())).unwrap();
    println!("{:?}", data.data);

    let first_timestamp = data.first_time().unwrap().to_utc();
    let first_time = first_timestamp.format("%H:%M:%S").to_string();
    let first_date = first_timestamp.format("%m/%d/%Y").to_string();
    let tail_number = data.header.metadata["tail_number"].clone();

    let fdr = FDRFileVersion4 {
        fields: vec![
            Box::new(CommentField {
                comment: format!(
                    "{} - {} ({}, {}). Converted using eis2fdr.",
                    tail_number,
                    data.header.metadata["airframe_name"].clone(),
                    data.header.metadata["unit"].clone(),
                    data.header.metadata["Product"].clone()
                ),
            }),
            Box::new(AircraftField {
                aircraft: "Aircraft/Laminar Research/Cirrus SR22/Cirrus SR22.acf".to_string(),
            }),
            Box::new(TailNumberField {
                tail_number: tail_number,
            }),
            Box::new(FlightTimeField {
                time: first_time.to_string(),
            }),
            Box::new(FlightDateField { date: first_date }),
            // Box::new(SeaLevelPressureField { pressure: 29.83 }),
            // Box::new(SeaLevelTemperatureField { temperature: 65.0 }),
            // Box::new(WindField { direction: 230, speed: 16 }),
            // Box::new(CalibrationField { longitude: -118.34, latitude: 34.57, elevation: 456 }),
            // Box::new(WarningField { time: 5, sound: "Resources/sounds/alert/1000ft.WAV".to_string() }),
            // Box::new(TextField { time: 5, text: "This is a test of the spoken text.".to_string() }),
            // Box::new(MarkerField { time: 5, text: "Marker at 5 seconds".to_string() }),
            // Box::new(EventField { time: 10.5 }),
        ],
    };

    let writer = match output {
        Some(output) => file_writer(&std::path::Path::new(output.as_str())).unwrap(),
        None => console_writer(),
    };

    fdr.write_fdr(writer, &data.data).unwrap();
}
