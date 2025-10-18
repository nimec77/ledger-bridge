use chrono::{DateTime, FixedOffset};
use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesEnd, BytesStart, Event};
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

        let mut parser = CamtParser::default();
        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => parser.handle_start(&e)?,
                Ok(Event::End(e)) => parser.handle_end(&e)?,
                Ok(Event::Text(e)) => {
                    let bytes = e.as_ref();
                    if !bytes.is_empty() {
                        let decoded = String::from_utf8_lossy(bytes);
                        let trimmed = decoded.trim();
                        if !trimmed.is_empty() {
                            parser.handle_text(trimmed)?;
                        }
                    }
                }
                Ok(Event::CData(e)) => {
                    let text = String::from_utf8_lossy(e.as_ref());
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        parser.handle_text(trimmed)?;
                    }
                }
                Ok(Event::Eof) => break,
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
        // Implementation in Task 4.3
        let _ = writer;
        Err(ParseError::Camt053Error(
            "CAMT.053 writing to be implemented in Task 4.3".into(),
        ))
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
}

#[derive(Default)]
struct CamtParser {
    account_number: Option<String>,
    currency: Option<String>,
    opening_balance: Option<f64>,
    opening_date: Option<DateTime<FixedOffset>>,
    opening_indicator: Option<BalanceType>,
    closing_balance: Option<f64>,
    closing_date: Option<DateTime<FixedOffset>>,
    closing_indicator: Option<BalanceType>,
    transactions: Vec<Transaction>,
    balance_scratch: BalanceScratch,
    entry_scratch: Option<EntryScratch>,
    path: Vec<ElementName>,
}

impl CamtParser {
    fn handle_start(&mut self, event: &BytesStart) -> Result<(), ParseError> {
        let name = Self::map_name(event.name().as_ref())?;
        self.path.push(name);

        match name {
            ElementName::Balance => self.balance_scratch.clear(),
            ElementName::Entry => self.entry_scratch = Some(EntryScratch::default()),
            ElementName::Amount => self.capture_currency(event.attributes())?,
            _ => {}
        }

        Ok(())
    }

