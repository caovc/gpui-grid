//! Converts user-entered text into a typed cell value before the atomic edit.
//! This module contains no GUI dependencies, so native and WASM use the same rules.
use super::{DataType, Value};

/// Parse a user-supplied editor value without silently coercing invalid types.
///
/// Empty numeric/boolean fields are represented by `Value::Empty` because the
/// core supports nullable cells. Empty text remains a string, not a null.
/// Both the native and browser editor must then call `GridModel::apply_edits`
/// for the final read-only, row-ID and column-ID validation.
pub fn parse_cell_value(kind: DataType, input: &str) -> Result<Value, &'static str> {
    match kind {
        DataType::Text => Ok(Value::Text(input.to_owned())),
        DataType::Number => {
            let raw = input.trim();
            if raw.is_empty() {
                return Ok(Value::Empty);
            }
            match raw.parse::<f64>() {
                Ok(number) if number.is_finite() => Ok(Value::Number(number)),
                _ => Err("Enter a finite number (e.g. 123.45)"),
            }
        }
        DataType::Boolean => {
            let raw = input.trim();
            if raw.is_empty() {
                return Ok(Value::Empty);
            }
            if raw.eq_ignore_ascii_case("true") {
                Ok(Value::Boolean(true))
            } else if raw.eq_ignore_ascii_case("false") {
                Ok(Value::Boolean(false))
            } else {
                Err("Enter true or false")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_editor_preserves_whitespace_and_unicode() {
        assert_eq!(parse_cell_value(DataType::Text, "  中文 test  "), Ok(Value::from("  中文 test  ")));
        assert_eq!(parse_cell_value(DataType::Text, ""), Ok(Value::from("")));
    }

    #[test]
    fn number_editor_is_typed_nullable_and_finite() {
        assert_eq!(parse_cell_value(DataType::Number, " 123.45 "), Ok(Value::Number(123.45)));
        assert_eq!(parse_cell_value(DataType::Number, "1e3"), Ok(Value::Number(1000.0)));
        assert_eq!(parse_cell_value(DataType::Number, "  "), Ok(Value::Empty));
        for invalid in ["abc", "12foo", "NaN", "inf", "-Infinity", "1e9999"] {
            assert!(parse_cell_value(DataType::Number, invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn boolean_editor_accepts_only_true_or_false() {
        assert_eq!(parse_cell_value(DataType::Boolean, "TRUE"), Ok(Value::Boolean(true)));
        assert_eq!(parse_cell_value(DataType::Boolean, " false "), Ok(Value::Boolean(false)));
        assert_eq!(parse_cell_value(DataType::Boolean, ""), Ok(Value::Empty));
        assert!(parse_cell_value(DataType::Boolean, "1").is_err());
        assert!(parse_cell_value(DataType::Boolean, "yes").is_err());
    }
}
