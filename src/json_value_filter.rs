use crate::filter::{Filter, FilterSegment};
use crate::json_parser::JSONValue;

pub fn apply_filter_to_value(
    initial_value: &JSONValue,
    filter: &Filter,
) -> Result<JSONValue, String> {
    let mut current_value_holder = initial_value.clone(); // Start with a clone

    for segment in &filter.segments {
        match segment {
            FilterSegment::Identity => {
                // If identity is a segment, it usually means the filter was just "."
                // which should have been handled by cloning the initial_value.
                // If it appears mid-filter (e.g. ".key1..key2"), parse_filter should prevent it.
                // For now, if encountered, it means no change to current_value_holder in this step.
            }
            FilterSegment::Field(key) => {
                match &current_value_holder {
                    JSONValue::Object(map) => {
                        if let Some(next_value) = map.get(key) {
                            current_value_holder = next_value.clone();
                        } else {
                            return Ok(JSONValue::Null); // Key not found, return null
                        }
                    }
                    _ => {
                        return Ok(JSONValue::Null); // Cannot access field on non-object, return null
                    }
                }
            }
            FilterSegment::Index(idx) => {
                match &current_value_holder {
                    JSONValue::Array(arr) => {
                        if let Some(next_value) = arr.get(*idx) {
                            current_value_holder = next_value.clone();
                        } else {
                            return Ok(JSONValue::Null); // Index out of bounds, return null
                        }
                    }
                    _ => {
                        return Ok(JSONValue::Null); // Cannot index non-array, return null
                    }
                }
            }
        }
    }
    Ok(current_value_holder)
}