    fn handle_end(&mut self, _event: &BytesEnd) -> Result<(), ParseError> {
        if let Some(ended) = self.path.pop() {
            match ended {
                ElementName::Balance => self.finish_balance(),
                ElementName::Entry => self.finish_entry(),
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_text(&mut self, text: &str) -> Result<(), ParseError> {
        if self.in_statement_account_id() {
            self.set_account_number(text);
        } else if self.path_ends_with(&[ElementName::Acct, ElementName::Currency]) {
            self.set_currency(text);
        } else if self.path_ends_with(&[
            ElementName::Balance,
            ElementName::BalanceType,
            ElementName::CodeOrProprietary,
            ElementName::Code,
        ]) {
            self.balance_scratch.balance_type = Some(text.to_string());
        } else if self.path_ends_with(&[ElementName::Balance, ElementName::Amount]) {
            self.balance_scratch.amount = Some(text.to_string());
        } else if self.path_ends_with(&[ElementName::Balance, ElementName::CreditDebit]) {
            self.balance_scratch.indicator = Some(text.to_string());
        } else if self.path_ends_with(&[ElementName::Balance, ElementName::Date, ElementName::Date])
        {
            self.balance_scratch.date = Some(text.to_string());
        } else if self.path_ends_with(&[ElementName::Entry, ElementName::Amount]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.amount = Some(text.to_string());
            }
        } else if self.path_ends_with(&[ElementName::Entry, ElementName::CreditDebit]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.indicator = Some(text.to_string());
            }
        } else if self.path_ends_with(&[
            ElementName::Entry,
            ElementName::BookingDate,
            ElementName::Date,
        ]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.booking_date = Some(text.to_string());
            }
        } else if self.path_ends_with(&[
            ElementName::Entry,
            ElementName::ValueDate,
            ElementName::Date,
        ]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.value_date = Some(text.to_string());
            }
        } else if self.path_ends_with(&[ElementName::Entry, ElementName::EntryRef]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.ntry_ref = Some(text.to_string());
            }
        } else if self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::References,
            ElementName::TransactionId,
        ]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.tx_id = Some(text.to_string());
            }
        } else if self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::RemittanceInfo,
            ElementName::UnstructuredRemittance,
        ]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.push_description(text);
            }
        } else if self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::RemittanceInfo,
            ElementName::StructuredRemittance,
            ElementName::CreditorReferenceInfo,
            ElementName::ReferenceValue,
        ]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.set_description_if_empty(text);
            }
        } else if self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::RelatedParties,
            ElementName::Debtor,
            ElementName::Name,
        ]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.counterparty_name = Some(text.to_string());
            }
        } else if self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::RelatedParties,
            ElementName::Creditor,
            ElementName::Name,
        ]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                if entry.counterparty_name.is_none() {
                    entry.counterparty_name = Some(text.to_string());
                }
            }
        } else if self.in_debtor_account_id() {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.counterparty_account = Some(text.to_string());
            }
        } else if self.in_creditor_account_id() {
            if let Some(entry) = self.entry_scratch.as_mut() {
                if entry.counterparty_account.is_none() {
                    entry.counterparty_account = Some(text.to_string());
                }
            }
        } else if self.path_ends_with(&[ElementName::Entry, ElementName::AdditionalInfo]) {
            if let Some(entry) = self.entry_scratch.as_mut() {
                entry.push_description(text);
            }
        }

        Ok(())
    }

    fn build_statement(self) -> Result<Camt053, ParseError> {
        let account_number = self
            .account_number
            .ok_or_else(|| ParseError::MissingField("account_number".into()))?;
        let currency = self
            .currency
            .ok_or_else(|| ParseError::MissingField("currency".into()))?;

        Ok(Camt053 {
            account_number,
            currency,
            opening_balance: self.opening_balance.unwrap_or(0.0),
            opening_date: self
                .opening_date
                .ok_or_else(|| ParseError::MissingField("opening_date".into()))?,
            opening_indicator: self
                .opening_indicator
                .ok_or_else(|| ParseError::MissingField("opening_indicator".into()))?,
            closing_balance: self.closing_balance.unwrap_or(0.0),
            closing_date: self
                .closing_date
                .ok_or_else(|| ParseError::MissingField("closing_date".into()))?,
            closing_indicator: self
                .closing_indicator
                .ok_or_else(|| ParseError::MissingField("closing_indicator".into()))?,
            transactions: self.transactions,
        })
    }

    fn finish_balance(&mut self) {
        if let Some(balance_type) = self.balance_scratch.balance_type.as_deref() {
            match balance_type {
                "OPBD" => self.apply_balance(BalanceKind::Opening),
                "CLBD" => self.apply_balance(BalanceKind::Closing),
                _ => {}
            }
        }
        self.balance_scratch.clear();
    }

    fn apply_balance(&mut self, kind: BalanceKind) {
        if let Some(amount_text) = self.balance_scratch.amount.as_deref() {
            if let Ok(amount) = Camt053::parse_amount(amount_text) {
                match kind {
                    BalanceKind::Opening => self.opening_balance = Some(amount),
                    BalanceKind::Closing => self.closing_balance = Some(amount),
                }
            }
        }

        if let Some(indicator_text) = self.balance_scratch.indicator.as_deref() {
            if let Ok(indicator) = Camt053::parse_balance_indicator(indicator_text) {
                match kind {
                    BalanceKind::Opening => self.opening_indicator = Some(indicator),
                    BalanceKind::Closing => self.closing_indicator = Some(indicator),
                }
            }
        }

        if let Some(date_text) = self.balance_scratch.date.as_deref() {
            if let Ok(date) = Camt053::parse_xml_date(date_text) {
                match kind {
                    BalanceKind::Opening => self.opening_date = Some(date),
                    BalanceKind::Closing => self.closing_date = Some(date),
                }
            }
        }
    }

    fn finish_entry(&mut self) {
        if let Some(entry) = self.entry_scratch.take() {
            if let Ok(Some(tx)) = entry.finish() {
                self.transactions.push(tx);
            }
        }
    }

    fn capture_currency(&mut self, attributes: Attributes<'_>) -> Result<(), ParseError> {
        if self.currency.is_some() {
            return Ok(());
        }

        for attr in attributes {
            let attr = attr
                .map_err(|err| ParseError::Camt053Error(format!("XML attribute error: {}", err)))?;
            if attr.key.as_ref() == b"Ccy" {
                let value = String::from_utf8(attr.value.as_ref().to_vec()).map_err(|err| {
                    ParseError::Camt053Error(format!("Invalid currency encoding: {}", err))
                })?;
                if !value.trim().is_empty() {
                    self.currency = Some(value);
                }
                break;
            }
        }

        Ok(())
    }

    fn set_account_number(&mut self, text: &str) {
        if self
            .account_number
            .as_ref()
            .map(|value| value.is_empty())
            .unwrap_or(true)
        {
            self.account_number = Some(text.to_string());
        }
    }

    fn set_currency(&mut self, text: &str) {
        if self.currency.is_none() && !text.trim().is_empty() {
            self.currency = Some(text.to_string());
        }
    }

    fn path_ends_with(&self, suffix: &[ElementName]) -> bool {
        if self.path.len() < suffix.len() {
            return false;
        }
        let offset = self.path.len() - suffix.len();
        self.path[offset..] == *suffix
    }

    fn in_statement_account_id(&self) -> bool {
        self.path_ends_with(&[ElementName::Acct, ElementName::Id, ElementName::Iban])
            || self.path_ends_with(&[
                ElementName::Acct,
                ElementName::Id,
                ElementName::Other,
                ElementName::Id,
            ])
    }

    fn in_debtor_account_id(&self) -> bool {
        self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::RelatedParties,
            ElementName::DebtorAccount,
            ElementName::Id,
            ElementName::Iban,
        ]) || self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::RelatedParties,
            ElementName::DebtorAccount,
            ElementName::Id,
            ElementName::Other,
            ElementName::Id,
        ])
    }

    fn in_creditor_account_id(&self) -> bool {
        self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::RelatedParties,
            ElementName::CreditorAccount,
            ElementName::Id,
            ElementName::Iban,
        ]) || self.path_ends_with(&[
            ElementName::Entry,
            ElementName::EntryDetails,
            ElementName::TransactionDetails,
            ElementName::RelatedParties,
            ElementName::CreditorAccount,
            ElementName::Id,
            ElementName::Other,
            ElementName::Id,
        ])
    }

    fn map_name(raw: &[u8]) -> Result<ElementName, ParseError> {
        let name = std::str::from_utf8(raw).map_err(|err| {
            ParseError::Camt053Error(format!("Invalid XML tag name encoding: {}", err))
        })?;
        let normalized = name.rsplit(':').next().unwrap_or(name);
        Ok(ElementName::from_name(normalized))
    }
}

