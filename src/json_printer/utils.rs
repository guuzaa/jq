use std::fmt::Write; // Import the Write trait

pub fn print_string(s: &str) -> String {
    let mut output = String::new();
    write!(output, "\"").unwrap();
    for c in s.chars() {
        match c {
            '"' => write!(output, r#"\""#).unwrap(),
            '\\' => write!(output, r#"\\"#).unwrap(),
            '\n' => write!(output, r#"\n"#).unwrap(),
            '\r' => write!(output, r#"\r"#).unwrap(),
            '\t' => write!(output, r#"\t"#).unwrap(),
            '\u{0008}' => write!(output, r#"\b"#).unwrap(),
            '\u{000C}' => write!(output, r#"\f"#).unwrap(),
            _ => write!(output, "{}", c).unwrap(),
        }
    }
    write!(output, "\"").unwrap();
    output
}
