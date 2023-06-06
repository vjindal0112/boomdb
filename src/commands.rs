use core::panic;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use prettytable::{Cell, Row, Table};
use sqlparser::ast::{ColumnDef, ObjectName, SelectItem, SetExpr};

use crate::common::Column;

fn make_printable_table<I, K: Display>(
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

fn parse_columns(existing_column_names: Vec<String>, projection: Vec<SelectItem>) -> Vec<Column> {
    let mut columns: Vec<Column> = vec![];
    for item in projection {
        let value = match item {
            SelectItem::UnnamedExpr(expr) => match expr {
                sqlparser::ast::Expr::Identifier(ident) => ident.value,
                _ => panic!("Other identifier parameters not supported"),
            },
            SelectItem::ExprWithAlias { expr: _, alias: _ } => {
                panic!("ExprWithAlias not yet supported")
            }
            SelectItem::QualifiedWildcard(_, _) => panic!("QualifiedWildcard not yet supported"),
            SelectItem::Wildcard(wildcard_options) => {
                if wildcard_options.opt_except.is_none()
                    && wildcard_options.opt_exclude.is_none()
                    && wildcard_options.opt_rename.is_none()
                    && wildcard_options.opt_replace.is_none()
                {
                    "*".to_string()
                } else {
                    panic!("Wildcard options not yet supported")
                }
            }
        };
        // TODO: rethink this logic
        if value == "*" {
            for (index, column_name) in existing_column_names.iter().enumerate() {
                columns.push(Column {
                    index,
                    name: column_name.to_string(),
                });
            }
        } else {
            for (index, column_name) in existing_column_names.iter().enumerate() {
                if column_name.to_string() == value {
                    columns.push(Column {
                        index,
                        name: column_name.to_string(),
                    });
                }
            }
        }
    }
    columns
}

pub fn select(query: sqlparser::ast::Query, data_base_path: &PathBuf) -> () {
    let select_query = match *query.body {
        // Assuming body is an Enum and one of its variant is Select
        SetExpr::Select(select_struct) => select_struct,
        _ => panic!("Expected a Select query"),
    };
    let table_name = match &select_query.from[0].relation {
        sqlparser::ast::TableFactor::Table { name, .. } => name,
        _ => panic!("Expected a table name"),
    };
    let projection = select_query.projection;
    let mut file_name = String::from(table_name.to_string());
    file_name.push_str(".txt");
    let file_path_buf = data_base_path.clone().join(&file_name);
    let file_path = file_path_buf.as_path();
    if !file_path.exists() {
        println!("Table {} does not exist", table_name);
        return;
    }
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // header should always be there
    let raw_header_line = match lines.next() {
        Some(line) => line.unwrap(),
        None => return,
    };
    let raw_headers: Vec<String> = raw_header_line.split(",").map(String::from).collect();
    let columns = parse_columns(raw_headers.clone(), projection);
    match make_printable_table(raw_headers, lines, columns) {
        Ok(table) => table.printstd(),
        Err(_) => println!("Error while printing table"),
    }
}

pub fn create_table(name: ObjectName, columns: Vec<ColumnDef>, data_base_path: &PathBuf) -> () {
    println!("Create table: {} with columns:", name);
    for c in columns.clone() {
        println!("\t{:?}", c);
    }
    let mut file_name = String::from(name.to_string());
    file_name.push_str(".txt");
    let mut file = std::fs::File::create(data_base_path.clone().join(&file_name)).unwrap();
    let mut content = String::new();
    for (index, c) in columns.iter().enumerate() {
        content.push_str(c.name.to_string().as_str());
        if index != columns.len() - 1 {
            content.push_str(",");
        }
    }
    file.write_all(content.as_bytes()).unwrap();
}
