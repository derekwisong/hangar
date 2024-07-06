use polars::prelude::*;
use std::{collections::HashMap, io::BufRead};

#[derive(Debug)]
pub struct EISColumn {
    name: String,
    unit: String,
}

impl EISColumn {
    pub fn name(&self) -> &str {
        self.name.trim()
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

impl EISHeader {
    pub fn from_csv2(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;
        let mut metadata = HashMap::new();
        let mut columns = Vec::new();

        // all data can be found in the first 3 rows of the file.
        // row 1 starts with a comment char and has metadata entries in the form of key="value" separated by commas and value quoted
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

    pub fn from_csv(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;
        let mut metadata = HashMap::new();
        let mut columns = Vec::new();

        // all data can be found in the first 3 rows of the file.
        // row 1 starts with a comment char and has metadata entries in the form of key="value" separated by commas and value quoted
        // row 2 starts with a comment char and has column units separated by commas
        // row 3 lists the column names separated by commas

        let mut lines = std::io::BufReader::new(file).lines();
        let line = lines.next().unwrap()?;
        if !line.starts_with('#') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "First line must start with a comment character",
            ));
        }
        let metadata_line = line.trim_start_matches('#');
        for entry in metadata_line.split(',') {
            let mut parts = entry.split('=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    metadata.insert(key.trim().to_string(), value.trim_matches('"').to_string());
                }
            }
        }

        let line = lines.next().unwrap()?;
        if !line.starts_with('#') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Second line must start with a comment character",
            ));
        }
        let units_line = line.trim_start_matches('#');
        for unit in units_line.split(',') {
            columns.push(EISColumn {
                name: "".to_string(),
                unit: unit.to_string(),
            });
        }

        let line = lines.next().unwrap()?;
        if line.starts_with('#') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Third line must not start with a comment character",
            ));
        }
        let names_line = line;
        for (column, name) in columns.iter_mut().zip(names_line.split(',')) {
            column.name = name.to_string();
        }

        Ok(Self { metadata, columns })
    }
}

pub fn read_csv_columns(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
    let mut columns = Vec::new();
    let file = std::fs::File::open(path)?;

    for line in std::io::BufReader::new(file).lines() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        columns = line.split(',').map(|s| s.to_string()).collect();
        break;
    }

    Ok(columns)
}

pub fn strip_column_names(mut df: DataFrame) -> Result<DataFrame, PolarsError> {
    df.set_column_names(
        &df.get_columns()
            .iter()
            .map(|s| s.name().trim().to_string())
            .collect::<Vec<String>>(),
    )?;
    Ok(df)
}

pub fn read_csv(path: &std::path::Path) -> PolarsResult<DataFrame> {
    const SKIP_ROWS: usize = 2;
    // let lines = num_lines(path)?;
    let mut df = CsvReadOptions::default()
        .with_skip_rows(SKIP_ROWS) // skip the first 2 rows
        // .with_n_rows(Some(lines - SKIP_ROWS - 2)) // don't read the last row (it's all null bytes)
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;

    df = strip_column_names(df)?;
    Ok(df)
}

pub fn read_lazy(path: &std::path::Path) -> PolarsResult<DataFrame> {
    const SKIP_ROWS: usize = 2;
    let reader = LazyCsvReader::new(path)
        .with_skip_rows(SKIP_ROWS)
        .with_has_header(true);

    let mut df = reader.finish()?;
    df = df
        // .drop_nulls()
        .filter(col("Lcl Date").is_not_null());
    df.collect()
}
