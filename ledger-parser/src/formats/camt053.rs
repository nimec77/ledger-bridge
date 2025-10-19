mod camt053_utils;
mod elements;
mod parser;
mod scratch;

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
        use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
        use quick_xml::Writer;

        let mut xml_writer = Writer::new_with_indent(writer, b' ', 2);

        // Write XML declaration
        xml_writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write XML declaration: {}", e))
            })?;

        // Document root with namespace
        let mut document = BytesStart::new("Document");
        document.push_attribute(("xmlns", "urn:iso:std:iso:20022:tech:xsd:camt.053.001.02"));
        xml_writer
            .write_event(Event::Start(document))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write Document tag: {}", e))
            })?;

        // BkToCstmrStmt
        xml_writer
            .write_event(Event::Start(BytesStart::new("BkToCstmrStmt")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write BkToCstmrStmt tag: {}", e))
            })?;

        // Stmt
        xml_writer
            .write_event(Event::Start(BytesStart::new("Stmt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Stmt tag: {}", e)))?;

        // Account information
        self.write_account(&mut xml_writer)?;

        // Opening balance
        self.write_balance(
            &mut xml_writer,
            "OPBD",
            self.opening_balance,
            &self.opening_indicator,
            &self.opening_date,
        )?;

        // Closing balance
        self.write_balance(
            &mut xml_writer,
            "CLBD",
            self.closing_balance,
            &self.closing_indicator,
            &self.closing_date,
        )?;

        // Transactions (entries)
        for (index, transaction) in self.transactions.iter().enumerate() {
            self.write_entry(&mut xml_writer, transaction, index + 1)?;
        }

        // Close Stmt
        xml_writer
            .write_event(Event::End(BytesEnd::new("Stmt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Stmt tag: {}", e)))?;

        // Close BkToCstmrStmt
        xml_writer
            .write_event(Event::End(BytesEnd::new("BkToCstmrStmt")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close BkToCstmrStmt tag: {}", e))
            })?;

        // Close Document
        xml_writer
            .write_event(Event::End(BytesEnd::new("Document")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close Document tag: {}", e))
            })?;

        Ok(())
    }

    /// Write account information section
    fn write_account<W: Write>(&self, writer: &mut quick_xml::Writer<W>) -> Result<(), ParseError> {
        use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

        writer
            .write_event(Event::Start(BytesStart::new("Acct")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Acct tag: {}", e)))?;

        writer
            .write_event(Event::Start(BytesStart::new("Id")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Id tag: {}", e)))?;

        writer
            .write_event(Event::Start(BytesStart::new("IBAN")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write IBAN tag: {}", e)))?;

        writer
            .write_event(Event::Text(BytesText::new(&self.account_number)))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write account number: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("IBAN")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close IBAN tag: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("Id")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Id tag: {}", e)))?;

        writer
            .write_event(Event::Start(BytesStart::new("Ccy")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Ccy tag: {}", e)))?;

        writer
            .write_event(Event::Text(BytesText::new(&self.currency)))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write currency: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("Ccy")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Ccy tag: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("Acct")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Acct tag: {}", e)))?;

        Ok(())
    }

    /// Write balance information section
    fn write_balance<W: Write>(
        &self,
        writer: &mut quick_xml::Writer<W>,
        balance_type: &str,
        amount: f64,
        indicator: &BalanceType,
        date: &DateTime<FixedOffset>,
    ) -> Result<(), ParseError> {
        use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

        writer
            .write_event(Event::Start(BytesStart::new("Bal")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Bal tag: {}", e)))?;

        // Balance type
        writer
            .write_event(Event::Start(BytesStart::new("Tp")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Tp tag: {}", e)))?;

        writer
            .write_event(Event::Start(BytesStart::new("CdOrPrtry")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write CdOrPrtry tag: {}", e))
            })?;

        writer
            .write_event(Event::Start(BytesStart::new("Cd")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Cd tag: {}", e)))?;

        writer
            .write_event(Event::Text(BytesText::new(balance_type)))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write balance type: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("Cd")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Cd tag: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("CdOrPrtry")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close CdOrPrtry tag: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("Tp")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Tp tag: {}", e)))?;

        // Amount with currency attribute
        let mut amt_tag = BytesStart::new("Amt");
        amt_tag.push_attribute(("Ccy", self.currency.as_str()));
        writer
            .write_event(Event::Start(amt_tag))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Amt tag: {}", e)))?;

        writer
            .write_event(Event::Text(BytesText::new(&format!("{:.2}", amount))))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write amount: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("Amt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Amt tag: {}", e)))?;

        // Credit/Debit indicator
        writer
            .write_event(Event::Start(BytesStart::new("CdtDbtInd")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write CdtDbtInd tag: {}", e))
            })?;

        let indicator_str = match indicator {
            BalanceType::Credit => "CRDT",
            BalanceType::Debit => "DBIT",
        };
        writer
            .write_event(Event::Text(BytesText::new(indicator_str)))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write indicator: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("CdtDbtInd")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close CdtDbtInd tag: {}", e))
            })?;

        // Date
        writer
            .write_event(Event::Start(BytesStart::new("Dt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Dt tag: {}", e)))?;

        writer
            .write_event(Event::Start(BytesStart::new("Dt")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write inner Dt tag: {}", e))
            })?;

        writer
            .write_event(Event::Text(BytesText::new(
                &date.format("%Y-%m-%d").to_string(),
            )))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write date: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("Dt")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close inner Dt tag: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("Dt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Dt tag: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("Bal")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Bal tag: {}", e)))?;

        Ok(())
    }

    /// Write transaction entry section
    fn write_entry<W: Write>(
        &self,
        writer: &mut quick_xml::Writer<W>,
        transaction: &Transaction,
        entry_ref: usize,
    ) -> Result<(), ParseError> {
        use crate::model::TransactionType;
        use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

        writer
            .write_event(Event::Start(BytesStart::new("Ntry")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Ntry tag: {}", e)))?;

        // Entry reference
        writer
            .write_event(Event::Start(BytesStart::new("NtryRef")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write NtryRef tag: {}", e)))?;

        writer
            .write_event(Event::Text(BytesText::new(&entry_ref.to_string())))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write entry reference: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("NtryRef")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close NtryRef tag: {}", e)))?;

        // Amount with currency attribute
        let mut amt_tag = BytesStart::new("Amt");
        amt_tag.push_attribute(("Ccy", self.currency.as_str()));
        writer
            .write_event(Event::Start(amt_tag))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Amt tag: {}", e)))?;

        writer
            .write_event(Event::Text(BytesText::new(&format!(
                "{:.2}",
                transaction.amount
            ))))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write transaction amount: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("Amt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Amt tag: {}", e)))?;

        // Credit/Debit indicator
        writer
            .write_event(Event::Start(BytesStart::new("CdtDbtInd")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write CdtDbtInd tag: {}", e))
            })?;

        let indicator_str = match transaction.transaction_type {
            TransactionType::Credit => "CRDT",
            TransactionType::Debit => "DBIT",
        };
        writer
            .write_event(Event::Text(BytesText::new(indicator_str)))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write transaction indicator: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("CdtDbtInd")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close CdtDbtInd tag: {}", e))
            })?;

        // Booking date
        writer
            .write_event(Event::Start(BytesStart::new("BookgDt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write BookgDt tag: {}", e)))?;

        writer
            .write_event(Event::Start(BytesStart::new("Dt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Dt tag: {}", e)))?;

        writer
            .write_event(Event::Text(BytesText::new(
                &transaction.booking_date.format("%Y-%m-%d").to_string(),
            )))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write booking date: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("Dt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Dt tag: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("BookgDt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close BookgDt tag: {}", e)))?;

        // Value date (if present)
        if let Some(ref value_date) = transaction.value_date {
            writer
                .write_event(Event::Start(BytesStart::new("ValDt")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write ValDt tag: {}", e))
                })?;

            writer
                .write_event(Event::Start(BytesStart::new("Dt")))
                .map_err(|e| ParseError::Camt053Error(format!("Failed to write Dt tag: {}", e)))?;

            writer
                .write_event(Event::Text(BytesText::new(value_date)))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write value date: {}", e))
                })?;

            writer
                .write_event(Event::End(BytesEnd::new("Dt")))
                .map_err(|e| ParseError::Camt053Error(format!("Failed to close Dt tag: {}", e)))?;

            writer
                .write_event(Event::End(BytesEnd::new("ValDt")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close ValDt tag: {}", e))
                })?;
        }

        // Entry details
        writer
            .write_event(Event::Start(BytesStart::new("NtryDtls")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write NtryDtls tag: {}", e))
            })?;

        writer
            .write_event(Event::Start(BytesStart::new("TxDtls")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write TxDtls tag: {}", e)))?;

        // References
        if transaction.reference.is_some() {
            writer
                .write_event(Event::Start(BytesStart::new("Refs")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write Refs tag: {}", e))
                })?;

            if let Some(ref reference) = transaction.reference {
                writer
                    .write_event(Event::Start(BytesStart::new("TxId")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write TxId tag: {}", e))
                    })?;

                writer
                    .write_event(Event::Text(BytesText::new(reference)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write reference: {}", e))
                    })?;

                writer
                    .write_event(Event::End(BytesEnd::new("TxId")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to close TxId tag: {}", e))
                    })?;
            }

            writer
                .write_event(Event::End(BytesEnd::new("Refs")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close Refs tag: {}", e))
                })?;
        }

        // Related parties (counterparty information)
        if transaction.counterparty_name.is_some() || transaction.counterparty_account.is_some() {
            writer
                .write_event(Event::Start(BytesStart::new("RltdPties")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write RltdPties tag: {}", e))
                })?;

            // For Credit transactions, counterparty is Debtor; for Debit, it's Creditor
            let party_tag = match transaction.transaction_type {
                TransactionType::Credit => "Dbtr",
                TransactionType::Debit => "Cdtr",
            };
            let account_tag = match transaction.transaction_type {
                TransactionType::Credit => "DbtrAcct",
                TransactionType::Debit => "CdtrAcct",
            };

            if let Some(ref counterparty_name) = transaction.counterparty_name {
                writer
                    .write_event(Event::Start(BytesStart::new(party_tag)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to write {} tag: {}",
                            party_tag, e
                        ))
                    })?;

                writer
                    .write_event(Event::Start(BytesStart::new("Nm")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write Nm tag: {}", e))
                    })?;

                writer
                    .write_event(Event::Text(BytesText::new(counterparty_name)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to write counterparty name: {}",
                            e
                        ))
                    })?;

                writer
                    .write_event(Event::End(BytesEnd::new("Nm")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to close Nm tag: {}", e))
                    })?;

                writer
                    .write_event(Event::End(BytesEnd::new(party_tag)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to close {} tag: {}",
                            party_tag, e
                        ))
                    })?;
            }

            if let Some(ref counterparty_account) = transaction.counterparty_account {
                writer
                    .write_event(Event::Start(BytesStart::new(account_tag)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to write {} tag: {}",
                            account_tag, e
                        ))
                    })?;

                writer
                    .write_event(Event::Start(BytesStart::new("Id")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write Id tag: {}", e))
                    })?;

                writer
                    .write_event(Event::Start(BytesStart::new("IBAN")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write IBAN tag: {}", e))
                    })?;

                writer
                    .write_event(Event::Text(BytesText::new(counterparty_account)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to write counterparty account: {}",
                            e
                        ))
                    })?;

                writer
                    .write_event(Event::End(BytesEnd::new("IBAN")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to close IBAN tag: {}", e))
                    })?;

                writer
                    .write_event(Event::End(BytesEnd::new("Id")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to close Id tag: {}", e))
                    })?;

                writer
                    .write_event(Event::End(BytesEnd::new(account_tag)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to close {} tag: {}",
                            account_tag, e
                        ))
                    })?;
            }

            writer
                .write_event(Event::End(BytesEnd::new("RltdPties")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close RltdPties tag: {}", e))
                })?;
        }

        // Remittance information (description)
        if !transaction.description.is_empty() {
            writer
                .write_event(Event::Start(BytesStart::new("RmtInf")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write RmtInf tag: {}", e))
                })?;

            writer
                .write_event(Event::Start(BytesStart::new("Ustrd")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write Ustrd tag: {}", e))
                })?;

            writer
                .write_event(Event::Text(BytesText::new(&transaction.description)))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write description: {}", e))
                })?;

            writer
                .write_event(Event::End(BytesEnd::new("Ustrd")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close Ustrd tag: {}", e))
                })?;

            writer
                .write_event(Event::End(BytesEnd::new("RmtInf")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close RmtInf tag: {}", e))
                })?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("TxDtls")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close TxDtls tag: {}", e)))?;

        writer
            .write_event(Event::End(BytesEnd::new("NtryDtls")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close NtryDtls tag: {}", e))
            })?;

        writer
            .write_event(Event::End(BytesEnd::new("Ntry")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Ntry tag: {}", e)))?;

        Ok(())
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
    fn test_write_minimal_camt053() {
        // Test writing a statement with no transactions
        let statement = Camt053 {
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
        let statement = Camt053 {
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
        let original = Camt053 {
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
        let parsed = Camt053::from_read(&mut reader).unwrap();

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
        let statement = Camt053 {
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
        let statement = Camt053 {
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
        let statement = Camt053 {
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
