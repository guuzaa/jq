use std::fmt::Write;

use super::JsonPrinter;
use crate::json_parser::JSONValue;

// Printer for compact output
pub(super) struct CompactPrinter {
    pub(super) sort_keys: bool,
}

impl JsonPrinter for CompactPrinter {
    fn print_value(&self, value: &JSONValue, _indent: usize) -> String {
        let mut output = String::new();
        match value {
            JSONValue::Null => write!(output, "null").unwrap(),
            JSONValue::True => write!(output, "true").unwrap(),
            JSONValue::False => write!(output, "false").unwrap(),
            JSONValue::Number(n) => write!(output, "{}", n).unwrap(),
            JSONValue::String(s) => write!(output, "\"{}\"", s).unwrap(),
            JSONValue::Array(arr) => {
                if arr.is_empty() {
                    write!(output, "[]").unwrap();
                } else {
                    write!(output, "[").unwrap();

                    for (i, item) in arr.iter().enumerate() {
                        write!(output, "{}", self.print_value(item, _indent + 1)).unwrap();
                        if i < arr.len() - 1 {
                            write!(output, ",").unwrap();
                        }
                    }

                    write!(output, "]").unwrap();
                }
            }
            JSONValue::Object(obj) => {
                if obj.is_empty() {
                    write!(output, "{{}}").unwrap();
                } else {
                    write!(output, "{{").unwrap();

                    let keys = self.sort_object_keys(obj);

                    for (i, key) in keys.iter().enumerate() {
                        write!(output, "\"{}\":", key).unwrap();

                        if let Some(val) = obj.get(*key) {
                            write!(output, "{}", self.print_value(val, _indent + 1)).unwrap();
                        }

                        if i < keys.len() - 1 {
                            write!(output, ",").unwrap();
                        }
                    }

                    write!(output, "}}").unwrap();
                }
            }
        }
        output
    }

    fn sort_object_keys<'a>(
        &self,
        obj: &'a std::collections::HashMap<String, JSONValue>,
    ) -> Vec<&'a String> {
        if self.sort_keys {
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();
            keys
        } else {
            obj.keys().collect()
        }
    }
}
