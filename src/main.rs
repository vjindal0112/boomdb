pub mod commands;
pub mod common;

use std::env;
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
                commands::select(*query, &data_base_path);
            }
            CreateTable { name, columns, .. } => {
                commands::create_table(name, columns, &data_base_path);
            }
            _ => {
                println!("Not supported yet");
            }
        }
    }
}
