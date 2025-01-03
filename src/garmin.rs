use crate::data::{clean_column_name, clean_dataframe};
use chrono::Utc;
use polars::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, Read};

use crate::fdr::{AircraftField, FDRField, FDRFileVersion4, FlightDateField, FlightTimeField, TailNumberField};

#[derive(Default)]
pub struct GarminToFDRBuilder {
    aircraft: String,
    tail_number_default: String,
    tail_number_override: Option<String>,
}

impl GarminToFDRBuilder {
    pub fn new(
        aircraft: String,
        tail_number_default: String,
    ) -> Self {
        Self {
            aircraft,
            tail_number_default,
            tail_number_override: None,
        }
    }

    pub fn with_tail_number_override(mut self, tail_number: String) -> Self {
        self.tail_number_override = Some(tail_number);
        self
    }

    pub fn build(self, log: GarminEISLog) -> FDRFileVersion4 {
        let mut fields: Vec<Box<dyn FDRField>> = vec![
            Box::new(AircraftField {
                aircraft: self.aircraft,
            }),
            Box::new(TailNumberField {
                tail_number: match self.tail_number_override {
                    Some(tail_number) => tail_number,
                    None => log
                        .header
                        .metadata
                        .get("tail_number")
                        .map_or(self.tail_number_default, |v| v.clone()),
                },
            }),
        ];

        // If there is a time point in the data, add the time fields to the FDR
        if let Some(first_time) = log.first_time() {
            let first_timestamp = first_time.to_utc();
            let first_time = first_timestamp.format("%H:%M:%S").to_string();
            let first_date = first_timestamp.format("%m/%d/%Y").to_string();
            fields.push(Box::new(FlightTimeField { time: first_time }));
            fields.push(Box::new(FlightDateField { date: first_date }));
        }

        FDRFileVersion4::new(log.data, Some(fields))
    }
}

#[derive(Debug)]
pub struct GarminEISColumn {
    name: String,
    unit: String,
}

impl GarminEISColumn {
    pub fn name(&self) -> &str {
        clean_column_name(&self.name)
    }

    pub fn raw_name(&self) -> &str {
        &self.name
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }
}

#[derive(Debug)]
pub struct GarminEISLogHeader {
    pub metadata: HashMap<String, String>,
    pub columns: Vec<GarminEISColumn>,
}

pub struct GarminEISLog {
    pub header: GarminEISLogHeader,
    pub data: DataFrame,
}

impl GarminEISLogHeader {
    pub fn from_csv(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;
        let mut metadata = HashMap::new();
        let mut columns = Vec::new();

        // all header data can be found in the first 3 rows of the file.
        // row 1 starts with a comment char and has metadata entries in the form of key="value" separated by commas
        // row 2 starts with a comment char and has column units separated by commas
        // row 3 lists the column names separated by commas

        let mut lines = std::io::BufReader::new(file).lines();
        let metadata_line = lines.next().expect("No lines in file")?;

        let units_line = lines.next().expect("No units line in file")?;
        let units = units_line.trim_start_matches('#').split(",");

        let names_line = lines.next().expect("No names line in file")?;
        let names = names_line.split(',');

        for entry in metadata_line.trim_start_matches('#').split(',') {
            let mut parts = entry.split('=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    metadata.insert(key.trim().to_string(), value.trim_matches('"').to_string());
                }
            }
        }

        for (unit, name) in units.zip(names) {
            columns.push(GarminEISColumn {
                name: name.to_string(),
                unit: unit.trim().to_string(),
            });
        }

