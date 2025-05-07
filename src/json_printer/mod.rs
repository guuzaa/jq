use crate::json_parser::JSONValue;

mod compact_printer;
mod pretty_printer;
mod utils;

use compact_printer::CompactPrinter;
use pretty_printer::PrettyPrinter;

#[derive(Default)]
pub struct PrintOptions {
    pub compact: bool,
    pub sort_keys: bool,
    pub use_tabs: bool,
}

pub fn print_json_value(value: &JSONValue, indent: usize, options: &PrintOptions) {
    // Factory method to get the right printer based on options
    let printer: Box<dyn JsonPrinter> = match options.compact {
        true => Box::new(CompactPrinter {
            sort_keys: options.sort_keys,
        }),
        false => Box::new(PrettyPrinter {
            sort_keys: options.sort_keys,
            use_tabs: options.use_tabs,
        }),
    };

    // Get the formatted string from the printer
    let output_string = printer.print_value(value, indent);
    // Print the string
    print!("{}", output_string);

    // For top-level values only, add a final newline
    if indent == 0 {
        println!();
    }
}

// Trait for JSON printers
trait JsonPrinter {
    fn print_value(&self, value: &JSONValue, indent: usize) -> String;
    fn sort_object_keys<'a>(
        &self,
        obj: &'a std::collections::HashMap<String, JSONValue>,
    ) -> Vec<&'a String>;
}
