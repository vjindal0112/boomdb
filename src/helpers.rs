use std::{collections::HashMap, env, path::PathBuf};

use csv::StringRecord;
use sqlparser::ast::{BinaryOperator, Expr, Value};

pub fn get_tmp_path(file_name: String) -> PathBuf {
    let mut tmp_dir = PathBuf::from(env::temp_dir());
    tmp_dir.push(file_name);
    tmp_dir
}

pub fn is_row_included(
    row: StringRecord,
    header_to_index: HashMap<String, i32>,
    logical_expression: Option<Expr>,
) -> bool {
    let result;
    match logical_expression {
        Some(expr) => {
            result = evaluate_binary_op(&row, header_to_index, expr);
        }
        None => {
            result = true;
        }
    }
    result
}

fn evaluate_binary_op(
    row: &StringRecord,
    header_to_index: HashMap<String, i32>,
    expr: Expr,
) -> bool {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            let left_expr = *left;
            let right_expr = *right;
            match (left_expr.clone(), right_expr.clone()) {
                (Expr::Identifier(ident), Expr::Value(value)) => {
                    let index = header_to_index.get(&ident.value).unwrap();
                    let row_value = row.get(*index as usize).unwrap();
                    let value_str = match value {
                        Value::Number(num, _) => num.to_string(),
                        Value::Boolean(bool) => bool.to_string(),
                        Value::SingleQuotedString(string)
                        | Value::EscapedStringLiteral(string)
                        | Value::SingleQuotedByteStringLiteral(string)
                        | Value::DoubleQuotedByteStringLiteral(string)
                        | Value::RawStringLiteral(string)
                        | Value::NationalStringLiteral(string)
                        | Value::HexStringLiteral(string)
                        | Value::DoubleQuotedString(string) => string.to_string(),
                        Value::Null => String::from(""),
                        _ => {
                            panic!("Not implemented")
                        }
                    };
                    match op {
                        BinaryOperator::Eq => {
                            return row_value == value_str;
                        }
                        BinaryOperator::Gt => {
                            return row_value.parse::<i32>().expect("Stored value not a number")
                                > value_str
                                    .parse::<i32>()
                                    .expect("Passed in value not a number");
                        }
                        BinaryOperator::Lt => {
                            return row_value.parse::<i32>().expect("Stored value not a number")
                                < value_str
                                    .parse::<i32>()
                                    .expect("Passed in value not a number");
                        }
                        BinaryOperator::GtEq => {
                            return row_value.parse::<i32>().expect("Stored value not a number")
                                >= value_str
                                    .parse::<i32>()
                                    .expect("Passed in value not a number");
                        }
                        BinaryOperator::LtEq => {
                            return row_value.parse::<i32>().expect("Stored value not a number")
                                <= value_str
                                    .parse::<i32>()
                                    .expect("Passed in value not a number");
                        }
                        _ => {
                            panic!("Not implemented")
                        }
                    }
                }
                (_, _) => {}
            }
            let left = evaluate_binary_op(&row, header_to_index.clone(), left_expr.clone());
            let right = evaluate_binary_op(&row, header_to_index.clone(), right_expr.clone());
            return match op {
                BinaryOperator::And => left && right,
                BinaryOperator::Or => left || right,
                _ => {
                    panic!("Not implemented")
                }
            };
        }
        Expr::Identifier(_ident) => {
            panic!("An identifier should not be passed in here")
        }
        _ => {
            panic!("Not implemented")
        }
    }
}
