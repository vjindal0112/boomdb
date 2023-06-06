use core::panic;
use csv::{Reader, WriterBuilder};
use std::collections::HashSet;

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use sqlparser::ast::{ColumnDef, Expr, Ident, ObjectName, Query, SelectItem, SetExpr};

use crate::common::Column;
use crate::utils::{check_table_name, make_printable_table};

fn parse_select_columns(
    existing_column_names: Vec<String>,
    projection: Vec<SelectItem>,
) -> Vec<Column> {
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

pub fn insert(
    table_name: ObjectName,
    columns: Vec<Ident>,
    values: Vec<Expr>,
    data_base_path: &PathBuf,
) -> () {
    let file_name = match check_table_name(table_name.to_string(), data_base_path) {
        Ok(file_path) => file_path,
        Err(_) => panic!("Table does not exist"),
    };
    let mut reader = Reader::from_path(file_name.clone()).unwrap();
    let headers = reader.headers().unwrap();
    let mut column_set: HashSet<String> = HashSet::new();
    for column in columns {
        column_set.insert(column.value);
    }
    for header in headers {
        if !column_set.contains(header) {
            panic!("Insert statement doesn't contain all necessary columns");
        }
    }
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name.clone())
        .unwrap();
    let mut writer = WriterBuilder::new()
        .has_headers(true) // Set to true if the file has headers
        .from_writer(file);
    let mut row: Vec<String> = vec![];
    for value in values {
        match value {
            Expr::Value(val) => row.push(val.to_string()), // TODO: check if to_string works here
            _ => panic!("Only values are supported in insert statement"),
        }
    }
    writer.write_record(row).unwrap();
    writer.flush().unwrap();
}

pub fn select(query: Query, data_base_path: &PathBuf) -> () {
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
    let file_path = match check_table_name(table_name.to_string(), data_base_path) {
        Ok(file_path) => file_path,
        Err(_) => panic!("Table does not exist"),
    };
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // header should always be there
    let raw_header_line = match lines.next() {
        Some(line) => line.unwrap(),
        None => return,
    };
    let raw_headers: Vec<String> = raw_header_line.split(",").map(String::from).collect();
    let columns = parse_select_columns(raw_headers.clone(), projection);
    match make_printable_table(raw_headers, lines, columns) {
        Ok(table) => table.printstd(),
        Err(_) => panic!("Error while printing table"),
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
