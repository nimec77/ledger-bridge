use crate::{BalanceType, ParseError, Transaction, TransactionType};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// CSV bank statement structure.
///
/// Parses from and writes to CSV format using the `csv` crate.
/// Fields are identical to Mt940/Camt053 for seamless conversions.
///
/// # CSV Format
///
/// The CSV file has two row types:
/// 1. **Statement metadata row** (first row after header):
///    - account_number, currency, opening_balance, opening_date, opening_indicator,
///      closing_balance, closing_date, closing_indicator
/// 2. **Transaction rows** (subsequent rows):
///    - booking_date, value_date, amount, transaction_type, description,
///      reference, counterparty_name, counterparty_account
///
/// The statement row and transaction rows have different columns, so we parse them
/// separately using Serde deserialization.
///
/// # Example
///
/// ```csv
/// account_number,currency,opening_balance,opening_date,opening_indicator,closing_balance,closing_date,closing_indicator
/// ACC123,USD,1000.0,2024-01-01,Credit,1200.0,2024-01-31,Credit
/// booking_date,value_date,amount,transaction_type,description,reference,counterparty_name,counterparty_account
/// 2024-01-15,2024-01-15,200.0,Credit,Payment received,REF123,John Doe,IBAN123
/// ```
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
    #[serde(skip)]
    pub transactions: Vec<Transaction>,
}

/// Statement metadata row (first data row in CSV)
#[derive(Debug, Serialize, Deserialize)]
struct StatementMetadata {
    account_number: String,
    currency: String,
    opening_balance: f64,
    opening_date: String,
    opening_indicator: String,
    closing_balance: f64,
    closing_date: String,
    closing_indicator: String,
}

/// Transaction data row (subsequent rows in CSV)
#[derive(Debug, Serialize, Deserialize)]
struct TransactionRow {
    booking_date: String,
    value_date: String,
    amount: f64,
    transaction_type: String,
    description: String,
    reference: String,
    counterparty_name: String,
    counterparty_account: String,
}

impl CsvStatement {
    /// Parse CSV from any Read source (file, stdin, buffer).
    ///
    /// The CSV format consists of:
    /// 1. Header row with statement column names
    /// 2. One statement metadata row
    /// 3. Header row with transaction column names
    /// 4. Zero or more transaction rows
    ///
    /// # Errors
    ///
    /// Returns `ParseError::CsvError` if:
    /// - The CSV structure is invalid
    /// - Required fields are missing
    /// - Field values cannot be parsed
    ///
    /// # Example
    ///
    /// ```
    /// use ledger_parser::CsvStatement;
    ///
    /// let csv_data = "\
    /// account_number,currency,opening_balance,opening_date,opening_indicator,closing_balance,closing_date,closing_indicator
    /// ACC123,USD,1000.0,2024-01-01,Credit,1200.0,2024-01-31,Credit
    /// booking_date,value_date,amount,transaction_type,description,reference,counterparty_name,counterparty_account
    /// 2024-01-15,2024-01-15,200.0,Credit,Payment received,REF123,John Doe,IBAN123
    /// ";
    ///
    /// let mut reader = csv_data.as_bytes();
    /// let statement = CsvStatement::from_read(&mut reader).unwrap();
    /// assert_eq!(statement.account_number, "ACC123");
    /// assert_eq!(statement.transactions.len(), 1);
    /// ```
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Read all CSV content
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        // Split into lines to handle two different CSV structures
        let lines: Vec<&str> = content.lines().collect();

        // Find where statement metadata ends and transactions begin
        let transaction_header_idx = lines
            .iter()
            .position(|line| line.starts_with("booking_date"))
            .ok_or_else(|| {
                ParseError::CsvError(
                    "Missing transaction header row (must start with 'booking_date')".to_string(),
                )
            })?;

        // Parse statement metadata (first section)
        let statement_section = lines[..transaction_header_idx].join("\n");
        let mut statement_reader = csv::Reader::from_reader(statement_section.as_bytes());

