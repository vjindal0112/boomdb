use std::fmt::Display;

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
