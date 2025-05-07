mod cli;
mod filter;
mod json_parser;
mod json_printer;
mod json_value_filter;

use cli::{parse_args, print_usage, InputSource};
use filter::parse_filter_expression;
use json_parser::parse;
use json_printer::{print_json_value, PrintOptions};
use json_value_filter::apply_filter_to_value;
use std::io::{self, Read, Write};

fn main() {
    // Parse command-line arguments
    let cli_options = match parse_args() {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("Error: {}", e);
            print_usage();
            std::process::exit(1);
        }
    };

    // Parse filter expression
    let filter_expression = match parse_filter_expression(&cli_options.filter_expression) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error parsing filter expression: {}", e);
            std::process::exit(1);
        }
    };

    // Get input JSON string
    let json_input_str = match cli_options.input_source {
        InputSource::File(file_path) => match std::fs::read_to_string(&file_path) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file_path, e);
                std::process::exit(1);
            }
        },
        InputSource::String(s) => s,
        InputSource::StdIn => {
            let mut buffer = String::new();
            if io::stdin().read_to_string(&mut buffer).is_ok() {
                buffer
            } else {
                eprintln!("Failed to read from stdin.");
                std::process::exit(1);
            }
        }
    };

    // Setup printing options based on command line flags
    let print_options = PrintOptions {
        compact: cli_options.compact_output,
        sort_keys: cli_options.sort_keys,
        use_tabs: cli_options.use_tabs,
    };

    // Parse and process JSON
    match parse(&json_input_str) {
        Ok(parsed_json) => match apply_filter_to_value(&parsed_json, &filter_expression) {
            Ok(filtered_value) => {
                print_json_value(&filtered_value, 0, &print_options);
                io::stdout().flush().expect("Failed to flush stdout");
            }
            Err(e) => {
                eprintln!("Error applying filter: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Failed to parse JSON: {:?}", e);
            std::process::exit(1);
        }
    }
}
