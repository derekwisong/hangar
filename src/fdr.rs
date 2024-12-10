use crate::garmin::EISData;
use polars::prelude::*;
use std::io::Write;

/// Convert a Garmin avionics log file into the X-Plane FDR v4 format
impl From<EISData> for FDRFileVersion4 {
    fn from(data: EISData) -> Self {
        let mut fields: Vec<Box<dyn FDRField>> = vec![
            Box::new(AircraftField {
                aircraft: "Aircraft/Laminar Research/Cirrus SR22/Cirrus SR22.acf".to_string(),
            }),
            Box::new(TailNumberField {
                tail_number: data
                    .header
                    .metadata
                    .get("tail_number")
                    .map_or("N12345".to_string(), |v| v.clone()),
            }),
        ];

        // If there is a time point in the data, add the time fields to the FDR
        if let Some(first_time) = data.first_time() {
            let first_timestamp = first_time.to_utc();
            let first_time = first_timestamp.format("%H:%M:%S").to_string();
            let first_date = first_timestamp.format("%m/%d/%Y").to_string();
            fields.push(Box::new(FlightTimeField { time: first_time }));
            fields.push(Box::new(FlightDateField { date: first_date }));
        }

        FDRFileVersion4 {
            data: data.data,
            fields: fields,
        }
    }
}

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

    fn write_fdr(&self, writer: Writer) -> std::io::Result<()>;
}

// create an alias for Box<dyn Write>
pub type Writer = Box<dyn Write>;

/// Construct a writer for the given path, or stdout if no path is provided.
pub fn get_writer(path: Option<&std::path::Path>) -> std::io::Result<Writer> {
    match path {
        Some(p) => std::fs::File::create(p).map(|f| Box::new(f) as Writer),
        None => Ok(Box::new(std::io::stdout())),
    }
}

pub struct FDRFileVersion4 {
    pub fields: Vec<Box<dyn FDRField>>,
    pub data: DataFrame,
}

impl FDRWriter for FDRFileVersion4 {
    fn write_fdr(&self, mut writer: Writer) -> std::io::Result<()> {
        writeln!(writer, "A")?;
        writeln!(writer, "4")?;

        for field in &self.fields {
            writeln!(writer, "{}", self.serialize_field(&**field))?;
        }

        const REQUIRED_COLS: [&str; 7] = [
            "Timestamp",
            "Longitude",
            "Latitude",
            "AltB",
            "HDG",
            "Pitch",
            "Roll",
        ];

        let mut df = self
            .data
            .select(REQUIRED_COLS)
            .expect("Missing required columns")
            .drop_nulls::<String>(None)
            .expect("Unable to shape the data");

        let result = CsvWriter::new(writer).include_header(false).finish(&mut df);

        match result {
            Ok(_) => Ok(()),
            Err(PolarsError::IO { error, msg }) => Err(std::io::Error::new(
                error.kind(),
                msg.map_or_else(|| error.to_string(), |m| m.to_string()),
            )),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }
}
