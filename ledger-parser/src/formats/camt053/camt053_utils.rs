use chrono::{DateTime, FixedOffset};

use crate::error::ParseError;
use crate::formats::utils;
use crate::model::{BalanceType, TransactionType};

const CRDT: &str = "crdt";
const DBIT: &str = "dbit";

/// Parse amount from string (handles both dot and comma as decimal separator)
pub(super) fn parse_amount(s: &str) -> Result<f64, ParseError> {
    let cleaned = s.trim().replace(',', ".");
    cleaned
        .parse::<f64>()
        .map_err(|_| ParseError::InvalidFieldValue {
            field: "amount".into(),
            value: s.into(),
        })
}

/// Parse XML date/datetime to DateTime<FixedOffset>
pub(super) fn parse_xml_date(s: &str) -> Result<DateTime<FixedOffset>, ParseError> {
    // Try parsing as datetime first (2023-04-20T23:24:31)
    utils::parse_date(s.trim())
}

/// Parse balance indicator (CRDT/DBIT) to BalanceType
pub(super) fn parse_balance_indicator(s: &str) -> Result<BalanceType, ParseError> {
    match s.trim().to_lowercase().as_str() {
        CRDT => Ok(BalanceType::Credit),
        DBIT => Ok(BalanceType::Debit),
        _ => Err(ParseError::InvalidFieldValue {
            field: "balance_indicator".into(),
            value: s.to_string(),
        }),
    }
}

/// Parse transaction type (CRDT/DBIT) to TransactionType
pub(super) fn parse_transaction_type(s: &str) -> Result<TransactionType, ParseError> {
    match s.trim().to_lowercase().as_str() {
        CRDT => Ok(TransactionType::Credit),
        DBIT => Ok(TransactionType::Debit),
        _ => Err(ParseError::InvalidFieldValue {
            field: "transaction_type".into(),
            value: s.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BalanceType, TransactionType};

    #[test]
    fn test_parse_amount() {
        assert_eq!(parse_amount("123.45").unwrap(), 123.45);
        assert_eq!(parse_amount("123,45").unwrap(), 123.45);
        assert_eq!(parse_amount("  123.45  ").unwrap(), 123.45);
        assert!(parse_amount("invalid").is_err());
    }

    #[test]
    fn test_parse_balance_indicator() {
        assert_eq!(
            parse_balance_indicator(CRDT).unwrap(),
            BalanceType::Credit
        );
        assert_eq!(
            parse_balance_indicator(DBIT).unwrap(),
            BalanceType::Debit
        );
        assert!(parse_balance_indicator("INVALID").is_err());
    }

    #[test]
    fn test_parse_transaction_type() {
        assert_eq!(
            parse_transaction_type(CRDT).unwrap(),
            TransactionType::Credit
        );
        assert_eq!(
            parse_transaction_type(DBIT).unwrap(),
            TransactionType::Debit
        );
        assert!(parse_transaction_type("INVALID").is_err());
    }

    #[test]
    fn test_parse_xml_date() {
        // Test date only
        let result = parse_xml_date("2023-04-20");
        assert!(result.is_ok());

        // Test datetime
        let result = parse_xml_date("2023-04-20T23:24:31");
        assert!(result.is_ok());

        // Test with timezone
        let result = parse_xml_date("2023-04-20T23:24:31+00:00");
        assert!(result.is_ok());
    }
}
