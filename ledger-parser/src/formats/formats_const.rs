/// Zero amount value for empty or null amount fields.
/// Used when parsing empty amount strings.
pub const ZERO_AMOUNT: f64 = 0.0;

/// ## Formatting Constants
///
/// Constants for number and date formatting.
///
/// Comma decimal separator (Russian format)
pub const DECIMAL_SEPARATOR_COMMA: &str = ",";

/// Dot decimal separator (international format)
pub const DECIMAL_SEPARATOR_DOT: &str = ".";

/// Negative sign for amounts
pub const NEGATIVE_SIGN: &str = "-";

/// Empty string (for positive amounts)
pub const POSITIVE_SIGN: &str = "";
