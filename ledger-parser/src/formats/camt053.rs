use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::error::ParseError;
use crate::model::{BalanceType, Transaction};

/// ISO 20022 CAMT.053 XML structure
///
/// Parses from and writes to CAMT.053 XML format using the `quick-xml` crate.
/// Fields are identical to Mt940/CsvStatement for seamless conversions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camt053 {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: DateTime<FixedOffset>,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: DateTime<FixedOffset>,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}

impl Camt053 {
    /// Parse CAMT.053 from any source implementing Read
    ///
    /// Uses `quick-xml` event-based parsing to extract account information,
    /// balances (OPBD/CLBD types), and transaction entries from ISO 20022 XML.
    ///
    /// # Errors
    /// Returns `ParseError::Camt053Error` if the XML structure is invalid.
    ///
    /// # Example
    /// ```no_run
    /// use ledger_parser::Camt053;
    /// let xml = r#"<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">...</Document>"#;
    /// let mut reader = xml.as_bytes();
    /// let result = Camt053::from_read(&mut reader);
    /// ```
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Implementation in Task 4.2
        let _ = reader;
        Err(ParseError::Camt053Error(
            "CAMT.053 parsing to be implemented in Task 4.2".into(),
        ))
    }

    /// Write CAMT.053 to any destination implementing Write
    ///
    /// Generates ISO 20022 CAMT.053 XML using `quick-xml` writer.
    ///
    /// # Errors
    /// Returns `ParseError::Camt053Error` if XML generation fails.
    ///
    /// # Example
    /// ```no_run
    /// use ledger_parser::Camt053;
    /// use ledger_parser::{BalanceType, Transaction, TransactionType};
    /// use chrono::{DateTime, FixedOffset};
    ///
    /// let statement = Camt053 {
    ///     account_number: "DK1234567890".into(),
    ///     currency: "DKK".into(),
    ///     opening_balance: 1000.0,
    ///     opening_date: DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00").unwrap(),
    ///     opening_indicator: BalanceType::Credit,
    ///     closing_balance: 1500.0,
    ///     closing_date: DateTime::parse_from_rfc3339("2025-01-31T00:00:00+00:00").unwrap(),
    ///     closing_indicator: BalanceType::Credit,
    ///     transactions: vec![],
    /// };
    /// let mut output = Vec::new();
    /// statement.write_to(&mut output).unwrap();
    /// ```
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Implementation in Task 4.3
        let _ = writer;
        Err(ParseError::Camt053Error(
            "CAMT.053 writing to be implemented in Task 4.3".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camt053_structure() {
        // Test that the structure can be created
        use crate::formats::utils;

        let statement = Camt053 {
            account_number: "DK1234567890".into(),
            currency: "DKK".into(),
            opening_balance: 1000.0,
            opening_date: utils::parse_date("2025-01-01").unwrap(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 1500.0,
            closing_date: utils::parse_date("2025-01-31").unwrap(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![],
        };

        assert_eq!(statement.account_number, "DK1234567890");
        assert_eq!(statement.currency, "DKK");
        assert_eq!(statement.opening_balance, 1000.0);
        assert_eq!(statement.closing_balance, 1500.0);
    }
}
