use chrono::{DateTime, FixedOffset};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::error::ParseError;
use crate::formats::utils;
use crate::model::{BalanceType, Transaction, TransactionType};

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
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        if content.trim().is_empty() {
            return Err(ParseError::Camt053Error("Empty input".into()));
        }

        let mut xml_reader = Reader::from_str(&content);
        xml_reader.config_mut().trim_text(true);

        // State variables for parsing
        let mut account_number = String::new();
        let mut currency = String::new();
        let mut opening_balance: Option<f64> = None;
        let mut opening_date: Option<DateTime<FixedOffset>> = None;
        let mut opening_indicator: Option<BalanceType> = None;
        let mut closing_balance: Option<f64> = None;
        let mut closing_date: Option<DateTime<FixedOffset>> = None;
        let mut closing_indicator: Option<BalanceType> = None;
        let mut transactions: Vec<Transaction> = Vec::new();

        // Temporary state for current element parsing
        let mut current_path: Vec<String> = Vec::new();
        let mut current_balance_type = String::new();
        let mut current_balance_amount = String::new();
        let mut current_balance_indicator = String::new();
        let mut current_balance_date = String::new();

        // Entry (transaction) state
        let mut in_entry = false;
        let mut entry_amount = String::new();
        let mut entry_type = String::new();
        let mut entry_booking_date = String::new();
        let mut entry_value_date = String::new();
        let mut entry_reference = String::new();
        let mut entry_description = String::new();
        let mut entry_counterparty_name = String::new();
        let mut entry_counterparty_account = String::new();

        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_path.push(name.clone());

                    // Extract currency from Amt attribute
                    if name == "Amt" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"Ccy" {
                                let ccy = String::from_utf8_lossy(&attr.value).to_string();
                                if currency.is_empty() {
                                    currency = ccy;
                                }
                            }
                        }
                    }

                    if name == "Ntry" {
                        in_entry = true;
                        // Reset entry state
                        entry_amount.clear();
                        entry_type.clear();
                        entry_booking_date.clear();
                        entry_value_date.clear();
                        entry_reference.clear();
                        entry_description.clear();
                        entry_counterparty_name.clear();
                        entry_counterparty_account.clear();
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    // Process balance when </Bal> is reached
                    if name == "Bal" && !current_balance_type.is_empty() {
                        if current_balance_type == "OPBD" {
                            opening_balance = parse_amount(&current_balance_amount).ok();
                            opening_indicator =
                                parse_balance_indicator(&current_balance_indicator).ok();
                            opening_date = parse_xml_date(&current_balance_date).ok();
                        } else if current_balance_type == "CLBD" {
                            closing_balance = parse_amount(&current_balance_amount).ok();
                            closing_indicator =
                                parse_balance_indicator(&current_balance_indicator).ok();
                            closing_date = parse_xml_date(&current_balance_date).ok();
                        }
                        // Clear balance state
                        current_balance_type.clear();
                        current_balance_amount.clear();
                        current_balance_indicator.clear();
                        current_balance_date.clear();
                    }

                    // Process entry when </Ntry> is reached
                    if name == "Ntry" && in_entry {
                        if let (Ok(amount), Ok(tx_type), Ok(booking_date)) = (
                            parse_amount(&entry_amount),
                            parse_transaction_type(&entry_type),
                            parse_xml_date(&entry_booking_date),
                        ) {
                            let transaction = Transaction {
                                booking_date,
                                value_date: if entry_value_date.is_empty() {
                                    None
                                } else {
                                    Some(entry_value_date.clone())
                                },
                                amount,
                                transaction_type: tx_type,
                                description: entry_description.clone(),
                                reference: if entry_reference.is_empty() {
                                    None
                                } else {
                                    Some(entry_reference.clone())
                                },
                                counterparty_name: if entry_counterparty_name.is_empty() {
                                    None
                                } else {
                                    Some(entry_counterparty_name.clone())
                                },
                                counterparty_account: if entry_counterparty_account.is_empty() {
                                    None
                                } else {
                                    Some(entry_counterparty_account.clone())
                                },
                            };
                            transactions.push(transaction);
                        }
                        in_entry = false;
                    }

                    if !current_path.is_empty() {
                        current_path.pop();
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                    if text.is_empty() {
                        continue;
                    }

                    let path_str = current_path.join("/");

                    // Account number (from Stmt/Acct, not from transaction parties)
                    if path_str.ends_with("Stmt/Acct/Id/IBAN")
                        || (path_str.ends_with("Stmt/Acct/Id/Othr/Id") && account_number.is_empty())
                    {
                        account_number = text;
                    }
                    // Currency (also from Acct)
                    else if path_str.ends_with("Acct/Ccy") {
                        currency = text;
                    }
                    // Balance parsing
                    else if path_str.ends_with("Bal/Tp/CdOrPrtry/Cd") {
                        current_balance_type = text;
                    } else if path_str.ends_with("Bal/Amt") {
                        current_balance_amount = text;
                    } else if path_str.ends_with("Bal/CdtDbtInd") {
                        current_balance_indicator = text;
                    } else if path_str.ends_with("Bal/Dt/Dt") {
                        current_balance_date = text;
                    }
                    // Entry/Transaction parsing
                    else if in_entry {
                        if path_str.ends_with("Ntry/Amt") {
                            entry_amount = text;
                        } else if path_str.ends_with("Ntry/CdtDbtInd") {
                            entry_type = text;
                        } else if path_str.ends_with("Ntry/BookgDt/Dt") {
                            entry_booking_date = text;
                        } else if path_str.ends_with("Ntry/ValDt/Dt") {
                            entry_value_date = text;
                        } else if path_str.ends_with("Ntry/NtryRef") {
                            entry_reference = text.clone();
                        } else if path_str.contains("TxDtls") && path_str.ends_with("Refs/TxId") {
                            // TxId takes precedence, but NtryRef is a fallback
                            entry_reference = text.clone();
                        } else if path_str.ends_with("NtryDtls/TxDtls/RmtInf/Ustrd") {
                            if !entry_description.is_empty() {
                                entry_description.push(' ');
                            }
                            entry_description.push_str(&text);
                        } else if path_str.ends_with("NtryDtls/TxDtls/RmtInf/Strd/CdtrRefInf/Ref") {
                            if entry_description.is_empty() {
                                entry_description = text;
                            }
                        } else if path_str.contains("RltdPties") && path_str.ends_with("Dbtr/Nm") {
                            entry_counterparty_name = text;
                        } else if path_str.contains("RltdPties") && path_str.ends_with("Cdtr/Nm") {
                            if entry_counterparty_name.is_empty() {
                                entry_counterparty_name = text;
                            }
                        } else if (path_str.contains("DbtrAcct") && path_str.ends_with("Id/IBAN"))
                            || (entry_counterparty_account.is_empty()
                                && ((path_str.contains("CdtrAcct")
                                    && path_str.ends_with("Id/IBAN"))
                                    || (path_str.contains("DbtrAcct")
                                        && path_str.ends_with("Id/Othr/Id"))
                                    || (path_str.contains("CdtrAcct")
                                        && path_str.ends_with("Id/Othr/Id"))))
                        {
                            entry_counterparty_account = text.clone();
                        } else if path_str.ends_with("AddtlTxInf") {
                            if !entry_description.is_empty() {
                                entry_description.push(' ');
                            }
                            entry_description.push_str(&text);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(ParseError::Camt053Error(format!("XML parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        // Validate required fields
        if account_number.is_empty() {
            return Err(ParseError::MissingField("account_number".into()));
        }
        if currency.is_empty() {
            return Err(ParseError::MissingField("currency".into()));
        }

        Ok(Camt053 {
            account_number,
            currency,
            opening_balance: opening_balance.unwrap_or(0.0),
            opening_date: opening_date
                .ok_or_else(|| ParseError::MissingField("opening_date".into()))?,
            opening_indicator: opening_indicator
                .ok_or_else(|| ParseError::MissingField("opening_indicator".into()))?,
            closing_balance: closing_balance.unwrap_or(0.0),
            closing_date: closing_date
                .ok_or_else(|| ParseError::MissingField("closing_date".into()))?,
            closing_indicator: closing_indicator
                .ok_or_else(|| ParseError::MissingField("closing_indicator".into()))?,
            transactions,
        })
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

/// Parse amount from string (handles both dot and comma as decimal separator)
fn parse_amount(s: &str) -> Result<f64, ParseError> {
    let cleaned = s.trim().replace(',', ".");
    cleaned
        .parse::<f64>()
        .map_err(|_| ParseError::InvalidFieldValue {
            field: "amount".into(),
            value: s.into(),
        })
}

/// Parse XML date/datetime to DateTime<FixedOffset>
fn parse_xml_date(s: &str) -> Result<DateTime<FixedOffset>, ParseError> {
    let s = s.trim();

    // Try parsing as datetime first (2023-04-20T23:24:31)
    utils::parse_date(s)
}

/// Parse balance indicator (CRDT/DBIT) to BalanceType
fn parse_balance_indicator(s: &str) -> Result<BalanceType, ParseError> {
    match s.trim() {
        "CRDT" => Ok(BalanceType::Credit),
        "DBIT" => Ok(BalanceType::Debit),
        _ => Err(ParseError::InvalidFieldValue {
            field: "balance_indicator".into(),
            value: s.to_string(),
        }),
    }
}

/// Parse transaction type (CRDT/DBIT) to TransactionType
fn parse_transaction_type(s: &str) -> Result<TransactionType, ParseError> {
    match s.trim() {
        "CRDT" => Ok(TransactionType::Credit),
        "DBIT" => Ok(TransactionType::Debit),
        _ => Err(ParseError::InvalidFieldValue {
            field: "transaction_type".into(),
            value: s.to_string(),
        }),
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

    #[test]
    fn test_parse_minimal_camt053() {
        let xml = r#"
        <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
            <BkToCstmrStmt>
                <Stmt>
                    <Acct>
                        <Id><IBAN>DK8030000001234567</IBAN></Id>
                        <Ccy>DKK</Ccy>
                    </Acct>
                    <Bal>
                        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
                        <Amt Ccy="DKK">12345.67</Amt>
                        <CdtDbtInd>DBIT</CdtDbtInd>
                        <Dt><Dt>2023-04-20</Dt></Dt>
                    </Bal>
                    <Bal>
                        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
                        <Amt Ccy="DKK">23456.78</Amt>
                        <CdtDbtInd>DBIT</CdtDbtInd>
                        <Dt><Dt>2023-04-20</Dt></Dt>
                    </Bal>
                </Stmt>
            </BkToCstmrStmt>
        </Document>
        "#;

        let mut reader = xml.as_bytes();
        let result = Camt053::from_read(&mut reader);

        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement.account_number, "DK8030000001234567");
        assert_eq!(statement.currency, "DKK");
        assert_eq!(statement.opening_balance, 12345.67);
        assert_eq!(statement.closing_balance, 23456.78);
        assert_eq!(statement.opening_indicator, BalanceType::Debit);
        assert_eq!(statement.closing_indicator, BalanceType::Debit);
        assert_eq!(statement.transactions.len(), 0);
    }

    #[test]
    fn test_parse_camt053_with_transaction() {
        let xml = r#"
        <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
            <BkToCstmrStmt>
                <Stmt>
                    <Acct>
                        <Id><IBAN>DK8030000001234567</IBAN></Id>
                        <Ccy>DKK</Ccy>
                    </Acct>
                    <Bal>
                        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
                        <Amt Ccy="DKK">1000.00</Amt>
                        <CdtDbtInd>CRDT</CdtDbtInd>
                        <Dt><Dt>2023-04-20</Dt></Dt>
                    </Bal>
                    <Bal>
                        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
                        <Amt Ccy="DKK">1591.15</Amt>
                        <CdtDbtInd>CRDT</CdtDbtInd>
                        <Dt><Dt>2023-04-20</Dt></Dt>
                    </Bal>
                    <Ntry>
                        <NtryRef>1</NtryRef>
                        <Amt Ccy="DKK">591.15</Amt>
                        <CdtDbtInd>CRDT</CdtDbtInd>
                        <BookgDt><Dt>2023-04-20</Dt></BookgDt>
                        <ValDt><Dt>2023-04-20</Dt></ValDt>
                        <NtryDtls>
                            <TxDtls>
                                <Refs><TxId>3825-0123456789</TxId></Refs>
                                <RltdPties>
                                    <Dbtr><Nm>Debtor Name</Nm></Dbtr>
                                    <DbtrAcct><Id><IBAN>SE5180000810512345678901</IBAN></Id></DbtrAcct>
                                </RltdPties>
                                <RmtInf><Ustrd>Payment description</Ustrd></RmtInf>
                            </TxDtls>
                        </NtryDtls>
                    </Ntry>
                </Stmt>
            </BkToCstmrStmt>
        </Document>
        "#;

        let mut reader = xml.as_bytes();
        let result = Camt053::from_read(&mut reader);

        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement.transactions.len(), 1);

        let tx = &statement.transactions[0];
        assert_eq!(tx.amount, 591.15);
        assert_eq!(tx.transaction_type, TransactionType::Credit);
        assert_eq!(tx.reference, Some("3825-0123456789".to_string())); // TxId takes precedence
        assert_eq!(tx.description, "Payment description");
        assert_eq!(tx.counterparty_name, Some("Debtor Name".to_string()));
        assert_eq!(
            tx.counterparty_account,
            Some("SE5180000810512345678901".to_string())
        );
    }

    #[test]
    fn test_parse_empty_camt053() {
        let xml = "";
        let mut reader = xml.as_bytes();
        let result = Camt053::from_read(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_camt053_filters_balance_types() {
        // Should only use OPBD and CLBD, ignore OPAV and CLAV
        let xml = r#"
        <Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.02">
            <BkToCstmrStmt>
                <Stmt>
                    <Acct>
                        <Id><IBAN>DK8030000001234567</IBAN></Id>
                        <Ccy>DKK</Ccy>
                    </Acct>
                    <Bal>
                        <Tp><CdOrPrtry><Cd>OPBD</Cd></CdOrPrtry></Tp>
                        <Amt Ccy="DKK">100.00</Amt>
                        <CdtDbtInd>CRDT</CdtDbtInd>
                        <Dt><Dt>2023-04-20</Dt></Dt>
                    </Bal>
                    <Bal>
                        <Tp><CdOrPrtry><Cd>OPAV</Cd></CdOrPrtry></Tp>
                        <Amt Ccy="DKK">999.99</Amt>
                        <CdtDbtInd>CRDT</CdtDbtInd>
                        <Dt><Dt>2023-04-20</Dt></Dt>
                    </Bal>
                    <Bal>
                        <Tp><CdOrPrtry><Cd>CLBD</Cd></CdOrPrtry></Tp>
                        <Amt Ccy="DKK">200.00</Amt>
                        <CdtDbtInd>CRDT</CdtDbtInd>
                        <Dt><Dt>2023-04-20</Dt></Dt>
                    </Bal>
                    <Bal>
                        <Tp><CdOrPrtry><Cd>CLAV</Cd></CdOrPrtry></Tp>
                        <Amt Ccy="DKK">888.88</Amt>
                        <CdtDbtInd>CRDT</CdtDbtInd>
                        <Dt><Dt>2023-04-20</Dt></Dt>
                    </Bal>
                </Stmt>
            </BkToCstmrStmt>
        </Document>
        "#;

        let mut reader = xml.as_bytes();
        let result = Camt053::from_read(&mut reader);

        assert!(result.is_ok());
        let statement = result.unwrap();
        // Should use OPBD (100) and CLBD (200), not OPAV (999.99) or CLAV (888.88)
        assert_eq!(statement.opening_balance, 100.00);
        assert_eq!(statement.closing_balance, 200.00);
    }

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
            parse_balance_indicator("CRDT").unwrap(),
            BalanceType::Credit
        );
        assert_eq!(parse_balance_indicator("DBIT").unwrap(), BalanceType::Debit);
        assert!(parse_balance_indicator("INVALID").is_err());
    }

    #[test]
    fn test_parse_transaction_type() {
        assert_eq!(
            parse_transaction_type("CRDT").unwrap(),
            TransactionType::Credit
        );
        assert_eq!(
            parse_transaction_type("DBIT").unwrap(),
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
