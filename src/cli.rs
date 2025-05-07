pub struct CliOptions {
    pub filter_expression: String,
    pub input_source: InputSource,
    pub compact_output: bool,
    pub sort_keys: bool,
    pub use_tabs: bool,
}

pub enum InputSource {
    StdIn,
    String(String),
    File(String),
}

pub fn parse_args() -> Result<CliOptions, String> {
    let args: Vec<String> = std::env::args().collect();

    let mut compact_output = false;
    let mut sort_keys = false;
    let mut use_tabs = false;
    let mut filter_idx = 1;

    if args.len() < 2 {
        return Ok(CliOptions {
            filter_expression: ".".to_string(),
            input_source: InputSource::StdIn,
            compact_output,
            sort_keys,
            use_tabs,
        });
    }

    // Parse options
    while filter_idx < args.len() && args[filter_idx].starts_with('-') {
        match args[filter_idx].as_str() {
            arg if arg.starts_with("--") => match arg {
                "--tab" => use_tabs = true,
                "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--version" => {
                    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                _ => return Err(format!("Unknown option: {}", args[filter_idx])),
            },
            arg if arg.starts_with('-') => {
                // Handle combined short options (e.g. -cS)
                for c in arg[1..].chars() {
                    match c {
                        'c' => compact_output = true,
                        'S' => sort_keys = true,
                        _ => return Err(format!("Unknown option: -{}", c)),
                    }
                }
            }
            _ => return Err(format!("Unknown option: {}", args[filter_idx])),
        }
        filter_idx += 1;
    }

    // If we've consumed all args with options, use stdin with default filter
    if filter_idx >= args.len() {
        return Ok(CliOptions {
            filter_expression: ".".to_string(),
            input_source: InputSource::StdIn,
            compact_output,
            sort_keys,
            use_tabs,
        });
    }

    // Get the filter expression
    let filter = args[filter_idx].clone();
    filter_idx += 1;

    // Try to parse the filter expression
    match crate::filter::parse_filter_expression(&filter) {
        Ok(_) => {
            // This is a valid filter expression
            let input_source = if filter_idx < args.len() {
                let input_arg = &args[filter_idx];
                match std::fs::read_to_string(input_arg) {
                    Ok(_) => InputSource::File(input_arg.clone()),
                    Err(_) => InputSource::String(input_arg.clone()),
                }
            } else {
                InputSource::StdIn
            };
            Ok(CliOptions {
                filter_expression: filter,
                input_source,
                compact_output,
                sort_keys,
                use_tabs,
            })
        }
        Err(e) => {
            // Not a valid filter expression and not a file, must be an error
            Err(format!("Error parsing filter expression: {}", e))
        }
    }
}

pub fn print_usage() {
    eprintln!("Usage: jq [options] [filter_expression] [json_input_string_or_filename]");
    eprintln!("       jq [options] [json_input_string_or_filename]");
    eprintln!("Options:");
    eprintln!("  -c               compact instead of pretty-printed output");
    eprintln!("  -S               sort keys of objects on output");
    eprintln!("  --tab            use tabs for indentation");
    eprintln!("  --help           display this help and exit");
    eprintln!("  --version        output version information and exit");
    eprintln!("Examples:");
    eprintln!("  jq data.json                     # Print data.json with default filter");
    eprintln!("  jq \'.name\' data.json           # Get name field from data.json");
    eprintln!("  jq -c data.json                  # Print data.json in compact form");
    eprintln!("  jq -S data.json                  # Print data.json with sorted keys");
    eprintln!("If no JSON input is provided, it will try to read from stdin.");
}
