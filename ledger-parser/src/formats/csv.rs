use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use crate::{Transaction, BalanceType, ParseError};

/// CSV bank statement structure.
///
/// Parses from and writes to CSV format using the `csv` crate.
/// Fields are identical to Mt940/Camt053 for seamless conversions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsvStatement {
    pub account_number: String,
    pub currency: String,
    pub opening_balance: f64,
    pub opening_date: String,
    pub opening_indicator: BalanceType,
    pub closing_balance: f64,
    pub closing_date: String,
    pub closing_indicator: BalanceType,
    pub transactions: Vec<Transaction>,
}

impl CsvStatement {
    /// Parse CSV from any Read source (file, stdin, buffer).
    ///
    /// # Errors
    /// Returns `ParseError::CsvError` if the CSV structure is invalid.
    ///
    /// # Example
    /// ```
    /// let mut reader = csv_data.as_bytes();
    /// let statement = CsvStatement::from_read(&mut reader)?;
    /// ```
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Implementation will be added in task 2.2
        todo!("CSV parsing implementation")
    }
    
    /// Write CSV to any Write destination (file, stdout, buffer).
    ///
    /// # Errors
    /// Returns `ParseError::CsvError` if writing fails.
    ///
    /// # Example
    /// ```
    /// let mut writer = Vec::new();
    /// statement.write_to(&mut writer)?;
    /// ```
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Implementation will be added in task 2.3
        todo!("CSV writing implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionType;

    #[test]
    fn test_csv_statement_creation() {
        let statement = CsvStatement {
            account_number: "ACC123".to_string(),
            currency: "USD".to_string(),
            opening_balance: 1000.0,
            opening_date: "2025-01-01".to_string(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 1200.0,
            closing_date: "2025-01-31".to_string(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![
                Transaction {
                    booking_date: "2025-01-15".to_string(),
                    value_date: Some("2025-01-15".to_string()),
                    amount: 200.0,
                    transaction_type: TransactionType::Credit,
                    description: "Payment received".to_string(),
                    reference: Some("REF123".to_string()),
                    counterparty_name: Some("John Doe".to_string()),
                    counterparty_account: Some("IBAN123".to_string()),
                }
            ],
        };
        
        assert_eq!(statement.account_number, "ACC123");
        assert_eq!(statement.currency, "USD");
        assert_eq!(statement.opening_balance, 1000.0);
        assert_eq!(statement.transactions.len(), 1);
    }

    #[test]
    fn test_csv_statement_serialization() {
        let statement = CsvStatement {
            account_number: "ACC456".to_string(),
            currency: "EUR".to_string(),
            opening_balance: 500.0,
            opening_date: "2025-01-01".to_string(),
            opening_indicator: BalanceType::Debit,
            closing_balance: 300.0,
            closing_date: "2025-01-31".to_string(),
            closing_indicator: BalanceType::Debit,
            transactions: vec![],
        };
        
        // Test that it can be serialized and deserialized
        let serialized = serde_json::to_string(&statement).unwrap();
        let deserialized: CsvStatement = serde_json::from_str(&serialized).unwrap();
        assert_eq!(statement, deserialized);
    }

    #[test]
    fn test_csv_statement_with_empty_transactions() {
        let statement = CsvStatement {
            account_number: "ACC789".to_string(),
            currency: "GBP".to_string(),
            opening_balance: 0.0,
            opening_date: "2025-01-01".to_string(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 0.0,
            closing_date: "2025-01-31".to_string(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![],
        };
        
        assert_eq!(statement.transactions.len(), 0);
        assert_eq!(statement.opening_balance, 0.0);
        assert_eq!(statement.closing_balance, 0.0);
    }
}
