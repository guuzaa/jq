use std::collections::HashMap;

#[allow(unused)]
use crate::json::Value;

pub enum ParseError {
    Error(usize),
    NotFound,
    UnexpectedChar(usize),
    MissingClosing(usize),
}

pub struct Token<'a> {
    rest: &'a str,
    value: Value,
}

impl<'a> Token<'a> {
    fn new(rest: &'a str, value: Value) -> Self {
        Self { rest, value }
    }
}

pub fn null(src: &str) -> Result<Token, ParseError> {
    match src.strip_prefix("null") {
        Some(rest) => Ok(Token::new(rest, Value::Null)),
        None => Err(ParseError::NotFound),
    }
}

pub fn bool(src: &str) -> Result<Token, ParseError> {
    match src.strip_prefix("true") {
        Some(rest) => Ok(Token::new(rest, Value::True)),
        None => match src.strip_prefix("false") {
            Some(rest) => Ok(Token::new(rest, Value::False)),
            None => Err(ParseError::NotFound),
        },
    }
}

pub fn string(mut src: &str) -> Result<Token, ParseError> {
    match src.strip_prefix("\"") {
        Some(rest) => src = rest,
        None => return Err(ParseError::NotFound),
    }

    let mut result = "".to_string();
    let mut escaping = false;
    let mut chars = src.chars();
    loop {
        let c = match chars.next() {
            Some(c) => c,
            None => return Err(ParseError::MissingClosing(src.len())),
        };

        if c == '\\' && !escaping {
            escaping = true;
        } else if c == '"' && !escaping {
            break;
        } else if escaping {
            match c {
                '\"' | '\\' | '/' | 'b' | 'n' | 'r' | 't' | 'u' => result.push(c),
                _ => return Err(ParseError::UnexpectedChar(chars.count())),
            }
            escaping = false;
        } else {
            result.push(c);
        }
    }
    Ok(Token::new(chars.as_str(), Value::String(result)))
}

fn integer(mut src: &str) -> Result<(&str, i64), ParseError> {
    let negative = match src.strip_prefix('-') {
        Some(rest) => {
            src = rest;
            true
        }
        None => false,
    };

    match non_zero(src) {
        Ok((rest, c)) => match digits(rest) {
            Ok((leftover, mut dgts)) => {
                dgts.insert(0, c);
                let int_str: String = dgts.iter().collect();
                let mut resulting_int = int_str.parse::<i64>().unwrap();
                if negative {
                    resulting_int *= -1;
                }
                return Ok((leftover, resulting_int));
            }
            Err(_) => {}
        },
        Err(_) => {}
    }

    match digit(src) {
        Ok((rest, c)) => {
            let mut n: i64 = c.to_digit(10).unwrap().into();
            if negative {
                n *= -1;
            }
            Ok((rest, n))
        }
        Err(e) => Err(e),
    }
}

fn non_zero(src: &str) -> Result<(&str, char), ParseError> {
    match src.chars().next() {
        Some('0') => Err(ParseError::NotFound),
        Some(c) if c.is_digit(10) => Ok((&src[1..], c)),
        _ => Err(ParseError::NotFound),
    }
}

fn digit(src: &str) -> Result<(&str, char), ParseError> {
    match src.chars().next() {
        Some('0') => Ok((&src[1..], '0')),
        Some(_) => non_zero(src),
        None => Err(ParseError::NotFound),
    }
}

fn digits(mut src: &str) -> Result<(&str, Vec<char>), ParseError> {
    let mut ret = vec![];
    loop {
        match digit(src) {
            Ok((rest, c)) => {
                src = rest;
                ret.push(c);
            }
            Err(_) => break,
        }
    }

    if ret.is_empty() {
        return Err(ParseError::NotFound);
    }
    Ok((src, ret))
}

fn fraction(src: &str) -> Result<(&str, f64), ParseError> {
    match src.strip_prefix(".") {
        Some(rest) => match digits(rest) {
            Ok((leftover, mut dgts)) => {
                dgts.insert(0, '.');
                dgts.insert(0, '0');
                let fraction_str: String = dgts.iter().collect();
                let fraction_val = fraction_str.parse::<f64>().unwrap();
                Ok((leftover, fraction_val))
            }
            Err(e) => Err(e),
        },
        None => Ok((src, 0.0)),
    }
}

fn exponent(mut src: &str) -> Result<(&str, i64), ParseError> {
    match src.chars().next() {
        Some(c) if c == 'e' || c == 'E' => {
            src = &src[1..];
        }
        _ => return Ok((src, 0)),
    }

    let negative = match src.chars().next() {
        Some('+') => {
            src = &src[1..];
            false
        }
        Some('-') => {
            src = &src[1..];
            true
        }
        _ => false,
    };
    match digits(src) {
        Ok((rest, dgts)) => {
            let num_str: String = dgts.iter().collect();
            let num = num_str.parse::<i64>().unwrap();
            Ok((rest, if negative { -num } else { num }))
        }
        Err(e) => Err(e),
    }
}

