pub mod commands;
pub mod common;
pub mod utils;

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

use sqlparser::ast::SetExpr;
use sqlparser::ast::Statement::{CreateTable, Delete, Insert, Query};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

fn main() {
    let mut data_base_path: PathBuf = env::current_dir().expect("Failed to get current directory");
    data_base_path.push("data");
    let mut line = String::new();
    loop {
        println!("");
        print!("> ");
        io::stdout().flush().unwrap();
        let b1 = std::io::stdin().read_line(&mut line).unwrap();
        let sql = line.trim_end();
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
                Insert {
                    table_name,
                    columns,
                    source,
                    ..
                } => {
                    match *source.body {
                        SetExpr::Values(values) => {
                            commands::insert(
                                table_name,
                                columns,
                                values.rows[0].clone(),
                                &data_base_path,
                            );
                        }
                        _ => {
                            panic!("There shouldn't be something other than values in an insert statement");
                        }
                    }
                }
                Delete {
                    tables,
                    from,
                    selection,
                    ..
                } => {
                    panic!("Delete supported yet");
                }
                _ => {
                    panic!("Not supported yet");
                }
            }
        }
    }
}
