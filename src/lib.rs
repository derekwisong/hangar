use polars::prelude::*;
use polars::df;
// use std::io::BufRead;


// fn num_lines(path: &std::path::Path) -> Result<usize, std::io::Error> {
//     let file = std::fs::File::open(path)?;
//     let reader = std::io::BufReader::new(file);
//     Ok(reader.lines().count())
// }

pub fn get_schema(path: &std::path::Path) -> PolarsResult<Schema> {
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .with_skip_rows(2)
        .with_n_rows(Some(1))
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;
    
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