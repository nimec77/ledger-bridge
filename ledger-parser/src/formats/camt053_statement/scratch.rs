use crate::error::ParseError;
use crate::model::Transaction;

use super::camt053_utils;

#[derive(Default)]
pub struct BalanceScratch {
    pub balance_type: Option<String>,
    pub amount: Option<String>,
    pub indicator: Option<String>,
    pub date: Option<String>,
}

impl BalanceScratch {
    pub fn clear(&mut self) {
        self.balance_type = None;
        self.amount = None;
        self.indicator = None;
        self.date = None;
    }
}

#[derive(Default)]
pub struct EntryScratch {
    pub amount: Option<String>,
    pub indicator: Option<String>,
    pub booking_date: Option<String>,
    pub value_date: Option<String>,
    pub ntry_ref: Option<String>,
    pub tx_id: Option<String>,
    pub description: String,
    pub counterparty_name: Option<String>,
    pub counterparty_account: Option<String>,
}

impl EntryScratch {
    pub fn push_description(&mut self, text: &str) {
        if !self.description.is_empty() {
            self.description.push(' ');
        }
        self.description.push_str(text);
    }

    pub fn set_description_if_empty(&mut self, text: &str) {
        if self.description.is_empty() {
            self.description = text.to_string();
        }
    }

    pub fn finish(self) -> Result<Option<Transaction>, ParseError> {
        let amount = match self
            .amount
            .as_deref()
            .and_then(|value| camt053_utils::parse_amount(value).ok())
        {
            Some(value) => value,
            None => return Ok(None),
        };

        let transaction_type = match self
            .indicator
            .as_deref()
            .and_then(|value| camt053_utils::parse_transaction_type(value).ok())
        {
            Some(value) => value,
            None => return Ok(None),
        };

        let booking_date = match self
            .booking_date
            .as_deref()
            .and_then(|value| camt053_utils::parse_xml_date(value).ok())
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
