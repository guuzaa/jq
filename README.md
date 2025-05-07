<br>
<p align="center">
<img src="https://objectstorageapi.us-west-1.run.claw.cloud/0i8f5lc3-assets/jq-rust.png" alt="jq" height="150" width="150">
</a>
</p>

<h1 align="center">jq ⚙️️</h1>
A lightweight, fast, and minimal jq-like command-line tool for filtering and formatting JSON data. This tool is written in Rust and supports basic field extraction, array indexing, and pretty/compact output formatting.

## Features

- **Filter JSON data** using simple expressions (e.g., `.field`, `.field[0]`, `.field.subfield`).
- **Pretty-print** or **compact** output.
- **Sort object keys** in output.
- **Tab or space indentation** for pretty output.
- **Read input** from files, stdin, or directly from a string.
- **Fast, zero-dependency JSON parser** implemented in Rust.

## Installation

Clone the repository and build with Cargo:

```sh
cargo build --release
```

The binary will be at `target/release/jq`.

## Usage

```
jq [options] [filter_expression] [json_input_string_or_filename]
jq [options] [json_input_string_or_filename]
```

If no JSON input is provided, it will try to read from stdin.

### Options

- `-c`               Compact instead of pretty-printed output
- `-S`               Sort keys of objects on output
- `--tab`            Use tabs for indentation (default is 2 spaces)
- `--help`           Display help and exit
- `--version`        Output version information and exit

### Filter Expression Syntax

- `.`                Identity filter (returns the whole input)
- `.field`           Extracts the value of `field` from an object
- `.field.subfield`  Extracts nested fields
- `.field[0]`        Extracts the first element of an array in `field`
- `.field[0].sub`    Combines array and field access
- `[index]`          Extracts the element at `index` from an array (e.g., `.[2]`)

If a field or index does not exist, `null` is returned.

### Examples

#### Pretty-print a JSON file
```sh
jq data.json
```

#### Extract a field
```sh
jq '.name' data.json
```

#### Extract a nested field
```sh
jq '.details.description' bar.json
```

#### Extract an array element
```sh
jq '.tags[1]' bar.json
```

#### Compact output
```sh
jq -c bar.json
```

#### Sort keys and use tabs for indentation
```sh
jq -S --tab bar.json
```

#### Read from stdin
```sh
cat foo.json | jq '.cmake'
```

#### Provide JSON as a string
```sh
jq '.cmake' '{"cmake": "/path/to/cmake"}'
```

## Limitations
- Only supports simple field and array access (no filters, maps, or advanced jq features).
- Returns `null` for missing fields or out-of-bounds indices.

## License
Licensed under CC0 1.0 Universal license, see [LICENSE](LICENSE) for details. 