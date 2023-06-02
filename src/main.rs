use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use prettytable::{Cell, Row, Table};
use sqlparser::ast::SetExpr;
use sqlparser::ast::Statement::{CreateTable, Query};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

fn main() {
    let mut data_base_path: PathBuf = env::current_dir().expect("Failed to get current directory");
    data_base_path.push("data");
    let mut line = String::new();
    let b1 = std::io::stdin().read_line(&mut line).unwrap();
    let sql = line.trim_end();
    println!("SQL , {}", line);
    println!("no of bytes read , {}", b1);

    let dialect = GenericDialect {}; // or AnsiDialect, or your own dialect ...

    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    for statement in ast {
        match statement {
            Query(query) => {
                let select_query = match *query.body {
                    // Assuming body is an Enum and one of its variant is Select
                    SetExpr::Select(select_struct) => select_struct,
                    _ => panic!("Expected a Select query"),
                };
                let table_name = match &select_query.from[0].relation {
                    sqlparser::ast::TableFactor::Table { name, .. } => name,
                    _ => panic!("Expected a table name"),
                };
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
                let header = match lines.next() {
                    Some(line) => line.unwrap(),
                    None => return,
                };

                let mut table = Table::new();
                table.add_row(Row::new(header.split(",").map(|s| Cell::new(s)).collect()));

                for line in lines {
                    let line = line.unwrap();
                    let row_data: Vec<Cell> = line.split(",").map(|s| Cell::new(s)).collect();
                    table.add_row(Row::new(row_data));
                }

                table.printstd();
            }
            CreateTable { name, columns, .. } => {
                println!("Create table: {} with columns:", name);
                for c in columns.clone() {
                    println!("\t{:?}", c);
                }
                // write a file with the table name
                let mut file_name = String::from(name.to_string());
                file_name.push_str(".txt");
                let mut file =
                    std::fs::File::create(data_base_path.clone().join(&file_name)).unwrap();
                let mut content = String::new();
                for (index, c) in columns.iter().enumerate() {
                    content.push_str(c.name.to_string().as_str());
                    if index != columns.len() - 1 {
                        content.push_str(",");
                    }
                }
                file.write_all(content.as_bytes()).unwrap();
            }
            _ => {
                println!("Not supported yet");
            }
        }
    }
}