        Ok(Self { metadata, columns })
    }

    pub fn build_schema(&self) -> Schema {
        Schema::from_iter(
            self.columns
                .iter()
                .map(|c| {
                    let dtype = match c.unit() {
                        "yyy-mm-dd" => DataType::Date,
                        "bool" => DataType::Int64,
                        "enum" => DataType::String,
                        "MHz" => DataType::Float64,
                        "degrees" => DataType::Float64,
                        "ft" => DataType::Float64,
                        "nm" => DataType::Float64,
                        "fsd" => DataType::Float64, // indication of full scale deflection?
                        "mt" => DataType::Float64,  // WAAS performance numbers
                        "ft wgs" => DataType::Float64,
                        "ft Baro" => DataType::Float64,
                        "ft msl" => DataType::Float64,
                        "kt" => DataType::Float64,
                        "fpm" => DataType::Float64,
                        "deg" => DataType::Float64,
                        "ft/min" => DataType::Float64,
                        "deg F/min" => DataType::Float64,
                        "kts" => DataType::Float64,
                        "lbs" => DataType::Float64,
                        "gals" => DataType::Float64,
                        "volts" => DataType::Float64,
                        "amps" => DataType::Float64,
                        "gph" => DataType::Float64,
                        "psi" => DataType::Float64,
                        "degF" => DataType::Float64,
                        "deg F" => DataType::Float64,
                        "deg C" => DataType::Float64,
                        "%" => DataType::Float64,
                        "rpm" => DataType::Float64,
                        "inch" => DataType::Float64,
                        "Hg" => DataType::Float64,
                        "G" => DataType::Float64,
                        "#" => DataType::Int64,
                        "s" => DataType::Float64,
                        "crc16" => DataType::String,
                        _ => DataType::String,
                    };
                    Field::new(c.name().into(), dtype)
                })
                .collect::<Vec<_>>(),
        )
    }
}

impl GarminEISLog {
    fn read_bytes(path: &std::path::Path) -> std::io::Result<std::io::Cursor<Vec<u8>>> {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        // remove trailing null bytes from buffer
        while buffer.last() == Some(&0) {
            buffer.pop();
        }
        Ok(std::io::Cursor::new(buffer))
    }

    fn read_df(path: &std::path::Path, schema: &Schema) -> PolarsResult<LazyFrame> {
        // read the file bytes into a buffer
        const SKIP_ROWS: usize = 2;
        // read into dataframe
        let reader = CsvReadOptions::default()
            .with_has_header(true)
            .with_schema(Some(Arc::new(schema.clone())))
            .with_skip_rows(SKIP_ROWS)
            .into_reader_with_file_handle(Self::read_bytes(path)?);
        Ok(reader.finish()?.lazy())
    }

    pub fn from_csv(path: &std::path::Path) -> PolarsResult<Self> {
        let header = GarminEISLogHeader::from_csv(path)?;
        let schema = header.build_schema();
        let data = Self::read_df(path, &schema)?;
        let data = parse_datetime(data, "Lcl Date", "Lcl Time", "UTCOfst", "Timestamp", true)?;
        let data = data.collect()?;
        let data = clean_dataframe(data)?;
        Ok(Self { header, data })
    }

    pub fn first_time(&self) -> Option<chrono::DateTime<Utc>> {
        match self.data.column("Timestamp") {
            Ok(Column::Series(s)) => Some(
                s.datetime()
                    .unwrap()
                    .as_datetime_iter()
                    .next()
                    .unwrap()
                    .unwrap()
                    .and_utc(),
            ),
            _ => None,
        }
    }
}

fn parse_datetime(
    lazy: LazyFrame,
    date_col: &str,
    time_col: &str,
    offset_col: &str,
    new_timestamp_col: &str,
    drop_source_cols: bool,
) -> PolarsResult<LazyFrame> {
    let mut lazy = lazy.with_column(
        concat_str(
            vec![
                col(date_col).dt().strftime("%Y-%m-%d"),
                lit("T"),
                col(time_col),
                col(offset_col),
            ],
            "",
            false,
        )
        .str()
        .to_datetime(
            Some(TimeUnit::Microseconds),
            Some("UTC".into()),
            StrptimeOptions {
                format: Some("%Y-%m-%dT%H:%M:%S%z".into()),
                ..Default::default()
            },
            lit("raise"),
        )
        .alias(new_timestamp_col),
    );

    if drop_source_cols {
        lazy = lazy.drop(vec![date_col, time_col, offset_col]);
    }

    Ok(lazy)
}
