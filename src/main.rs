use std::env;
use std::io::Write;
use std::path::PathBuf;

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
                println!("Query: {:#?}", query);
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