pub fn number(mut src: &str) -> Result<Token, ParseError> {
    let mut result;
    let negative;

    match integer(src) {
        Ok((rest, num)) => {
            result = num.abs() as f64;
            negative = num.is_negative();
            src = rest;
        }
        Err(e) => return Err(e),
    };

    match fraction(src) {
        Ok((rest, frac)) => {
            result += frac;
            src = rest;
        }
        Err(ParseError::NotFound) => {}
        Err(e) => return Err(e),
    }

    match exponent(src) {
        Ok((rest, exponent)) => {
            src = rest;
            let multipier = 10_f64.powf(exponent as f64);
            result *= multipier;
        }
        Err(ParseError::NotFound) => {}
        Err(e) => return Err(e),
    }

    if negative {
        result *= -1.0f64;
    }

    Ok(Token::new(src, Value::Number(result)))
}

pub fn elements(mut src: &str) -> Result<(&str, Vec<Value>), ParseError> {
    let mut values = vec![];
    loop {
        match element(src) {
            Ok(Token { rest, value }) => {
                src = rest;
                values.push(value);
            }
            Err(e) => return Err(e),
        }
        if let Some(',') = src.chars().next() {
            src = &src[1..];
        } else {
            break;
        }
    }
    Ok((src, values))
}

fn trim_blanks(src: &str) -> &str {
    src.trim_start_matches(&[' ', '\n', '\t', '\r'])
}

fn element(mut src: &str) -> Result<Token, ParseError> {
    src = trim_blanks(src);
    match value(src) {
        Ok(Token{rest, value}) => Ok(Token::new(trim_blanks(rest), value)),
        Err(e) => Err(e),
    }
}

fn members(mut src: &str) -> Result<(&str, Vec<(String, Value)>), ParseError> {
    let mut values = vec![];

    loop {
        match member(src) {
            Ok((rest, v)) => {
                src = rest;
                values.push(v);
            }
            Err(e) => return Err(e),
        }

        if src.chars().next() == Some(',') {
            src = &src[1..];
        } else {
            break;
        }
    }

    Ok((src, values))
}

fn member(mut src: &str) -> Result<(&str, (String, Value)), ParseError> {
    src = trim_blanks(src);

    match string(src) {
        Ok(Token{rest, value: Value::String(key)}) => {
            src = rest;
            src = trim_blanks(src);

            // now expect a ":"
            if src.chars().next() == Some(':') {
                src = &src[1..];
                match element(src) {
                    Ok(Token{rest, value}) => return Ok((rest, (key, value))),
                    Err(e) => return Err(e),
                }
            } else {
                return Err(ParseError::UnexpectedChar(src.len()));
            }
        }
        Ok(_) => Err(ParseError::Error(src.len())),
        Err(e) => Err(e),
    }
}

fn object(mut src: &str) -> Result<Token, ParseError> {
    match src.strip_prefix("{") {
        Some(rest) => src = trim_blanks(rest),
        None => return Err(ParseError::NotFound),
    }
    if src.chars().next() == Some('}') {
        src = &src[1..];
        return Ok(Token::new(src, Value::Object(HashMap::default())));
    }

    match members(src) {
        Ok((rest, values)) => {
            if rest.chars().next() == Some('}') {
                let mut map = HashMap::new();
                values.iter().for_each(|(key, value)| {
                    map.insert(key.to_owned(), value.to_owned());
                });
                Ok((&rest[1..], Value::Object(map)))
            } else {
                Err(ParseError::MissingClosing(src.len()))
            }
        }
        Err(e) => Err(e)
    }
}

fn value(src: &str) -> Result<Token, ParseError>{
    match object(src) {
        Ok(token) => return Ok(token),
        Err(ParseError::NotFound) => {},
        Err(e) => return  Err(e),
    }

    match array(src) {
        Ok(token) => return Ok(token),
        Err(ParseError::NotFound) => {},
        Err(e) => return  Err(e),
    }

    match string(src) {
        Ok(token) => return Ok(token),
        Err(ParseError::NotFound) => {},
        Err(e) => return  Err(e),
    }

    match number(src) {
        Ok(token) => return Ok(token),
        Err(ParseError::NotFound) => {},
        Err(e) => return  Err(e),
    }

    match bool(src) {
        Ok(token) => return Ok(token),
        Err(ParseError::NotFound) => {},
        Err(e) => return  Err(e),
    }

    match null(src) {
        Ok(token) => return Ok(token),
        Err(ParseError::NotFound) => {},
        Err(e) => return  Err(e),
    }
    Err(ParseError::NotFound)
}