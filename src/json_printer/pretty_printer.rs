use std::fmt::Write; // Import the Write trait

use super::JsonPrinter;
use crate::json_parser::JSONValue;

// Printer for pretty output
pub(super) struct PrettyPrinter {
    pub(super) sort_keys: bool,
    pub(super) use_tabs: bool,
}

impl JsonPrinter for PrettyPrinter {
    fn print_value(&self, value: &JSONValue, indent: usize) -> String {
        let mut output = String::new();
        let indent_char = if self.use_tabs { "\t" } else { "  " };
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
                    writeln!(output, "[").unwrap();

                    for (i, item) in arr.iter().enumerate() {
                        write!(output, "{}", indent_char.repeat(indent + 1)).unwrap();
                        // Recursive call now appends to the current output string
                        write!(output, "{}", self.print_value(item, indent + 1)).unwrap();
                        if i < arr.len() - 1 {
                            write!(output, ",").unwrap();
                        }
                        writeln!(output).unwrap();
                    }

                    write!(output, "{}]", indent_char.repeat(indent)).unwrap();
                }
            }
            JSONValue::Object(obj) => {
                if obj.is_empty() {
                    write!(output, "{{}}").unwrap();
                } else {
                    writeln!(output, "{{").unwrap();

                    let keys = self.sort_object_keys(obj);

                    for (i, key) in keys.iter().enumerate() {
                        write!(output, "{}\"{}\":", indent_char.repeat(indent + 1), key).unwrap();
                        write!(output, " ").unwrap(); // Add exactly one space after the colon

                        if let Some(val) = obj.get(*key) {
                            // Recursive call now appends to the current output string
                            write!(output, "{}", self.print_value(val, indent + 1)).unwrap();
                        }

                        if i < keys.len() - 1 {
                            write!(output, ",").unwrap();
                        }
                        writeln!(output).unwrap();
                    }

                    write!(output, "{}}}", indent_char.repeat(indent)).unwrap();
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
