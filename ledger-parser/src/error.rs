/// Error type for all parsing and formatting operations
#[derive(Debug)]
pub enum ParseError {
    // General errors
    InvalidFormat(String),
    MissingField(String),
    InvalidFieldValue { field: String, value: String },

    // Format-specific errors
    CsvError(String),
    Mt940Error(String),
    Camt053Error(String),

    // I/O errors
    IoError(String),
}

// User-friendly error messages
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ParseError::InvalidFieldValue { field, value } => {
                write!(f, "Invalid value '{}' for field '{}'", value, field)
            }
            ParseError::CsvError(msg) => write!(f, "CSV error: {}", msg),
            ParseError::Mt940Error(msg) => write!(f, "MT940 error: {}", msg),
            ParseError::Camt053Error(msg) => write!(f, "CAMT.053 error: {}", msg),
            ParseError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

// Standard error trait
impl std::error::Error for ParseError {}

// Convert std::io::Error to ParseError
impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}

/// Automatic conversion from csv::Error to ParseError
impl From<csv::Error> for ParseError {
    fn from(err: csv::Error) -> Self {
        ParseError::CsvError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::InvalidFormat("Test format".to_string());
        assert_eq!(format!("{}", error), "Invalid format: Test format");
    }

    #[test]
    fn test_csv_error_display() {
        let error = ParseError::CsvError("Invalid CSV structure".to_string());
        assert_eq!(format!("{}", error), "CSV error: Invalid CSV structure");
    }

    #[test]
    fn test_missing_field_error_display() {
        let error = ParseError::MissingField("account_number".to_string());
        assert_eq!(
            format!("{}", error),
            "Missing required field: account_number"
        );
    }

    #[test]
    fn test_invalid_field_value_error_display() {
        let error = ParseError::InvalidFieldValue {
            field: "amount".to_string(),
            value: "invalid".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid value 'invalid' for field 'amount'"
        );
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let parse_error: ParseError = io_error.into();

        match parse_error {
            ParseError::IoError(msg) => assert!(msg.contains("File not found")),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_error_debug() {
        let error = ParseError::Mt940Error("Test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Mt940Error"));
        assert!(debug_str.contains("Test error"));
    }
}
