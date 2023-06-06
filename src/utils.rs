use std::{fmt::Display, path::PathBuf};

use prettytable::{Cell, Row, Table};

use crate::common::Column;

pub fn make_printable_table<I, K: Display>(
    header: Vec<String>,
    lines: I,
    columns: Vec<Column>,
) -> Result<Table, String>
where
    I: IntoIterator<Item = Result<String, K>>,
{
    let mut table = Table::new();
    table.add_row(Row::new(
        columns
            .iter()
            .map(|i| header[i.index].clone())
            .map(|s| Cell::new(&s))
            .collect(),
    ));
    for line in lines {
        let line = line.map_err(|e| e.to_string())?;
        let raw_row_data: Vec<&str> = line.split(",").collect();
        let row_data: Vec<Cell> = columns
            .iter()
            .map(|i| raw_row_data[i.index])
            .map(|s| Cell::new(s))
            .collect();
        table.add_row(Row::new(row_data));
    }
    Ok(table)
}

pub fn check_table_name(table_name: String, data_base_path: &PathBuf) -> Result<PathBuf, String> {
    let mut file_name = String::from(table_name.to_string());
    file_name.push_str(".txt");
    let file_path_buf = data_base_path.clone().join(&file_name);
    let file_path = file_path_buf.as_path();
    if !file_path.exists() {
        return Err("Table does not exist".to_string());
    }
    Ok(file_path.to_path_buf())
}