#[derive(Default)]
struct BalanceScratch {
    balance_type: Option<String>,
    amount: Option<String>,
    indicator: Option<String>,
    date: Option<String>,
}

impl BalanceScratch {
    fn clear(&mut self) {
        self.balance_type = None;
        self.amount = None;
        self.indicator = None;
        self.date = None;
    }
}

#[derive(Default)]
struct EntryScratch {
    amount: Option<String>,
    indicator: Option<String>,
    booking_date: Option<String>,
    value_date: Option<String>,
    ntry_ref: Option<String>,
    tx_id: Option<String>,
    description: String,
    counterparty_name: Option<String>,
    counterparty_account: Option<String>,
}

impl EntryScratch {
    fn push_description(&mut self, text: &str) {
        if !self.description.is_empty() {
            self.description.push(' ');
        }
        self.description.push_str(text);
    }

    fn set_description_if_empty(&mut self, text: &str) {
        if self.description.is_empty() {
            self.description = text.to_string();
        }
    }

    fn finish(self) -> Result<Option<Transaction>, ParseError> {
        let amount = match self
            .amount
            .as_deref()
            .and_then(|value| Camt053::parse_amount(value).ok())
        {
            Some(value) => value,
            None => return Ok(None),
        };

        let transaction_type = match self
            .indicator
            .as_deref()
            .and_then(|value| Camt053::parse_transaction_type(value).ok())
        {
            Some(value) => value,
            None => return Ok(None),
        };

        let booking_date = match self
            .booking_date
            .as_deref()
            .and_then(|value| Camt053::parse_xml_date(value).ok())
        {
            Some(value) => value,
            None => return Ok(None),
        };

        let value_date = self.value_date.map(|value| value.to_string());
        let reference = self.tx_id.or(self.ntry_ref);
        let counterparty_name = self.counterparty_name;
        let counterparty_account = self.counterparty_account;
        let description = self.description.trim().to_string();

        Ok(Some(Transaction {
            booking_date,
            value_date,
            amount,
            transaction_type,
            description,
            reference,
            counterparty_name,
            counterparty_account,
        }))
    }
}

