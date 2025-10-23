use thiserror::Error;

/// Error type for all parsing and formatting operations in the ledger-parser library.
///
/// This unified error type covers all possible error conditions that can occur
/// during parsing, conversion, and writing of financial statement formats.
///
/// # Error Categories
/// - **General errors**: Format validation, missing fields, invalid values
/// - **Format-specific errors**: CSV, MT940, and CAMT.053 parsing errors
/// - **I/O errors**: File reading/writing failures
///
/// # Example
/// ```
/// use ledger_parser::{Mt940Statement, ParseError};
///
/// fn parse_statement(data: &str) -> Result<Mt940Statement, ParseError> {
///     let mut reader = data.as_bytes();
///     Mt940Statement::from_read(&mut reader)
/// }
///
/// match parse_statement("invalid") {
///     Ok(_) => println!("Success"),
///     Err(ParseError::Mt940Error(msg)) => eprintln!("MT940 parse error: {}", msg),
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// ```
#[derive(Error, Debug)]
pub enum ParseError {
    /// Invalid or unsupported format specified
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    /// Required field is missing from the input
    #[error("Missing required field: {0}")]
    MissingField(String),
    /// Field value cannot be parsed or is invalid
    #[error("Invalid value '{value}' for field '{field}'")]
    InvalidFieldValue {
        /// Name of the field that has an invalid value
        field: String,
        /// The invalid value that was encountered
        value: String,
    },

    /// CSV format parsing error
    #[error("CSV error: {0}")]
    CsvError(String),
    /// MT940 format parsing error
    #[error("MT940 error: {0}")]
    Mt940Error(String),
    /// CAMT.053 XML format parsing error
    #[error("CAMT.053 error: {0}")]
    Camt053Error(String),
    /// I/O operation error (file reading/writing)
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Automatic conversion from CSV errors to ParseError
impl From<csv::Error> for ParseError {
    fn from(error: csv::Error) -> Self {
        ParseError::CsvError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::InvalidFormat("Test format".into());
        assert_eq!(format!("{}", error), "Invalid format: Test format");
    }

    #[test]
    fn test_csv_error_display() {
        let error = ParseError::CsvError("Invalid CSV structure".into());
        assert!(format!("{}", error).contains("Invalid CSV structure"));
    }

    #[test]
    fn test_missing_field_error_display() {
        let error = ParseError::MissingField("account_number".into());
        assert_eq!(
            format!("{}", error),
            "Missing required field: account_number"
        );
    }

    #[test]
    fn test_invalid_field_value_error_display() {
        let error = ParseError::InvalidFieldValue {
            field: "amount".into(),
            value: "invalid".into(),
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
            ParseError::IoError(error) => assert!(error.to_string().contains("File not found")),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_error_debug() {
        let error = ParseError::Mt940Error("Test error".into());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Mt940Error"));
        assert!(debug_str.contains("Test error"));
    }
}
