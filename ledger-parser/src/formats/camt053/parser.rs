use chrono::{DateTime, FixedOffset};
use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesEnd, BytesStart};

use crate::error::ParseError;
use crate::model::{BalanceType, Transaction};

use super::elements::ElementName;
use super::scratch::{BalanceScratch, EntryScratch};
use super::camt053_utils;

#[derive(Default)]
pub struct CamtParser {
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
    pub fn handle_start(&mut self, event: &BytesStart) -> Result<(), ParseError> {
        let name = ElementName::from_name_bytes(event.name().as_ref())?;
        self.path.push(name);

        match name {
            ElementName::Balance => self.balance_scratch.clear(),
            ElementName::Entry => self.entry_scratch = Some(EntryScratch::default()),
            ElementName::Amount => self.capture_currency(event.attributes())?,
            _ => {}
        }

        Ok(())
    }

    pub fn handle_end(&mut self, _event: &BytesEnd) -> Result<(), ParseError> {
        if let Some(ended) = self.path.pop() {
            match ended {
                ElementName::Balance => self.finish_balance(),
                ElementName::Entry => self.finish_entry(),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn handle_text(&mut self, text: &str) -> Result<(), ParseError> {
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

    pub fn build_statement(self) -> Result<super::Camt053, ParseError> {
        let account_number = self
            .account_number
            .ok_or_else(|| ParseError::MissingField("account_number".into()))?;
        let currency = self
            .currency
            .ok_or_else(|| ParseError::MissingField("currency".into()))?;

        Ok(super::Camt053 {
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
            match balance_type.to_lowercase().as_str() {
                "opbd" => self.apply_balance(BalanceKind::Opening),
                "clbd" => self.apply_balance(BalanceKind::Closing),
                _ => {}
            }
        }
        self.balance_scratch.clear();
    }

    fn apply_balance(&mut self, kind: BalanceKind) {
        if let Some(amount_text) = self.balance_scratch.amount.as_deref() {
            if let Ok(amount) = camt053_utils::parse_amount(amount_text) {
                match kind {
                    BalanceKind::Opening => self.opening_balance = Some(amount),
                    BalanceKind::Closing => self.closing_balance = Some(amount),
                }
            }
        }

        if let Some(indicator_text) = self.balance_scratch.indicator.as_deref() {
            if let Ok(indicator) = camt053_utils::parse_balance_indicator(indicator_text) {
                match kind {
                    BalanceKind::Opening => self.opening_indicator = Some(indicator),
                    BalanceKind::Closing => self.closing_indicator = Some(indicator),
                }
            }
        }

        if let Some(date_text) = self.balance_scratch.date.as_deref() {
            if let Ok(date) = camt053_utils::parse_xml_date(date_text) {
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
            
            // Convert attribute key to lowercase for case-insensitive comparison
            let key_str = std::str::from_utf8(attr.key.as_ref()).map_err(|err| {
                ParseError::Camt053Error(format!("Invalid attribute key encoding: {}", err))
            })?;
            
            if key_str.to_lowercase() == "ccy" {
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
}

enum BalanceKind {
    Opening,
    Closing,
}

#[cfg(test)]
mod tests {
    use crate::model::{BalanceType, TransactionType};

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
        let result = super::super::Camt053::from_read(&mut reader);

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
        let result = super::super::Camt053::from_read(&mut reader);

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
        let result = super::super::Camt053::from_read(&mut reader);
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
        let result = super::super::Camt053::from_read(&mut reader);

        assert!(result.is_ok());
        let statement = result.unwrap();
        // Should use OPBD (100) and CLBD (200), not OPAV (999.99) or CLAV (888.88)
        assert_eq!(statement.opening_balance, 100.00);
        assert_eq!(statement.closing_balance, 200.00);
    }
}
