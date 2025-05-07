#[derive(Debug, Clone, PartialEq)]
pub enum FilterSegment {
    Identity,      // Represents the "." filter
    Field(String), // Represents ".field" access
    Index(usize),  // Represents .[index] access
}

#[derive(Debug)]
pub struct Filter {
    pub segments: Vec<FilterSegment>,
}

pub fn parse_filter_expression(filter_str: &str) -> Result<Filter, String> {
    if filter_str.is_empty() || filter_str == "." {
        return Ok(Filter {
            segments: vec![FilterSegment::Identity],
        });
    }

    if !filter_str.starts_with('.') {
        return Err("Filter expression must start with '.' (or be just '.')".to_string());
    }

    let mut segments = Vec::new();
    let mut current_filter = filter_str;

    // Ensure the first segment is processed correctly if it's the identity or a field
    if current_filter.starts_with(".") {
        current_filter = &current_filter[1..]; // Consume the leading dot
    } else {
        // This case should ideally be caught by the initial checks if filter_str isn't just "."
        return Err("Filter must start with '.'".to_string());
    }

    // Regex to parse segments like "field", "[0]", or "field[0]"
    // This is a simplified regex. A more robust one would be needed for complex cases.
    // For now, we'll split by '.' and then check for bracket indexing.
    for part in current_filter.split('.') {
        if part.is_empty() {
            // Happens with ".." or if current_filter became empty after stripping '.' (e.g. original was just ".")
            if segments.is_empty() && filter_str == "." {
                // Already handled by the identity case
                continue;
            }
            return Err(format!(
                "Invalid empty filter part in expression: {}",
                filter_str
            ));
        }

        // Check for array indexing like "field[0]" or just "[0]"
        if let Some(bracket_start) = part.find('[') {
            if bracket_start > 0 {
                // We have a field part, e.g., "field[0]"
                segments.push(FilterSegment::Field(part[0..bracket_start].to_string()));
            }
            if let Some(bracket_end) = part.rfind(']') {
                if bracket_end > bracket_start + 1 {
                    // Ensure there's a number inside
                    let index_str = &part[bracket_start + 1..bracket_end];
                    match index_str.parse::<usize>() {
                        Ok(idx) => segments.push(FilterSegment::Index(idx)),
                        Err(_) => return Err(format!("Invalid array index: {}", index_str)),
                    }
                } else {
                    return Err(format!("Malformed array index in part: {}", part));
                }
            } else {
                return Err(format!("Missing closing bracket in part: {}", part));
            }
        } else {
            // Just a field
            segments.push(FilterSegment::Field(part.to_string()));
        }
    }

    if segments.is_empty() {
        // This can happen if filter_str was only "." and current_filter became empty
        if filter_str == "." {
            return Ok(Filter {
                segments: vec![FilterSegment::Identity],
            });
        }
        return Err(format!(
            "Failed to parse filter into segments: {}",
            filter_str
        ));
    }

    Ok(Filter { segments })
}
