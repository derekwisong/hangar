use polars::prelude::*;
use std::{collections::HashMap, io::BufRead};

#[doc(hidden)]
pub fn resource_path(filename: &str) -> std::path::PathBuf {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources");
    d.push(filename);
    d
}

#[derive(Debug)]
pub struct EISColumn {
    name: String,
    unit: String,
}

impl EISColumn {
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
pub struct EISHeader {
    pub metadata: HashMap<String, String>,
    pub columns: Vec<EISColumn>,
}

pub struct EISData {
    pub header: EISHeader,
    pub data: DataFrame,
}

impl EISHeader {
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
            columns.push(EISColumn {
                name: name.to_string(),
                unit: unit.trim().to_string(),
            });
        }

        Ok(Self { metadata, columns })
    }
}

impl EISData {
    pub fn from_csv(path: &std::path::Path) -> PolarsResult<Self> {
        let header = EISHeader::from_csv(path)?;
        let data = read_eis(path)?;
        let data = clean_eis(data)?;
        Ok(Self { header, data })
    }
}

// Clean up a raw column name by trimming whitespace
pub fn clean_column_name(name: &str) -> &str {
    name.trim()
}

// Clean the column names of a dataframe using clean_column_name
pub fn strip_column_names(mut df: DataFrame) -> Result<DataFrame, PolarsError> {
    df.set_column_names(
        &df.get_columns()
            .iter()
            .map(|s| clean_column_name(s.name()).to_string())
            .collect::<Vec<String>>(),
    )?;
    Ok(df)
}

/// drop rows where all values in that row are null
pub fn remove_empty_rows(mut df: DataFrame) -> Result<DataFrame, PolarsError> {
    let mask = df
        .get_columns()
        .iter()
        .fold(None, |acc, s| match acc {
            None => Some(s.is_not_null()),
            Some(mask) => Some(mask & s.is_not_null()),
        })
        .unwrap();
    df = df.filter(&mask)?;
    Ok(df)
}

pub fn clean_eis(mut df: DataFrame) -> Result<DataFrame, PolarsError> {
    df = strip_column_names(df)?;
    df = remove_empty_rows(df)?;
    Ok(df)
}

pub fn read_eis(path: &std::path::Path) -> PolarsResult<DataFrame> {
    const SKIP_ROWS: usize = 2;
    // let lines = num_lines(path)?;
    let df = CsvReadOptions::default()
        .with_skip_rows(SKIP_ROWS) // skip the first 2 rows
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;
    let df = clean_eis(df)?;
    Ok(df)
}

