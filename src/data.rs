use polars::prelude::*;

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
fn remove_empty_rows(mut df: DataFrame) -> Result<DataFrame, PolarsError> {
    let mask = df
        .get_columns()
        .iter()
        .fold(None, |acc, s| match acc {
            None => Some(s.is_not_null()),
            Some(mask) => Some(mask | s.is_not_null()),
        })
        .unwrap();
    df = df.filter(&mask)?;
    Ok(df)
}

fn clean_strings(mut df: DataFrame) -> PolarsResult<DataFrame> {
    // get names of string columns
    let string_columns = df
        .get_columns()
        .iter()
        .filter(|s| s.dtype() == &DataType::String)
        .map(|s| s.name().to_string())
        .collect::<Vec<String>>();
    use polars::prelude::*;

    // trim whitespace from string columns
    for col in string_columns {
        df.replace(
            &col,
            Series::new(
                &col,
                df.column(&col)?
                    .str()?
                    .into_iter()
                    .map(|opt_s| opt_s.map(|s| s.trim().to_string()))
                    .collect::<Vec<Option<String>>>(),
            ),
        )?;
    }

    Ok(df)
}

pub fn clean_dataframe(mut df: DataFrame) -> Result<DataFrame, PolarsError> {
    df = strip_column_names(df)?;
    df = clean_strings(df)?;
    df = remove_empty_rows(df)?;
    Ok(df)
}