enum BalanceKind {
    Opening,
    Closing,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ElementName {
    Document,
    BkToCstmrStmt,
    Stmt,
    Acct,
    Id,
    Iban,
    Currency,
    Balance,
    BalanceType,
    CodeOrProprietary,
    Code,
    Amount,
    CreditDebit,
    Date,
    Entry,
    EntryRef,
    BookingDate,
    ValueDate,
    EntryDetails,
    TransactionDetails,
    References,
    TransactionId,
    RemittanceInfo,
    UnstructuredRemittance,
    StructuredRemittance,
    CreditorReferenceInfo,
    ReferenceValue,
    RelatedParties,
    Debtor,
    Creditor,
    DebtorAccount,
    CreditorAccount,
    Name,
    AdditionalInfo,
    Other,
}

impl ElementName {
    fn from_name(name: &str) -> Self {
        match name {
            "Document" => ElementName::Document,
            "BkToCstmrStmt" => ElementName::BkToCstmrStmt,
            "Stmt" => ElementName::Stmt,
            "Acct" => ElementName::Acct,
            "Id" => ElementName::Id,
            "IBAN" => ElementName::Iban,
            "Ccy" => ElementName::Currency,
            "Bal" => ElementName::Balance,
            "Tp" => ElementName::BalanceType,
            "CdOrPrtry" => ElementName::CodeOrProprietary,
            "Cd" => ElementName::Code,
            "Amt" => ElementName::Amount,
            "CdtDbtInd" => ElementName::CreditDebit,
            "Dt" => ElementName::Date,
            "Ntry" => ElementName::Entry,
            "NtryRef" => ElementName::EntryRef,
            "BookgDt" => ElementName::BookingDate,
            "ValDt" => ElementName::ValueDate,
            "NtryDtls" => ElementName::EntryDetails,
            "TxDtls" => ElementName::TransactionDetails,
            "Refs" => ElementName::References,
            "TxId" => ElementName::TransactionId,
            "RmtInf" => ElementName::RemittanceInfo,
            "Ustrd" => ElementName::UnstructuredRemittance,
            "Strd" => ElementName::StructuredRemittance,
            "CdtrRefInf" => ElementName::CreditorReferenceInfo,
            "Ref" => ElementName::ReferenceValue,
            "RltdPties" => ElementName::RelatedParties,
            "Dbtr" => ElementName::Debtor,
            "Cdtr" => ElementName::Creditor,
            "DbtrAcct" => ElementName::DebtorAccount,
            "CdtrAcct" => ElementName::CreditorAccount,
            "Nm" => ElementName::Name,
            "AddtlTxInf" => ElementName::AdditionalInfo,
            "Othr" => ElementName::Other,
            _ => ElementName::Other,
        }
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
        assert_eq!(Camt053::parse_amount("123.45").unwrap(), 123.45);
        assert_eq!(Camt053::parse_amount("123,45").unwrap(), 123.45);
        assert_eq!(Camt053::parse_amount("  123.45  ").unwrap(), 123.45);
        assert!(Camt053::parse_amount("invalid").is_err());
    }

    #[test]
    fn test_parse_balance_indicator() {
        assert_eq!(
            Camt053::parse_balance_indicator("CRDT").unwrap(),
            BalanceType::Credit
        );
        assert_eq!(
            Camt053::parse_balance_indicator("DBIT").unwrap(),
            BalanceType::Debit
        );
        assert!(Camt053::parse_balance_indicator("INVALID").is_err());
    }

    #[test]
    fn test_parse_transaction_type() {
        assert_eq!(
            Camt053::parse_transaction_type("CRDT").unwrap(),
            TransactionType::Credit
        );
        assert_eq!(
            Camt053::parse_transaction_type("DBIT").unwrap(),
            TransactionType::Debit
        );
        assert!(Camt053::parse_transaction_type("INVALID").is_err());
    }

    #[test]
    fn test_parse_xml_date() {
        // Test date only
        let result = Camt053::parse_xml_date("2023-04-20");
        assert!(result.is_ok());

        // Test datetime
        let result = Camt053::parse_xml_date("2023-04-20T23:24:31");
        assert!(result.is_ok());

        // Test with timezone
        let result = Camt053::parse_xml_date("2023-04-20T23:24:31+00:00");
        assert!(result.is_ok());
    }
}
