mod camt053_const;
mod camt053_utils;
mod elements;
mod parser;
mod scratch;
mod writer;

use parser::CamtParser;

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
pub struct Camt053Statement {
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

impl Camt053Statement {
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
    /// use ledger_parser::Camt053Statement;
    /// let xml = r#"<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">...</Document>"#;
    /// let mut reader = xml.as_bytes();
    /// let result = Camt053Statement::from_read(&mut reader);
    /// ```
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        if content.trim().is_empty() {
            return Err(ParseError::Camt053Error("Empty input".into()));
        }

        let mut xml_reader = quick_xml::Reader::from_str(&content);
        xml_reader.config_mut().trim_text(true);

        let mut parser = CamtParser::default();
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) => parser.handle_start(&e)?,
                Ok(quick_xml::events::Event::End(e)) => parser.handle_end(&e)?,
                Ok(quick_xml::events::Event::Text(e)) => {
                    let bytes = e.as_ref();
                    if !bytes.is_empty() {
                        let decoded = String::from_utf8_lossy(bytes);
                        let trimmed = decoded.trim();
                        if !trimmed.is_empty() {
                            parser.handle_text(trimmed)?;
                        }
                    }
                }
                Ok(quick_xml::events::Event::CData(e)) => {
                    let text = String::from_utf8_lossy(e.as_ref());
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        parser.handle_text(trimmed)?;
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(ParseError::Camt053Error(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        parser.build_statement()
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
    /// use ledger_parser::Camt053Statement;
    /// use ledger_parser::{BalanceType, Transaction, TransactionType};
    /// use chrono::{DateTime, FixedOffset};
    ///
    /// let statement = Camt053Statement {
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
        writer::CamtWriter::new(self, writer).write()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::utils;
    use crate::model::{Transaction, TransactionType};

    #[test]
    fn test_camt053_structure() {
        // Test that the structure can be created
        let statement = Camt053Statement {
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

    #[test]
    fn test_write_minimal_camt053() {
        // Test writing a statement with no transactions
        let statement = Camt053Statement {
            account_number: "DK8030000001234567".into(),
            currency: "DKK".into(),
            opening_balance: 1000.00,
            opening_date: utils::parse_date("2025-01-01").unwrap(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 1500.00,
            closing_date: utils::parse_date("2025-01-31").unwrap(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![],
        };

        let mut output = Vec::new();
        let result = statement.write_to(&mut output);

        assert!(result.is_ok());
        let xml_output = String::from_utf8(output).unwrap();

        // Verify key elements are present
        assert!(xml_output.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml_output
            .contains("<Document xmlns=\"urn:iso:std:iso:20022:tech:xsd:camt.053.001.02\">"));
        assert!(xml_output.contains("<IBAN>DK8030000001234567</IBAN>"));
        assert!(xml_output.contains("<Ccy>DKK</Ccy>"));
        assert!(xml_output.contains("<Cd>OPBD</Cd>"));
        assert!(xml_output.contains("<Cd>CLBD</Cd>"));
        assert!(xml_output.contains("<Amt Ccy=\"DKK\">1000.00</Amt>"));
        assert!(xml_output.contains("<Amt Ccy=\"DKK\">1500.00</Amt>"));
        assert!(xml_output.contains("<CdtDbtInd>CRDT</CdtDbtInd>"));
        assert!(xml_output.contains("</Document>"));
    }

    #[test]
    fn test_write_camt053_with_transactions() {
        // Test writing a statement with transactions
        let statement = Camt053Statement {
            account_number: "DK8030000001234567".into(),
            currency: "DKK".into(),
            opening_balance: 1000.00,
            opening_date: utils::parse_date("2025-01-01").unwrap(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 1591.15,
            closing_date: utils::parse_date("2025-01-31").unwrap(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![
                Transaction {
                    booking_date: utils::parse_date("2025-01-15").unwrap(),
                    value_date: Some("2025-01-15".into()),
                    amount: 591.15,
                    transaction_type: TransactionType::Credit,
                    description: "Payment received".into(),
                    reference: Some("TXN-123".into()),
                    counterparty_name: Some("John Doe".into()),
                    counterparty_account: Some("SE5180000810512345678901".into()),
                },
                Transaction {
                    booking_date: utils::parse_date("2025-01-20").unwrap(),
                    value_date: None,
                    amount: 250.00,
                    transaction_type: TransactionType::Debit,
                    description: "Payment sent".into(),
                    reference: Some("TXN-456".into()),
                    counterparty_name: Some("Jane Smith".into()),
                    counterparty_account: Some("NO9386011117947".into()),
                },
            ],
        };

        let mut output = Vec::new();
        let result = statement.write_to(&mut output);

        assert!(result.is_ok());
        let xml_output = String::from_utf8(output).unwrap();

        // Verify transactions are present
        assert!(xml_output.contains("<Ntry>"));
        assert!(xml_output.contains("<NtryRef>1</NtryRef>"));
        assert!(xml_output.contains("<NtryRef>2</NtryRef>"));
        assert!(xml_output.contains("<Amt Ccy=\"DKK\">591.15</Amt>"));
        assert!(xml_output.contains("<Amt Ccy=\"DKK\">250.00</Amt>"));
        assert!(xml_output.contains("<TxId>TXN-123</TxId>"));
        assert!(xml_output.contains("<TxId>TXN-456</TxId>"));
        assert!(xml_output.contains("<Dbtr>"));
        assert!(xml_output.contains("<Nm>John Doe</Nm>"));
        assert!(xml_output.contains("<Cdtr>"));
        assert!(xml_output.contains("<Nm>Jane Smith</Nm>"));
        assert!(xml_output.contains("<Ustrd>Payment received</Ustrd>"));
        assert!(xml_output.contains("<Ustrd>Payment sent</Ustrd>"));
    }

    #[test]
    fn test_round_trip_camt053() {
        // Test that parsing and writing preserves data
        let original = Camt053Statement {
            account_number: "DK8030000001234567".into(),
            currency: "DKK".into(),
            opening_balance: 12345.67,
            opening_date: utils::parse_date("2025-04-20").unwrap(),
            opening_indicator: BalanceType::Debit,
            closing_balance: 23456.78,
            closing_date: utils::parse_date("2025-04-20").unwrap(),
            closing_indicator: BalanceType::Debit,
            transactions: vec![Transaction {
                booking_date: utils::parse_date("2025-04-20").unwrap(),
                value_date: Some("2025-04-20".into()),
                amount: 591.15,
                transaction_type: TransactionType::Credit,
                description: "Payment description".into(),
                reference: Some("3825-0123456789".into()),
                counterparty_name: Some("Debtor Name".into()),
                counterparty_account: Some("SE5180000810512345678901".into()),
            }],
        };

        // Write to buffer
        let mut buffer = Vec::new();
        original.write_to(&mut buffer).unwrap();

        // Parse back
        let mut reader = buffer.as_slice();
        let parsed = Camt053Statement::from_read(&mut reader).unwrap();

        // Verify all fields match
        assert_eq!(parsed.account_number, original.account_number);
        assert_eq!(parsed.currency, original.currency);
        assert_eq!(parsed.opening_balance, original.opening_balance);
        assert_eq!(
            parsed.opening_date.format("%Y-%m-%d").to_string(),
            original.opening_date.format("%Y-%m-%d").to_string()
        );
        assert_eq!(parsed.opening_indicator, original.opening_indicator);
        assert_eq!(parsed.closing_balance, original.closing_balance);
        assert_eq!(
            parsed.closing_date.format("%Y-%m-%d").to_string(),
            original.closing_date.format("%Y-%m-%d").to_string()
        );
        assert_eq!(parsed.closing_indicator, original.closing_indicator);
        assert_eq!(parsed.transactions.len(), original.transactions.len());

        // Verify transaction details
        let parsed_tx = &parsed.transactions[0];
        let original_tx = &original.transactions[0];
        assert_eq!(parsed_tx.amount, original_tx.amount);
        assert_eq!(parsed_tx.transaction_type, original_tx.transaction_type);
        assert_eq!(parsed_tx.description, original_tx.description);
        assert_eq!(parsed_tx.reference, original_tx.reference);
        assert_eq!(parsed_tx.counterparty_name, original_tx.counterparty_name);
        assert_eq!(
            parsed_tx.counterparty_account,
            original_tx.counterparty_account
        );
    }

    #[test]
    fn test_write_to_buffer() {
        // Test writing to an in-memory buffer
        let statement = Camt053Statement {
            account_number: "TEST123".into(),
            currency: "EUR".into(),
            opening_balance: 500.0,
            opening_date: utils::parse_date("2025-01-01").unwrap(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 750.0,
            closing_date: utils::parse_date("2025-01-31").unwrap(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![],
        };

        let mut output = Vec::new();
        let result = statement.write_to(&mut output);

        assert!(result.is_ok());
        assert!(!output.is_empty());

        // Verify it's valid UTF-8
        let xml_string = String::from_utf8(output).unwrap();
        assert!(xml_string.starts_with("<?xml"));
    }

    #[test]
    fn test_write_camt053_with_debit_balance() {
        // Test writing a statement with debit balances
        let statement = Camt053Statement {
            account_number: "DEBIT123".into(),
            currency: "USD".into(),
            opening_balance: 100.0,
            opening_date: utils::parse_date("2025-01-01").unwrap(),
            opening_indicator: BalanceType::Debit,
            closing_balance: 50.0,
            closing_date: utils::parse_date("2025-01-31").unwrap(),
            closing_indicator: BalanceType::Debit,
            transactions: vec![],
        };

        let mut output = Vec::new();
        let result = statement.write_to(&mut output);

        assert!(result.is_ok());
        let xml_output = String::from_utf8(output).unwrap();

        // Verify debit indicators are present
        assert!(xml_output.contains("<CdtDbtInd>DBIT</CdtDbtInd>"));
    }

    #[test]
    fn test_write_camt053_transaction_without_optional_fields() {
        // Test writing transactions with minimal information
        let statement = Camt053Statement {
            account_number: "MINIMAL123".into(),
            currency: "GBP".into(),
            opening_balance: 1000.0,
            opening_date: utils::parse_date("2025-01-01").unwrap(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 1100.0,
            closing_date: utils::parse_date("2025-01-31").unwrap(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![Transaction {
                booking_date: utils::parse_date("2025-01-15").unwrap(),
                value_date: None,
                amount: 100.0,
                transaction_type: TransactionType::Credit,
                description: "Simple payment".into(),
                reference: None,
                counterparty_name: None,
                counterparty_account: None,
            }],
        };

        let mut output = Vec::new();
        let result = statement.write_to(&mut output);

        assert!(result.is_ok());
        let xml_output = String::from_utf8(output).unwrap();

        // Verify minimal transaction structure
        assert!(xml_output.contains("<Ntry>"));
        assert!(xml_output.contains("<Amt Ccy=\"GBP\">100.00</Amt>"));
        assert!(xml_output.contains("<Ustrd>Simple payment</Ustrd>"));
        // Should not contain optional elements when they're not present
        assert!(!xml_output.contains("<TxId>"));
        assert!(!xml_output.contains("<Dbtr>"));
        assert!(!xml_output.contains("<DbtrAcct>"));
    }
}
