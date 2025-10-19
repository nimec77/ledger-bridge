use chrono::{DateTime, FixedOffset};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Write;

use crate::model::{BalanceType, Transaction, TransactionType};

use super::{Camt053, ParseError};

/// Helper responsible for serialising `Camt053` statements into CAMT.053 XML.
pub struct CamtWriter<'a, W: Write> {
    statement: &'a Camt053,
    writer: Writer<&'a mut W>,
}

impl<'a, W: Write> CamtWriter<'a, W> {
    /// Create a new XML writer around the provided `Write` sink.
    pub fn new(statement: &'a Camt053, sink: &'a mut W) -> Self {
        let writer = Writer::new_with_indent(sink, b' ', 2);
        Self { statement, writer }
    }

    /// Render the CAMT.053 document to the sink.
    pub fn write(mut self) -> Result<(), ParseError> {
        self.write_document_start()?;
        self.write_statement()?;
        self.write_document_end()
    }

    fn write_document_start(&mut self) -> Result<(), ParseError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write XML declaration: {}", e))
            })?;

        let mut document = BytesStart::new("Document");
        document.push_attribute(("xmlns", "urn:iso:std:iso:20022:tech:xsd:camt.053.001.02"));
        self.writer
            .write_event(Event::Start(document))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write Document tag: {}", e))
            })?;

        Ok(())
    }

    fn write_document_end(&mut self) -> Result<(), ParseError> {
        self.writer
            .write_event(Event::End(BytesEnd::new("Document")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close Document tag: {}", e))
            })
    }

    fn write_statement(&mut self) -> Result<(), ParseError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("BkToCstmrStmt")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write BkToCstmrStmt tag: {}", e))
            })?;

        self.writer
            .write_event(Event::Start(BytesStart::new("Stmt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Stmt tag: {}", e)))?;

        self.write_account()?;
        self.write_balances()?;
        self.write_entries()?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Stmt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Stmt tag: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("BkToCstmrStmt")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close BkToCstmrStmt tag: {}", e))
            })?;

        Ok(())
    }

    fn write_account(&mut self) -> Result<(), ParseError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("Acct")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Acct tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("Id")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Id tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("IBAN")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write IBAN tag: {}", e)))?;

        self.writer
            .write_event(Event::Text(BytesText::new(&self.statement.account_number)))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write account number: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("IBAN")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close IBAN tag: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Id")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Id tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("Ccy")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Ccy tag: {}", e)))?;

        self.writer
            .write_event(Event::Text(BytesText::new(&self.statement.currency)))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write currency: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Ccy")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Ccy tag: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Acct")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Acct tag: {}", e)))?;

        Ok(())
    }

    fn write_balances(&mut self) -> Result<(), ParseError> {
        self.write_balance(
            "OPBD",
            self.statement.opening_balance,
            &self.statement.opening_indicator,
            &self.statement.opening_date,
        )?;

        self.write_balance(
            "CLBD",
            self.statement.closing_balance,
            &self.statement.closing_indicator,
            &self.statement.closing_date,
        )?;

        Ok(())
    }

    fn write_balance(
        &mut self,
        balance_type: &str,
        amount: f64,
        indicator: &BalanceType,
        date: &DateTime<FixedOffset>,
    ) -> Result<(), ParseError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("Bal")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Bal tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("Tp")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Tp tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("CdOrPrtry")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write CdOrPrtry tag: {}", e))
            })?;

        self.writer
            .write_event(Event::Start(BytesStart::new("Cd")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Cd tag: {}", e)))?;

        self.writer
            .write_event(Event::Text(BytesText::new(balance_type)))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write balance type: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Cd")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Cd tag: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("CdOrPrtry")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close CdOrPrtry tag: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Tp")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Tp tag: {}", e)))?;

        let mut amt_tag = BytesStart::new("Amt");
        amt_tag.push_attribute(("Ccy", self.statement.currency.as_str()));
        self.writer
            .write_event(Event::Start(amt_tag))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Amt tag: {}", e)))?;

        self.writer
            .write_event(Event::Text(BytesText::new(&format!("{:.2}", amount))))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write amount: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Amt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Amt tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("CdtDbtInd")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write CdtDbtInd tag: {}", e))
            })?;

        let indicator_str = match indicator {
            BalanceType::Credit => "CRDT",
            BalanceType::Debit => "DBIT",
        };
        self.writer
            .write_event(Event::Text(BytesText::new(indicator_str)))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write indicator: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("CdtDbtInd")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close CdtDbtInd tag: {}", e))
            })?;

        self.writer
            .write_event(Event::Start(BytesStart::new("Dt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Dt tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("Dt")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write inner Dt tag: {}", e))
            })?;

        self.writer
            .write_event(Event::Text(BytesText::new(
                &date.format("%Y-%m-%d").to_string(),
            )))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write date: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Dt")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close inner Dt tag: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Dt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Dt tag: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Bal")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Bal tag: {}", e)))?;

        Ok(())
    }

    fn write_entries(&mut self) -> Result<(), ParseError> {
        for (index, transaction) in self.statement.transactions.iter().enumerate() {
            self.write_entry(transaction, index + 1)?;
        }
        Ok(())
    }

    fn write_entry(&mut self, transaction: &Transaction, entry_ref: usize) -> Result<(), ParseError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("Ntry")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Ntry tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("NtryRef")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write NtryRef tag: {}", e)))?;

        self.writer
            .write_event(Event::Text(BytesText::new(&entry_ref.to_string())))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write entry reference: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("NtryRef")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close NtryRef tag: {}", e)))?;

        let mut amt_tag = BytesStart::new("Amt");
        amt_tag.push_attribute(("Ccy", self.statement.currency.as_str()));
        self.writer
            .write_event(Event::Start(amt_tag))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Amt tag: {}", e)))?;

        self.writer
            .write_event(Event::Text(BytesText::new(&format!(
                "{:.2}",
                transaction.amount
            ))))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write transaction amount: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Amt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Amt tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("CdtDbtInd")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write CdtDbtInd tag: {}", e))
            })?;

        let indicator_str = match transaction.transaction_type {
            TransactionType::Credit => "CRDT",
            TransactionType::Debit => "DBIT",
        };
        self.writer
            .write_event(Event::Text(BytesText::new(indicator_str)))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write transaction indicator: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("CdtDbtInd")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close CdtDbtInd tag: {}", e))
            })?;

        self.writer
            .write_event(Event::Start(BytesStart::new("BookgDt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write BookgDt tag: {}", e)))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("Dt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write Dt tag: {}", e)))?;

        self.writer
            .write_event(Event::Text(BytesText::new(
                &transaction.booking_date.format("%Y-%m-%d").to_string(),
            )))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write booking date: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Dt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Dt tag: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("BookgDt")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close BookgDt tag: {}", e)))?;

        if let Some(value_date) = transaction.value_date.as_ref() {
            self.writer
                .write_event(Event::Start(BytesStart::new("ValDt")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write ValDt tag: {}", e))
                })?;

            self.writer
                .write_event(Event::Start(BytesStart::new("Dt")))
                .map_err(|e| ParseError::Camt053Error(format!("Failed to write Dt tag: {}", e)))?;

            self.writer
                .write_event(Event::Text(BytesText::new(value_date)))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write value date: {}", e))
                })?;

            self.writer
                .write_event(Event::End(BytesEnd::new("Dt")))
                .map_err(|e| ParseError::Camt053Error(format!("Failed to close Dt tag: {}", e)))?;

            self.writer
                .write_event(Event::End(BytesEnd::new("ValDt")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close ValDt tag: {}", e))
                })?;
        }

        self.writer
            .write_event(Event::Start(BytesStart::new("NtryDtls")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to write NtryDtls tag: {}", e))
            })?;

        self.writer
            .write_event(Event::Start(BytesStart::new("TxDtls")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to write TxDtls tag: {}", e)))?;

        if transaction.reference.is_some() {
            self.writer
                .write_event(Event::Start(BytesStart::new("Refs")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write Refs tag: {}", e))
                })?;

            if let Some(reference) = transaction.reference.as_ref() {
                self.writer
                    .write_event(Event::Start(BytesStart::new("TxId")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write TxId tag: {}", e))
                    })?;

                self.writer
                    .write_event(Event::Text(BytesText::new(reference)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write reference: {}", e))
                    })?;

                self.writer
                    .write_event(Event::End(BytesEnd::new("TxId")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to close TxId tag: {}", e))
                    })?;
            }

            self.writer
                .write_event(Event::End(BytesEnd::new("Refs")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close Refs tag: {}", e))
                })?;
        }

        if transaction.counterparty_name.is_some() || transaction.counterparty_account.is_some() {
            self.writer
                .write_event(Event::Start(BytesStart::new("RltdPties")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write RltdPties tag: {}", e))
                })?;

            let party_tag = match transaction.transaction_type {
                TransactionType::Credit => "Dbtr",
                TransactionType::Debit => "Cdtr",
            };
            let account_tag = match transaction.transaction_type {
                TransactionType::Credit => "DbtrAcct",
                TransactionType::Debit => "CdtrAcct",
            };

            if let Some(counterparty_name) = transaction.counterparty_name.as_ref() {
                self.writer
                    .write_event(Event::Start(BytesStart::new(party_tag)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to write {} tag: {}",
                            party_tag, e
                        ))
                    })?;

                self.writer
                    .write_event(Event::Start(BytesStart::new("Nm")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write Nm tag: {}", e))
                    })?;

                self.writer
                    .write_event(Event::Text(BytesText::new(counterparty_name)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to write counterparty name: {}",
                            e
                        ))
                    })?;

                self.writer
                    .write_event(Event::End(BytesEnd::new("Nm")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to close Nm tag: {}", e))
                    })?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(party_tag)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to close {} tag: {}",
                            party_tag, e
                        ))
                    })?;
            }

            if let Some(counterparty_account) = transaction.counterparty_account.as_ref() {
                self.writer
                    .write_event(Event::Start(BytesStart::new(account_tag)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to write {} tag: {}",
                            account_tag, e
                        ))
                    })?;

                self.writer
                    .write_event(Event::Start(BytesStart::new("Id")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write Id tag: {}", e))
                    })?;

                self.writer
                    .write_event(Event::Start(BytesStart::new("IBAN")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to write IBAN tag: {}", e))
                    })?;

                self.writer
                    .write_event(Event::Text(BytesText::new(counterparty_account)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to write counterparty account: {}",
                            e
                        ))
                    })?;

                self.writer
                    .write_event(Event::End(BytesEnd::new("IBAN")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to close IBAN tag: {}", e))
                    })?;

                self.writer
                    .write_event(Event::End(BytesEnd::new("Id")))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!("Failed to close Id tag: {}", e))
                    })?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(account_tag)))
                    .map_err(|e| {
                        ParseError::Camt053Error(format!(
                            "Failed to close {} tag: {}",
                            account_tag, e
                        ))
                    })?;
            }

            self.writer
                .write_event(Event::End(BytesEnd::new("RltdPties")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close RltdPties tag: {}", e))
                })?;
        }

        if !transaction.description.is_empty() {
            self.writer
                .write_event(Event::Start(BytesStart::new("RmtInf")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write RmtInf tag: {}", e))
                })?;

            self.writer
                .write_event(Event::Start(BytesStart::new("Ustrd")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write Ustrd tag: {}", e))
                })?;

            self.writer
                .write_event(Event::Text(BytesText::new(&transaction.description)))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to write description: {}", e))
                })?;

            self.writer
                .write_event(Event::End(BytesEnd::new("Ustrd")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close Ustrd tag: {}", e))
                })?;

            self.writer
                .write_event(Event::End(BytesEnd::new("RmtInf")))
                .map_err(|e| {
                    ParseError::Camt053Error(format!("Failed to close RmtInf tag: {}", e))
                })?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("TxDtls")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close TxDtls tag: {}", e)))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("NtryDtls")))
            .map_err(|e| {
                ParseError::Camt053Error(format!("Failed to close NtryDtls tag: {}", e))
            })?;

        self.writer
            .write_event(Event::End(BytesEnd::new("Ntry")))
            .map_err(|e| ParseError::Camt053Error(format!("Failed to close Ntry tag: {}", e)))?;

        Ok(())
    }
}
