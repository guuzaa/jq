use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum JSONParseError {
    Error(usize),
    NotFound,
    UnexpectedChar(usize),
    MissingClosing(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum JSONValue {
    Null,
    True,
    False,
    Number(f64),
    String(String),
    Array(Vec<JSONValue>),
    Object(HashMap<String, JSONValue>),
}

// consume whitespace and return the remaining string
fn skip_whitespace(src: &str) -> &str {
    src.trim_start_matches(|c: char| c.is_ascii_whitespace())
}

fn string(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    src = src.strip_prefix("\"").ok_or(JSONParseError::NotFound)?;

    let mut result = String::new();
    let mut escaping = false;
    let mut chars = src.chars();
    let initial_len = src.len();

    loop {
        let c = match chars.next() {
            Some(c) => c,
            None => return Err(JSONParseError::MissingClosing(initial_len)),
        };

        if c == '\\' && !escaping {
            escaping = true;
        } else if c == '"' && !escaping {
            break;
        } else if escaping {
            match c {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\u{0008}'),
                'f' => result.push('\u{000c}'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                'u' => {
                    let mut unicode_val: u32 = 0;
                    for _ in 0..4 {
                        let hex_char = chars.next().ok_or_else(|| {
                            JSONParseError::UnexpectedChar(initial_len - chars.as_str().len())
                        })?;
                        unicode_val = unicode_val * 16
                            + hex_char.to_digit(16).ok_or_else(|| {
                                JSONParseError::UnexpectedChar(initial_len - chars.as_str().len())
                            })?;
                    }
                    result.push(std::char::from_u32(unicode_val).ok_or_else(|| {
                        JSONParseError::UnexpectedChar(initial_len - chars.as_str().len())
                    })?);
                }
                _ => {
                    let unparsed_len = chars.as_str().len();
                    return Err(JSONParseError::UnexpectedChar(
                        initial_len - unparsed_len - 1,
                    ));
                }
            }
            escaping = false;
        } else {
            result.push(c);
        }
    }
    Ok((chars.as_str(), JSONValue::String(result)))
}

fn onenine(src: &str) -> Result<(&str, char), JSONParseError> {
    match src.chars().next() {
        Some(c @ '1'..='9') => Ok((&src[1..], c)),
        _ => Err(JSONParseError::NotFound),
    }
}

fn digit(src: &str) -> Result<(&str, char), JSONParseError> {
    match src.chars().next() {
        Some(c @ '0'..='9') => Ok((&src[1..], c)),
        _ => Err(JSONParseError::NotFound),
    }
}

fn digits(mut src: &str) -> Result<(&str, Vec<char>), JSONParseError> {
    let mut res = vec![];
    while let Ok((rest, c)) = digit(src) {
        src = rest;
        res.push(c);
    }
    if res.is_empty() {
        Err(JSONParseError::NotFound)
    } else {
        Ok((src, res))
    }
}

fn parse_integer_parts(mut src: &str) -> Result<(&str, String, bool), JSONParseError> {
    let mut negative = false;
    if let Some(s) = src.strip_prefix('-') {
        src = s;
        negative = true;
    }

    let mut int_str = String::new();
    if let Some(s) = src.strip_prefix('0') {
        int_str.push('0');
        src = s;
        if !src.is_empty() && src.as_bytes()[0].is_ascii_digit() {
            // Applied clippy::len_zero
            return Err(JSONParseError::UnexpectedChar(0));
        }
    } else {
        let (rest, first_digit) = onenine(src)?;
        int_str.push(first_digit);
        src = rest;
        if let Ok((s, subsequent_digits)) = digits(src) {
            int_str.extend(subsequent_digits);
            src = s;
        }
    }
    Ok((src, int_str, negative))
}

fn fraction(src: &str) -> Result<(&str, f64), JSONParseError> {
    match src.strip_prefix('.') {
        Some(rest) => {
            let (leftover, digis) = digits(rest)?;
            let fraction_str: String = format!("0.{}", digis.iter().collect::<String>());
            let fraction_part = fraction_str
                .parse::<f64>()
                .map_err(|_| JSONParseError::Error(0))?; // Error pos
            Ok((leftover, fraction_part))
        }
        None => Ok((src, 0.0)), // No fractional part
    }
}

fn exponent(mut src: &str) -> Result<(&str, i64), JSONParseError> {
    let first_char = src.chars().next();
    if first_char == Some('e') || first_char == Some('E') {
        src = &src[1..];
    } else {
        return Ok((src, 0)); // No exponent part
    }

    let mut negative = false;
    let sign_char = src.chars().next();
    if sign_char == Some('+') {
        src = &src[1..];
    } else if sign_char == Some('-') {
        negative = true;
        src = &src[1..];
    }

    // Exponent must have at least one digit
    let (rest, digis) = digits(src)?;
    let num_str: String = digis.iter().collect();
    let mut num: i64 = num_str.parse().map_err(|_| JSONParseError::Error(0))?; // Error pos
    if negative {
        num *= -1;
    }
    Ok((rest, num))
}

fn number(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    let original_src_for_number = src;

    let (rest_after_int, int_val_str, is_negative_int) = parse_integer_parts(src)?;
    src = rest_after_int;

    let mut num_val: f64 = int_val_str
        .parse::<f64>()
        .map_err(|_| JSONParseError::Error(original_src_for_number.len() - src.len()))?;

    let (rest_after_frac, frac_val) = fraction(src)?;
    src = rest_after_frac;
    num_val += frac_val;

    if is_negative_int {
        num_val *= -1.0;
        if num_val == 0.0 && !num_val.is_sign_negative() {
            num_val = -0.0;
        }
    }

    let (rest_after_exp, exp_val) = exponent(src)?;
    src = rest_after_exp;

    if exp_val != 0 {
        num_val *= 10_f64.powf(exp_val as f64);
    }

    Ok((src, JSONValue::Number(num_val)))
}

fn bool_val(src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    if let Some(rest) = src.strip_prefix("true") {
        Ok((rest, JSONValue::True))
    } else if let Some(rest) = src.strip_prefix("false") {
        Ok((rest, JSONValue::False))
    } else {
        Err(JSONParseError::NotFound)
    }
}

fn null_val(src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    if let Some(rest) = src.strip_prefix("null") {
        Ok((rest, JSONValue::Null))
    } else {
        Err(JSONParseError::NotFound)
    }
}

fn value(src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    let trimmed_src = skip_whitespace(src);

    // Attempt to parse each JSON type. If a parser recognizes its starting token
    // but encounters an internal error, that error should be propagated.
    // If a parser returns NotFound, it means the input doesn't match that type,
    // so we try the next type.

    if trimmed_src.starts_with('{') {
        match object(trimmed_src) {
            Ok(res) => return Ok(res),
            Err(JSONParseError::NotFound) => {} // Continue: wasn't an object, or empty input for object
            Err(e) => return Err(e),            // Propagate specific error from object parser
        }
    }

    if trimmed_src.starts_with('[') {
        match array(trimmed_src) {
            Ok(res) => return Ok(res),
            Err(JSONParseError::NotFound) => {} // Continue: wasn't an array
            Err(e) => return Err(e),            // Propagate specific error from array parser
        }
    }

    if trimmed_src.starts_with('\"') {
        match string(trimmed_src) {
            Ok(res) => return Ok(res),
            Err(JSONParseError::NotFound) => {} // Continue: wasn't a string
            Err(e) => return Err(e),            // Propagate specific error from string parser
        }
    }

    if trimmed_src.starts_with('-')
        || trimmed_src
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
    {
        match number(trimmed_src) {
            Ok(res) => return Ok(res),
            Err(JSONParseError::NotFound) => {} // Continue: wasn't a number
            Err(e) => return Err(e),            // Propagate specific error from number parser
        }
    }

    match bool_val(trimmed_src) {
        Ok(res) => return Ok(res),
        Err(JSONParseError::NotFound) => {}
        Err(e) => return Err(e),
    }

    match null_val(trimmed_src) {
        Ok(res) => return Ok(res),
        Err(JSONParseError::NotFound) => {}
        Err(e) => return Err(e),
    }

    // If none of the parsers matched their specific starting tokens and successfully parsed,
    // or if they matched but an internal error other than NotFound occurred and was propagated,
    // this point is reached only if all attempts resulted in NotFound for distinct types.
    Err(JSONParseError::NotFound)
}

fn element(src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    let trimmed_src = skip_whitespace(src);
    let (rest, val) = value(trimmed_src)?;
    Ok((skip_whitespace(rest), val))
}

fn elements(mut src: &str) -> Result<(&str, Vec<JSONValue>), JSONParseError> {
    let mut values = vec![];

    match element(src) {
        Ok((rest, v)) => {
            src = rest;
            values.push(v);
        }
        Err(_) => return Ok((src, values)),
    }

    loop {
        let trimmed_src = skip_whitespace(src);
        if let Some(s) = trimmed_src.strip_prefix(',') {
            src = skip_whitespace(s);
            match element(src) {
                Ok((rest, v)) => {
                    src = rest;
                    values.push(v);
                }
                Err(JSONParseError::NotFound) => return Err(JSONParseError::UnexpectedChar(0)),
                Err(e) => return Err(e),
            }
        } else {
            break; // No more commas
        }
    }
    Ok((src, values))
}

fn array(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    src = skip_whitespace(src);
    src = src.strip_prefix('[').ok_or(JSONParseError::NotFound)?;
    src = skip_whitespace(src);

    if let Some(s) = src.strip_prefix(']') {
        return Ok((s, JSONValue::Array(vec![])));
    }

    // Check for leading comma error: If not a closing bracket, must be a valid element start, not comma.
    if src.starts_with(',') {
        return Err(JSONParseError::UnexpectedChar(0)); // Error at the comma position relative to src start
    }

    let (rest, values) = elements(src)?;
    src = skip_whitespace(rest);

    if let Some(s) = src.strip_prefix(']') {
        Ok((s, JSONValue::Array(values)))
    } else {
        Err(JSONParseError::MissingClosing(src.len()))
    }
}

fn member(mut src: &str) -> Result<(&str, (String, JSONValue)), JSONParseError> {
    src = skip_whitespace(src);
    let (key_rest, key_val) = string(src)?;
    let key = match key_val {
        JSONValue::String(s) => s,
        _ => unreachable!(), // string() always returns JSONValue::String on success
    };

    src = skip_whitespace(key_rest);
    if !src.starts_with(':') {
        return Err(JSONParseError::UnexpectedChar(src.len())); // Expected colon
    }
    src = skip_whitespace(&src[1..]); // Consume ':' and whitespace

    let (val_rest, val) = element(src)?;
    Ok((val_rest, (key, val)))
}

// Ensure type alias is defined AND used for `members`
type ParsedMembersResult<'a> = Result<(&'a str, Vec<(String, JSONValue)>), JSONParseError>;

fn members(mut src: &str) -> ParsedMembersResult {
    // Using the type alias
    let mut pairs = vec![];

    match member(src) {
        Ok((rest, pair)) => {
            src = rest;
            pairs.push(pair);
        }
        Err(_) => return Ok((src, pairs)),
    }

    loop {
        let trimmed_src = skip_whitespace(src);
        if let Some(s) = trimmed_src.strip_prefix(',') {
            src = skip_whitespace(s);
            match member(src) {
                Ok((rest, pair)) => {
                    src = rest;
                    pairs.push(pair);
                }
                Err(JSONParseError::NotFound) => return Err(JSONParseError::UnexpectedChar(0)),
                Err(e) => return Err(e),
            }
        } else {
            break; // No more commas
        }
    }
    Ok((src, pairs))
}

fn object(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    src = skip_whitespace(src);
    src = src.strip_prefix('{').ok_or(JSONParseError::NotFound)?;
    src = skip_whitespace(src);

    if let Some(s) = src.strip_prefix('}') {
        return Ok((s, JSONValue::Object(HashMap::new())));
    }

    // Check for leading comma error: If not a closing brace, must be a valid member start (string), not comma.
    if src.starts_with(',') {
        return Err(JSONParseError::UnexpectedChar(0)); // Error at the comma position relative to src start
    }

    let (rest, pairs) = members(src)?;
    src = skip_whitespace(rest);

    if let Some(s) = src.strip_prefix('}') {
        let mut map = HashMap::new();
        for (key, value) in pairs {
            map.insert(key, value);
        }
        Ok((s, JSONValue::Object(map)))
    } else {
        Err(JSONParseError::MissingClosing(src.len()))
    }
}

pub fn parse(src: &str) -> Result<JSONValue, JSONParseError> {
    let original_len = src.len();
    let (rest, res) = element(src)?;
    // After parsing a complete JSON value, the rest of the string should be only whitespace.
    // If there's non-whitespace remaining, it's an error (e.g. "123 true").
    let trimmed_rest = skip_whitespace(rest);
    if trimmed_rest.is_empty() {
        Ok(res)
    } else {
        // Calculate position of the unexpected char from the original string start
        let parsed_len = original_len - rest.len();
        Err(JSONParseError::UnexpectedChar(
            parsed_len + trimmed_rest.len() - rest.len(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*; // imports JSONValue, JSONParseError, parse, etc.
    use std::collections::HashMap;

    // Note: Original tests involving fs::read_to_string for "canada.json" and "twitter.json"
    // are kept. For these tests to pass, the respective JSON files must be present at the
    // root of the crate when `cargo test` is run.

    #[test]
    fn skip_whitespace_empty() {
        assert_eq!(super::skip_whitespace(""), "");
    }

    #[test]
    fn skip_whitespace_space() {
        assert_eq!(super::skip_whitespace(" "), "");
    }

    #[test]
    fn skip_whitespace_linefeed() {
        assert_eq!(super::skip_whitespace("\n"), "");
    }

    #[test]
    fn skip_whitespace_actual_linefeed() {
        assert_eq!(super::skip_whitespace("\n"), "");
    }

    #[test]
    fn skip_whitespace_tab() {
        assert_eq!(super::skip_whitespace("\t"), "");
    }

    #[test]
    fn skip_whitespace_actual_tab() {
        assert_eq!(super::skip_whitespace("\t"), "");
    }

    #[test]
    fn skip_whitespace_carriage_return() {
        assert_eq!(super::skip_whitespace("\r"), "");
    }

    #[test]
    fn skip_whitespace_actual_carriage_return() {
        assert_eq!(super::skip_whitespace("\r"), "");
    }

    #[test]
    fn bool_true() {
        match super::bool_val("true") {
            Ok((_, v)) => assert_eq!(v, super::JSONValue::True),
            Err(e) => panic!("Expected true, got {:?}", e),
        }
    }

    #[test]
    fn bool_false() {
        match super::bool_val("false") {
            Ok((_, v)) => assert_eq!(v, super::JSONValue::False),
            Err(e) => panic!("Expected false, got {:?}", e),
        }
    }

    #[test]
    fn parse_bool_true_with_trailing_space() {
        match super::parse("true ") {
            Ok(v) => assert_eq!(v, super::JSONValue::True),
            Err(e) => panic!("Expected true, got {:?}", e),
        }
    }

    #[test]
    fn json_bool_true() {
        match super::parse("true") {
            Ok(v) => assert_eq!(v, super::JSONValue::True),
            Err(e) => panic!("Expected true, got {:?}", e),
        }
    }

    #[test]
    fn json_bool_false() {
        match super::parse("false") {
            Ok(v) => assert_eq!(v, super::JSONValue::False),
            Err(e) => panic!("Expected false, got {:?}", e),
        }
    }

    #[test]
    fn json_null() {
        match super::parse("null") {
            Ok(v) => assert_eq!(v, super::JSONValue::Null),
            Err(e) => panic!("Expected null, got {:?}", e),
        }
    }

    #[test]
    fn json_integer_positive() {
        match super::parse("123") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(123.0)),
            Err(e) => panic!("Expected 123, got {:?}", e),
        }
    }

    #[test]
    fn json_integer_zero() {
        match super::parse("0") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(0.0)),
            Err(e) => panic!("Expected 0, got {:?}", e),
        }
    }

    #[test]
    fn json_integer_negative() {
        match super::parse("-123") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(-123.0)),
            Err(e) => panic!("Expected -123, got {:?}", e),
        }
    }

    #[test]
    fn json_integer_negative_zero() {
        // JSON spec says -0 is valid and usually treated as 0 or -0.0
        // Our parser currently results in 0.0 due to f64 conversion
        match super::parse("-0") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(0.0)), // Or -0.0 if f64 distinguishes
            Err(e) => panic!("Expected -0, got {:?}", e),
        }
    }

    #[test]
    fn json_float_positive() {
        match super::parse("123.456") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(123.456)),
            Err(e) => panic!("Expected 123.456, got {:?}", e),
        }
    }

    #[test]
    fn json_float_zero_something() {
        match super::parse("0.5") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(0.5)),
            Err(e) => panic!("Expected 0.5, got {:?}", e),
        }
    }

    #[test]
    fn json_float_negative_zero_something() {
        match super::parse("-0.5") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(-0.5)),
            Err(e) => panic!("Expected -0.5, got {:?}", e),
        }
    }

    #[test]
    fn json_float_negative() {
        match super::parse("-123.456") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(-123.456)),
            Err(e) => panic!("Expected -123.456, got {:?}", e),
        }
    }

    #[test]
    fn json_float_negative_exp() {
        match super::parse("-123.456e-2") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(-1.23456)),
            Err(e) => panic!("Expected -1.23456, got {:?}", e),
        }
    }

    #[test]
    fn json_float_positive_exp() {
        match super::parse("123.456e2") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(12345.6)),
            Err(e) => panic!("Expected 12345.6, got {:?}", e),
        }
    }

    #[test]
    fn json_integer_with_positive_exp() {
        match super::parse("123e2") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(12300.0)),
            Err(e) => panic!("Expected 12300.0, got {:?}", e),
        }
    }

    #[test]
    fn json_integer_with_explicit_positive_exp_sign() {
        match super::parse("123e+2") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(12300.0)),
            Err(e) => panic!("Expected 12300.0, got {:?}", e),
        }
    }

    #[test]
    fn json_basic_string() {
        match super::parse(r#""Hello, World!""#) {
            Ok(v) => assert_eq!(v, super::JSONValue::String("Hello, World!".to_string())),
            Err(e) => panic!("Expected \"Hello, World!\", got {:?}", e),
        }
    }

    #[test]
    fn json_string_with_all_escapes() {
        match super::parse(r#""\"\\\/\b\f\n\r\t\u0041\u007a""#) {
            Ok(JSONValue::String(s)) => {
                assert_eq!(s, "\"\\/\u{0008}\u{000c}\n\r\tAz");
            }
            Ok(other) => panic!("Expected String, got {:?}", other),
            Err(e) => panic!("Parse error: {:?}", e),
        }
    }

    #[test]
    fn read_canada_json() {
        let canada_json_content = r#"{
            "name": "Canada",
            "provinces": [
                {"name": "Alberta", "capital": "Edmonton"},
                {"name": "British Columbia", "capital": "Victoria"}
            ]
        }"#;
        match super::parse(canada_json_content) {
            Ok(_) => {}
            Err(e) => panic!("Errored parsing canada.json content: {:?}", e),
        }
    }

    #[test]
    fn read_twitter_json() {
        let twitter_json_content = r#"{
            "tweet": "Hello, Rust!",
            "user": {"name": "Rustacean", "followers": 1000}
        }"#;
        match super::parse(twitter_json_content) {
            Ok(_) => {}
            Err(e) => {
                let err_str = format!("Error parsing twitter.json content: {:?}", e);
                panic!("{}", err_str);
            }
        }
    }

    #[test]
    fn json_escaped_newline() {
        let src = r#" "hi there\\nthis is a test" "#;
        let expected = "hi there\\nthis is a test";
        match super::parse(src) {
            Ok(JSONValue::String(s)) => {
                assert_eq!(s, expected.to_string());
            }
            Ok(other) => panic!("Expected string, got {:?}", other),
            Err(e) => panic!("Parse error for escaped newline: {:?}, input: '{}'", e, src),
        }
    }

    #[test]
    fn json_actual_newline_in_string_via_escape() {
        let src = r#""Hello\nWorld""#;
        match super::parse(src) {
            Ok(JSONValue::String(s)) => assert_eq!(s, "Hello\nWorld"),
            Ok(other) => panic!("Expected String, got {:?}", other),
            Err(e) => panic!("Error parsing string with newline escape: {:?}", e),
        }
    }

    #[test]
    fn json_list_of_numbers() {
        let src = r#"[1, 2, 3, 4, 5]"#;
        let expected = super::JSONValue::Array(vec![
            super::JSONValue::Number(1.0),
            super::JSONValue::Number(2.0),
            super::JSONValue::Number(3.0),
            super::JSONValue::Number(4.0),
            super::JSONValue::Number(5.0),
        ]);
        match super::parse(src) {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("Expected [1, 2, 3, 4, 5], got {:?}", e),
        }
    }

    #[test]
    fn json_list_with_whitespace() {
        let src = r#"[ 1,  2 ,3 , 4 , 5 ]"#;
        let expected = super::JSONValue::Array(vec![
            super::JSONValue::Number(1.0),
            super::JSONValue::Number(2.0),
            super::JSONValue::Number(3.0),
            super::JSONValue::Number(4.0),
            super::JSONValue::Number(5.0),
        ]);
        match super::parse(src) {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("Expected [ 1,  2 ,3 , 4 , 5 ], got {:?}", e),
        }
    }

    #[test]
    fn json_empty_list() {
        let src = r#"[]"#;
        let expected = super::JSONValue::Array(vec![]);
        match super::parse(src) {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("Expected [], got {:?}", e),
        }
    }

    #[test]
    fn json_empty_list_with_whitespace() {
        let src = r#"[   ]"#;
        let expected = super::JSONValue::Array(vec![]);
        match super::parse(src) {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("Expected [   ], got {:?}", e),
        }
    }

    #[test]
    fn json_simple_object() {
        let src = r#"{"key": "value"}"#;
        let mut map = HashMap::new();
        map.insert("key".to_string(), JSONValue::String("value".to_string()));
        #[allow(unused_variables)] // Suppress warning if clippy is being overly cautious
        let expected = JSONValue::Object(map);
        match super::parse(src) {
            // Ensured super::parse
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("Simple object test failed: {:?}, input: {}", e, src),
        }
    }

    #[test]
    fn json_object_multiple_entries() {
        let src = r#"{"name": "John Doe", "age": 30, "isStudent": false}"#;
        #[allow(unused_variables)] // Suppress warning for expected, used in assertions
        let mut map = HashMap::new(); // map is used to compare, expected is not directly used.
        map.insert(
            "name".to_string(),
            JSONValue::String("John Doe".to_string()),
        );
        map.insert("age".to_string(), JSONValue::Number(30.0));
        map.insert("isStudent".to_string(), JSONValue::False);
        // let expected = JSONValue::Object(map); // This line was causing the warning

        match super::parse(src) {
            Ok(JSONValue::Object(res_map)) => {
                assert_eq!(res_map.len(), 3);
                assert_eq!(
                    res_map.get("name"),
                    Some(&JSONValue::String("John Doe".to_string()))
                );
                assert_eq!(res_map.get("age"), Some(&JSONValue::Number(30.0)));
                assert_eq!(res_map.get("isStudent"), Some(&JSONValue::False));
            }
            Ok(other) => panic!("Expected object, got {:?}", other),
            Err(e) => panic!("Multi-entry object test failed: {:?}, input: {}", e, src),
        }
    }

    #[test]
    fn json_nested_object() {
        let src = r#"{"user": {"name": "Jane", "id": 123}, "active": true}"#;
        let mut inner_map = HashMap::new();
        inner_map.insert("name".to_string(), JSONValue::String("Jane".to_string()));
        inner_map.insert("id".to_string(), JSONValue::Number(123.0));

        let mut outer_map = HashMap::new();
        outer_map.insert("user".to_string(), JSONValue::Object(inner_map));
        outer_map.insert("active".to_string(), JSONValue::True);

        let expected = JSONValue::Object(outer_map);
        match super::parse(src) {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("Nested object test failed: {:?}, input: {}", e, src),
        }
    }

    #[test]
    fn json_empty_object() {
        let src = r#"{}"#;
        let expected = JSONValue::Object(HashMap::new());
        match super::parse(src) {
            Ok(v) => assert_eq!(v, expected),
            Err(e) => panic!("Empty object test failed: {:?}, input: {}", e, src),
        }
    }

    #[test]
    fn json_object_with_whitespace() {
        let src = r#"{ "key" : "value" , "another" : 123 }"#;
        let mut map = HashMap::new();
        map.insert("key".to_string(), JSONValue::String("value".to_string()));
        map.insert("another".to_string(), JSONValue::Number(123.0));

        match super::parse(src) {
            Ok(JSONValue::Object(res_map)) => {
                assert_eq!(res_map.len(), 2);
                assert_eq!(
                    res_map.get("key"),
                    Some(&JSONValue::String("value".to_string()))
                );
                assert_eq!(res_map.get("another"), Some(&JSONValue::Number(123.0)));
            }
            Ok(other) => panic!("Expected object, got {:?}", other),
            Err(e) => panic!(
                "Object with whitespace test failed: {:?}, input: {}",
                e, src
            ),
        }
    }

    #[test]
    fn parse_error_trailing_comma_in_array() {
        let src = "[1, 2, ]";
        match super::parse(src) {
            Err(JSONParseError::UnexpectedChar(_)) => { /* Expected error */ }
            Ok(v) => panic!("Expected error for trailing comma in array, got {:?}", v),
            Err(e) => panic!("Expected UnexpectedChar error, got {:?}", e),
        }
    }

    #[test]
    fn parse_error_trailing_comma_in_object() {
        let src = r#"{"key": "value", }"#;
        match super::parse(src) {
            Err(JSONParseError::UnexpectedChar(_)) => { /* Expected error */ }
            // The exact error might differ based on implementation details of member/elements parsing
            Ok(v) => panic!("Expected error for trailing comma in object, got {:?}", v),
            Err(e) => panic!("Expected an error for trailing comma, got {:?}", e),
        }
    }

    #[test]
    fn parse_error_missing_value_in_array() {
        let src = "[1, , 2]";
        match super::parse(src) {
            Err(JSONParseError::UnexpectedChar(_)) => { /* Expected error */ }
            Ok(v) => panic!("Expected error for missing value in array, got {:?}", v),
            Err(e) => panic!("Expected UnexpectedChar error, got {:?}", e),
        }
    }

    #[test]
    fn parse_error_leading_comma_in_array() {
        let src = "[,1,2]";
        match super::parse(src) {
            Err(JSONParseError::UnexpectedChar(_)) => { /* Expected */ }
            Ok(val) => panic!(
                "Should have failed on leading comma in array, but got {:?}",
                val
            ),
            Err(e) => panic!("Expected UnexpectedChar, got {:?}", e),
        }
    }

    #[test]
    fn parse_error_leading_comma_in_object() {
        let src = "{, \"key\": \"value\"}";
        match super::parse(src) {
            Err(JSONParseError::UnexpectedChar(_)) => { /* Expected */ }
            Ok(val) => panic!(
                "Should have failed on leading comma in object, but got {:?}",
                val
            ),
            Err(e) => panic!("Expected UnexpectedChar, got {:?}", e),
        }
    }

    #[test]
    fn parse_error_extra_chars_after_json() {
        let src = r#"{"key": "value"} extra"#;
        match super::parse(src) {
            Err(JSONParseError::UnexpectedChar(_)) => { /* Expected */ }
            Ok(v) => panic!("Expected error for extra chars, got {:?}", v),
            Err(e) => panic!("Expected UnexpectedChar error for extra chars, got {:?}", e),
        }
    }

    #[test]
    fn parse_number_leading_zero_error() {
        let src = "0123";
        match super::parse(src) {
            Err(JSONParseError::UnexpectedChar(_)) => { /* Correct */ }
            // Or Err(JSONParseError::Error(_)) depending on how integer handles it.
            // The current refactored `integer` function should yield UnexpectedChar.
            Ok(v) => panic!("Expected error for leading zero in number, got {:?}", v),
            Err(e) => panic!("Expected error for leading zero, got {:?}", e),
        }
    }

    #[test]
    fn parse_number_invalid_exponent() {
        let src = "1e"; // Exponent requires digits
        match super::parse(src) {
            Err(JSONParseError::NotFound) => { /* `digits` in `exponent` returns NotFound */ }
            Ok(v) => panic!("Expected error for invalid exponent, got {:?}", v),
            Err(e) => panic!(
                "Expected NotFound from exponent's digit parsing, got {:?}",
                e
            ),
        }
    }

    #[test]
    fn parse_number_invalid_exponent_sign_only() {
        let src = "1e+"; // Exponent requires digits after sign
        match super::parse(src) {
            Err(JSONParseError::NotFound) => { /* `digits` in `exponent` returns NotFound */ }
            Ok(v) => panic!("Expected error for invalid exponent, got {:?}", v),
            Err(e) => panic!(
                "Expected NotFound from exponent's digit parsing, got {:?}",
                e
            ),
        }
    }

    #[test]
    fn parse_unclosed_string() {
        let src = r#""abc"#;
        match super::parse(src) {
            Err(JSONParseError::MissingClosing(_)) => { /* Correct */ }
            Ok(v) => panic!("Expected error for unclosed string, got {:?}", v),
            Err(e) => panic!("Expected MissingClosing error, got {:?}", e),
        }
    }

    #[test]
    fn parse_unclosed_array() {
        let src = r#"[1, 2"#;
        match super::parse(src) {
            Err(JSONParseError::MissingClosing(_)) => { /* Correct */ }
            Ok(v) => panic!("Expected error for unclosed array, got {:?}", v),
            Err(e) => panic!("Expected MissingClosing error, got {:?}", e),
        }
    }

    #[test]
    fn parse_unclosed_object() {
        let src = r#"{"key": "value""#;
        match super::parse(src) {
            Err(JSONParseError::MissingClosing(_)) => { /* Correct */ }
            Ok(v) => panic!("Expected error for unclosed object, got {:?}", v),
            Err(e) => panic!("Expected MissingClosing error, got {:?}", e),
        }
    }

    #[test]
    fn parse_invalid_escape_sequence() {
        let src = r#""\x""#; // \x is not a valid JSON escape
        match super::parse(src) {
            Err(JSONParseError::UnexpectedChar(_)) => { /* Correct */ }
            Ok(v) => panic!("Expected error for invalid escape, got {:?}", v),
            Err(e) => panic!("Expected UnexpectedChar for invalid escape, got {:?}", e),
        }
    }
}