        let metadata: StatementMetadata = statement_reader
            .deserialize()
            .next()
            .ok_or_else(|| ParseError::CsvError("Missing statement metadata row".to_string()))??;

        // Parse transactions (second section)
        let transaction_section = lines[transaction_header_idx..].join("\n");
        let mut transaction_reader = csv::Reader::from_reader(transaction_section.as_bytes());

        let mut transactions = Vec::new();
        for result in transaction_reader.deserialize() {
            let row: TransactionRow = result?;
            transactions.push(Self::parse_transaction_row(row)?);
        }

        Ok(CsvStatement {
            account_number: metadata.account_number,
            currency: metadata.currency,
            opening_balance: metadata.opening_balance,
            opening_date: metadata.opening_date,
            opening_indicator: Self::parse_balance_type(&metadata.opening_indicator)?,
            closing_balance: metadata.closing_balance,
            closing_date: metadata.closing_date,
            closing_indicator: Self::parse_balance_type(&metadata.closing_indicator)?,
            transactions,
        })
    }

    /// Write CSV to any Write destination (file, stdout, buffer).
    ///
    /// Outputs a CSV with two sections:
    /// 1. Statement metadata with column headers
    /// 2. Transaction rows with column headers
    ///
    /// # Errors
    ///
    /// Returns `ParseError::CsvError` if writing fails.
    ///
    /// # Example
    ///
    /// ```
    /// use ledger_parser::{CsvStatement, Transaction, BalanceType, TransactionType};
    ///
    /// let statement = CsvStatement {
    ///     account_number: "ACC123".to_string(),
    ///     currency: "USD".to_string(),
    ///     opening_balance: 1000.0,
    ///     opening_date: "2024-01-01".to_string(),
    ///     opening_indicator: BalanceType::Credit,
    ///     closing_balance: 1200.0,
    ///     closing_date: "2024-01-31".to_string(),
    ///     closing_indicator: BalanceType::Credit,
    ///     transactions: vec![],
    /// };
    ///
    /// let mut output = Vec::new();
    /// statement.write_to(&mut output).unwrap();
    /// assert!(!output.is_empty());
    /// ```
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Build complete CSV in memory, then write to output
        let mut buffer = Vec::new();

        // Write statement metadata section
        {
            let mut csv_writer = csv::Writer::from_writer(&mut buffer);

            csv_writer.serialize(StatementMetadata {
                account_number: self.account_number.clone(),
                currency: self.currency.clone(),
                opening_balance: self.opening_balance,
                opening_date: self.opening_date.clone(),
                opening_indicator: Self::format_balance_type(&self.opening_indicator),
                closing_balance: self.closing_balance,
                closing_date: self.closing_date.clone(),
                closing_indicator: Self::format_balance_type(&self.closing_indicator),
            })?;

            csv_writer.flush()?;
        }

        // Write transaction section
        if self.transactions.is_empty() {
            // Manually write header row when no transactions
            let header = "booking_date,value_date,amount,transaction_type,description,reference,counterparty_name,counterparty_account\n";
            buffer.extend_from_slice(header.as_bytes());
        } else {
            let mut csv_writer = csv::Writer::from_writer(&mut buffer);

            for tx in &self.transactions {
                csv_writer.serialize(TransactionRow {
                    booking_date: tx.booking_date.clone(),
                    value_date: tx.value_date.clone().unwrap_or_default(),
                    amount: tx.amount,
                    transaction_type: Self::format_transaction_type(&tx.transaction_type),
                    description: tx.description.clone(),
                    reference: tx.reference.clone().unwrap_or_default(),
                    counterparty_name: tx.counterparty_name.clone().unwrap_or_default(),
                    counterparty_account: tx.counterparty_account.clone().unwrap_or_default(),
                })?;
            }

            csv_writer.flush()?;
        }

        // Write complete buffer to output
        writer.write_all(&buffer)?;

        Ok(())
    }

    /// Parse BalanceType from string
    fn parse_balance_type(s: &str) -> Result<BalanceType, ParseError> {
        match s {
            "Credit" => Ok(BalanceType::Credit),
            "Debit" => Ok(BalanceType::Debit),
            _ => Err(ParseError::InvalidFieldValue {
                field: "balance_indicator".to_string(),
                value: s.to_string(),
            }),
        }
    }

    /// Format BalanceType as string
    fn format_balance_type(bt: &BalanceType) -> String {
        match bt {
            BalanceType::Credit => "Credit".to_string(),
            BalanceType::Debit => "Debit".to_string(),
        }
    }

    /// Parse TransactionType from string
    fn parse_transaction_type(s: &str) -> Result<TransactionType, ParseError> {
        match s {
            "Credit" => Ok(TransactionType::Credit),
            "Debit" => Ok(TransactionType::Debit),
            _ => Err(ParseError::InvalidFieldValue {
                field: "transaction_type".to_string(),
                value: s.to_string(),
            }),
        }
    }

    /// Format TransactionType as string
    fn format_transaction_type(tt: &TransactionType) -> String {
        match tt {
            TransactionType::Credit => "Credit".to_string(),
            TransactionType::Debit => "Debit".to_string(),
        }
    }

    /// Convert TransactionRow to Transaction
    fn parse_transaction_row(row: TransactionRow) -> Result<Transaction, ParseError> {
        Ok(Transaction {
            booking_date: row.booking_date,
            value_date: if row.value_date.is_empty() {
                None
            } else {
                Some(row.value_date)
            },
            amount: row.amount,
            transaction_type: Self::parse_transaction_type(&row.transaction_type)?,
            description: row.description,
            reference: if row.reference.is_empty() {
                None
            } else {
                Some(row.reference)
            },
            counterparty_name: if row.counterparty_name.is_empty() {
                None
            } else {
                Some(row.counterparty_name)
            },
            counterparty_account: if row.counterparty_account.is_empty() {
                None
            } else {
                Some(row.counterparty_account)
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            transactions: vec![Transaction {
                booking_date: "2025-01-15".to_string(),
                value_date: Some("2025-01-15".to_string()),
                amount: 200.0,
                transaction_type: TransactionType::Credit,
                description: "Payment received".to_string(),
                reference: Some("REF123".to_string()),
                counterparty_name: Some("John Doe".to_string()),
                counterparty_account: Some("IBAN123".to_string()),
            }],
        };

        assert_eq!(statement.account_number, "ACC123");
        assert_eq!(statement.currency, "USD");
        assert_eq!(statement.opening_balance, 1000.0);
        assert_eq!(statement.transactions.len(), 1);
    }

    #[test]
    fn test_csv_parse_simple() {
        let csv_data = "\
account_number,currency,opening_balance,opening_date,opening_indicator,closing_balance,closing_date,closing_indicator
ACC123,USD,1000.0,2024-01-01,Credit,1200.0,2024-01-31,Credit
booking_date,value_date,amount,transaction_type,description,reference,counterparty_name,counterparty_account
2024-01-15,2024-01-15,200.0,Credit,Payment received,REF123,John Doe,IBAN123
";

        let mut reader = csv_data.as_bytes();
        let result = CsvStatement::from_read(&mut reader);

        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement.account_number, "ACC123");
        assert_eq!(statement.currency, "USD");
        assert_eq!(statement.opening_balance, 1000.0);
        assert_eq!(statement.transactions.len(), 1);
        assert_eq!(statement.transactions[0].amount, 200.0);
    }

    #[test]
    fn test_csv_write_simple() {
        let statement = CsvStatement {
            account_number: "ACC456".to_string(),
            currency: "EUR".to_string(),
            opening_balance: 500.0,
            opening_date: "2025-01-01".to_string(),
            opening_indicator: BalanceType::Debit,
            closing_balance: 300.0,
            closing_date: "2025-01-31".to_string(),
            closing_indicator: BalanceType::Debit,
            transactions: vec![Transaction {
                booking_date: "2025-01-10".to_string(),
                value_date: None,
                amount: 100.0,
                transaction_type: TransactionType::Debit,
                description: "Payment made".to_string(),
                reference: None,
                counterparty_name: None,
                counterparty_account: None,
            }],
        };

        let mut output = Vec::new();
        let result = statement.write_to(&mut output);

        assert!(result.is_ok());
        assert!(!output.is_empty());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("ACC456"));
        assert!(output_str.contains("EUR"));
        assert!(output_str.contains("500"));
    }

    #[test]
    fn test_csv_round_trip() {
        let original = CsvStatement {
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

        // Write to bytes
        let mut output = Vec::new();
        original.write_to(&mut output).unwrap();

        // Read back
        let mut reader = output.as_slice();
        let parsed = CsvStatement::from_read(&mut reader).unwrap();

        assert_eq!(original.account_number, parsed.account_number);
        assert_eq!(original.currency, parsed.currency);
        assert_eq!(original.opening_balance, parsed.opening_balance);
        assert_eq!(original.transactions.len(), parsed.transactions.len());
    }

    #[test]
    fn test_parse_empty_csv() {
        let input = "";
        let mut reader = input.as_bytes();
        let result = CsvStatement::from_read(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_transaction_header() {
        let input = "\
account_number,currency,opening_balance,opening_date,opening_indicator,closing_balance,closing_date,closing_indicator
ACC123,USD,1000.0,2024-01-01,Credit,1200.0,2024-01-31,Credit
";
        let mut reader = input.as_bytes();
        let result = CsvStatement::from_read(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_balance_type() {
        let input = "\
account_number,currency,opening_balance,opening_date,opening_indicator,closing_balance,closing_date,closing_indicator
ACC123,USD,1000.0,2024-01-01,INVALID,1200.0,2024-01-31,Credit
booking_date,value_date,amount,transaction_type,description,reference,counterparty_name,counterparty_account
";
        let mut reader = input.as_bytes();
        let result = CsvStatement::from_read(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_transactions() {
        let csv_data = "\
account_number,currency,opening_balance,opening_date,opening_indicator,closing_balance,closing_date,closing_indicator
ACC999,RUB,10000.0,2024-01-01,Credit,12000.0,2024-12-31,Credit
booking_date,value_date,amount,transaction_type,description,reference,counterparty_name,counterparty_account
2024-01-15,2024-01-15,1000.0,Credit,Payment 1,REF1,Alice,ACC1
2024-01-20,2024-01-20,500.0,Debit,Payment 2,REF2,Bob,ACC2
2024-01-25,,750.0,Credit,Payment 3,,,
";

        let mut reader = csv_data.as_bytes();
        let result = CsvStatement::from_read(&mut reader);

        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement.transactions.len(), 3);
        assert_eq!(statement.transactions[0].amount, 1000.0);
        assert_eq!(statement.transactions[1].amount, 500.0);
        assert_eq!(statement.transactions[2].amount, 750.0);
        assert_eq!(statement.transactions[2].value_date, None);
    }

    #[test]
    fn test_format_balance_type() {
        assert_eq!(
            CsvStatement::format_balance_type(&BalanceType::Credit),
            "Credit"
        );
        assert_eq!(
            CsvStatement::format_balance_type(&BalanceType::Debit),
            "Debit"
        );
    }

    #[test]
    fn test_parse_balance_type() {
        assert_eq!(
            CsvStatement::parse_balance_type("Credit").unwrap(),
            BalanceType::Credit
        );
        assert_eq!(
            CsvStatement::parse_balance_type("Debit").unwrap(),
            BalanceType::Debit
        );
        assert!(CsvStatement::parse_balance_type("Invalid").is_err());
    }
}
