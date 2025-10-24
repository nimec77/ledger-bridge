use crate::formats::cvs_const::*;
use crate::formats::formats_const::*;
use crate::{formats::utils, BalanceType, ParseError, Transaction, TransactionType};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// CSV bank statement structure.
///
/// Parses from and writes to CSV format using the `csv` crate.
/// Fields are identical to Mt940/Camt053 for seamless conversions.
///
/// This implementation handles the Russian Sberbank CSV format with:
/// - Multi-line header section (metadata)
/// - Transaction rows with separate debit/credit columns
/// - Multi-line cells (account information)
/// - Footer section with balance information
/// - Russian text and comma decimal separators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsvStatement {
    /// Account number (IBAN or local format) from the bank statement
    pub account_number: String,
    /// Three-letter ISO 4217 currency code (e.g., USD, EUR, RUB)
    pub currency: String,
    /// Opening balance amount at the start of the statement period
    pub opening_balance: f64,
    /// Date and time of the opening balance
    pub opening_date: DateTime<FixedOffset>,
    /// Opening balance type (Credit or Debit indicator)
    pub opening_indicator: BalanceType,
    /// Closing balance amount at the end of the statement period
    pub closing_balance: f64,
    /// Date and time of the closing balance
    pub closing_date: DateTime<FixedOffset>,
    /// Closing balance type (Credit or Debit indicator)
    pub closing_indicator: BalanceType,
    /// List of transactions in chronological order
    pub transactions: Vec<Transaction>,
}

impl CsvStatement {
    /// Parse CSV from any Read source (file, stdin, buffer).
    ///
    /// Handles the Russian Sberbank CSV format:
    /// - Header section (lines 1-12): Metadata and column headers
    /// - Transaction section (lines 13+): Transaction rows
    /// - Footer section: Balance information
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
    /// ```no_run
    /// use ledger_parser::CsvStatement;
    /// use std::fs::File;
    ///
    /// let mut file = File::open("statement.csv").unwrap();
    /// let statement = CsvStatement::from_read(&mut file).unwrap();
    /// ```
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Read entire content - needed because multi-line cells complicate streaming
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        if content.is_empty() {
            return Err(ParseError::CsvError(ERROR_EMPTY_INPUT.into()));
        }

        // Use csv crate with flexible parsing options
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(false) // We'll handle headers manually
            .flexible(true) // Allow variable column counts
            .from_reader(content.as_bytes());

        // Collect all records
        let records: Vec<csv::StringRecord> =
            csv_reader.records().collect::<Result<Vec<_>, _>>()?;

        if records.len() < MIN_CSV_LINES {
            return Err(ParseError::CsvError(ERROR_CSV_TOO_SHORT.into()));
        }

        // Extract account number from header (line 6, column 12)
        let account_number = Self::extract_account_number(&records)?;

        // Extract currency from header (line 9, column 2)
        let currency = Self::extract_currency(&records)?;

        // Find transaction section and footer
        let (transaction_start, footer_start) = Self::find_sections(&records)?;

        // Parse transactions
        let transactions = Self::parse_transactions(&records, transaction_start, footer_start)?;

        // Extract balances from footer
        let (opening_balance, opening_date, opening_indicator) =
            Self::extract_opening_balance(&records, footer_start)?;
        let (closing_balance, closing_date, closing_indicator) =
            Self::extract_closing_balance(&records, footer_start)?;

        Ok(CsvStatement {
            account_number,
            currency,
            opening_balance,
            opening_date,
            opening_indicator,
            closing_balance,
            closing_date,
            closing_indicator,
            transactions,
        })
    }

    /// Write CSV to any Write destination (file, stdout, buffer).
    ///
    /// Outputs in Russian Sberbank CSV format.
    ///
    /// # Errors
    ///
    /// Returns `ParseError::CsvError` if writing fails.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        let mut csv_writer = csv::WriterBuilder::new()
            .flexible(true) // Allow records with varying field counts
            .from_writer(writer);

        // Write header section
        Self::write_header(&mut csv_writer, &self.account_number, &self.currency)?;

        // Write transaction section
        Self::write_transactions(&mut csv_writer, &self.transactions)?;

        // Write footer section
        Self::write_footer(
            &mut csv_writer,
            self.opening_balance,
            &self.opening_date,
            &self.opening_indicator,
            self.closing_balance,
            &self.closing_date,
            &self.closing_indicator,
            &self.transactions,
        )?;

        csv_writer.flush()?;
        Ok(())
    }

    /// Extract account number from header section
    fn extract_account_number(records: &[csv::StringRecord]) -> Result<String, ParseError> {
        if records.len() <= MIN_LINES_FOR_ACCOUNT {
            return Err(ParseError::CsvError(ERROR_MISSING_ACCOUNT.into()));
        }

        // Search in first 10 lines for 20-digit account number
        for record in &records[0..records.len().min(MAX_ACCOUNT_SEARCH_LINES)] {
            for field in record.iter() {
                let trimmed = field.trim();
                // Account number format: typically 20 digits
                if trimmed.len() == ACCOUNT_NUMBER_LENGTH
                    && trimmed.chars().all(|c| c.is_ascii_digit())
                {
                    return Ok(trimmed.into());
                }
            }
        }

        Err(ParseError::CsvError(ERROR_ACCOUNT_NOT_FOUND.into()))
    }

    /// Extract currency from header section
    fn extract_currency(records: &[csv::StringRecord]) -> Result<String, ParseError> {
        let record = records
            .get(CURRENCY_LINE_INDEX)
            .ok_or_else(|| ParseError::CsvError(ERROR_MISSING_CURRENCY.into()))?;

        // Currency is in line 9 (index 8), look for "Российский рубль" or currency code
        for field in record.iter() {
            let trimmed = field.trim().to_lowercase();
            if trimmed.contains(RUSSIAN_RUBLE_FULL) || trimmed.contains(RUSSIAN_RUBLE_SHORT) {
                return Ok(CURRENCY_RUB.into());
            }
            if trimmed.contains(RUSSIAN_DOLLAR) || trimmed.contains("usd") {
                return Ok(CURRENCY_USD.into());
            }
            if trimmed.contains(RUSSIAN_EURO) || trimmed.contains("eur") {
                return Ok(CURRENCY_EUR.into());
            }
        }

        // Default to RUB if not found
        Ok(CURRENCY_RUB.into())
    }

    /// Find transaction start and footer start positions
    fn find_sections(records: &[csv::StringRecord]) -> Result<(usize, usize), ParseError> {
        // Transaction section starts after "Дата проводки" header (typically line 11-12)
        let mut transaction_start = None;
        for (i, record) in records.iter().enumerate() {
            if record
                .iter()
                .any(|f| f.to_lowercase().contains(TRANSACTION_DATE_HEADER))
            {
                // Skip header row and sub-header row
                transaction_start = Some(i + TRANSACTION_HEADER_SKIP_LINES);
                break;
            }
        }

        let transaction_start = transaction_start
            .ok_or_else(|| ParseError::CsvError(ERROR_TRANSACTION_SECTION_NOT_FOUND.into()))?;

        // Footer starts at "б/с" marker
        let mut footer_start = records.len();
        for (i, record) in records.iter().enumerate().skip(transaction_start) {
            if record
                .iter()
                .any(|f| f.to_lowercase().contains(BALANCE_SHEET_MARKER))
            {
                footer_start = i;
                break;
            }
        }

        Ok((transaction_start, footer_start))
    }

    /// Parse transaction rows
    fn parse_transactions(
        records: &[csv::StringRecord],
        start: usize,
        end: usize,
    ) -> Result<Vec<Transaction>, ParseError> {
        let mut transactions = Vec::new();

        for record in &records[start..end] {
            // Skip empty rows
            if record.iter().all(|f| f.trim().is_empty()) {
                continue;
            }

            // Try to parse as transaction
            if let Ok(transaction) = Self::parse_transaction_record(record) {
                transactions.push(transaction);
            }
        }

        Ok(transactions)
    }

    /// Parse a single transaction record
    fn parse_transaction_record(record: &csv::StringRecord) -> Result<Transaction, ParseError> {
        // Get field values by index
        let get_field =
            |idx: usize| -> String { record.get(idx).map(|s| s.trim().into()).unwrap_or_default() };

        // Extract date (column 1, index 1)
        let date_str = get_field(DATE_COLUMN_INDEX);
        if date_str.is_empty() {
            return Err(ParseError::CsvError(ERROR_EMPTY_DATE_FIELD.into()));
        }
        let booking_date = Self::parse_date(&date_str)?;

        // Extract debit amount (column 9, around index 9)
        let debit_str = get_field(DEBIT_AMOUNT_COLUMN_INDEX);
        let debit_amount = Self::parse_amount(&debit_str)?;

        // Extract credit amount (column 13, around index 13)
        let credit_str = get_field(CREDIT_AMOUNT_COLUMN_INDEX);
        let credit_amount = Self::parse_amount(&credit_str)?;

        // Determine transaction type and amount
        let (amount, transaction_type) = if debit_amount > 0.0 {
            (debit_amount, TransactionType::Debit)
        } else if credit_amount > 0.0 {
            (credit_amount, TransactionType::Credit)
        } else {
            return Err(ParseError::CsvError(ERROR_NO_TRANSACTION_AMOUNT.into()));
        };

        // Extract document number (around index 14)
        let reference_str = get_field(REFERENCE_COLUMN_INDEX);
        let reference = if reference_str.is_empty() {
            None
        } else {
            Some(reference_str)
        };

        // Extract description (around index 20 or later)
        let mut description = String::new();
        for i in DESCRIPTION_SEARCH_START_INDEX..record.len() {
            let field = get_field(i);
            if !field.is_empty() {
                description = field;
                break;
            }
        }

        Ok(Transaction {
            booking_date,
            value_date: None, // Not available in this format
            amount,
            transaction_type,
            description,
            reference,
            counterparty_name: None,    // Could extract from account field
            counterparty_account: None, // Could extract from account field
        })
    }

    /// Parse date format (comma as decimal separator)
    fn parse_date(date_str: &str) -> Result<DateTime<FixedOffset>, ParseError> {
        utils::parse_date(date_str)
            .map_err(|_| ParseError::CsvError(format!("Invalid date: {}", date_str)))
    }

    /// Parse amount format (comma as decimal separator)
    fn parse_amount(amount_str: &str) -> Result<f64, ParseError> {
        utils::parse_amount(amount_str)
            .map_err(|_| ParseError::CsvError(format!("Invalid amount: {}", amount_str)))
    }

    /// Extract opening balance from footer section
    fn extract_opening_balance(
        records: &[csv::StringRecord],
        footer_start: usize,
    ) -> Result<(f64, DateTime<FixedOffset>, BalanceType), ParseError> {
        // Look for "Входящий остаток" in footer
        for record in &records[footer_start..] {
            for (i, field) in record.iter().enumerate() {
                if field.to_lowercase().contains(OPENING_BALANCE_LABEL) {
                    // Amount is typically a few columns later - skip zeros
                    for offset in 1..MAX_BALANCE_SEARCH_OFFSET {
                        if let Some(amount_field) = record.get(i + offset) {
                            if let Ok(amount) = Self::parse_amount(amount_field) {
                                // Skip zero amounts - find the actual balance
                                if amount.abs() < MIN_AMOUNT_THRESHOLD {
                                    continue;
                                }

                                let indicator = if amount >= 0.0 {
                                    BalanceType::Credit
                                } else {
                                    BalanceType::Debit
                                };

                                // Try to extract date (often at end of row)
                                let date =
                                    Self::parse_date(&Self::extract_date_from_record(record)?)?;

                                return Ok((amount.abs(), date, indicator));
                            }
                        }
                    }
                }
            }
        }

        Err(ParseError::CsvError(ERROR_OPENING_BALANCE_NOT_FOUND.into()))
    }

    /// Extract closing balance from footer section
    fn extract_closing_balance(
        records: &[csv::StringRecord],
        footer_start: usize,
    ) -> Result<(f64, DateTime<FixedOffset>, BalanceType), ParseError> {
        // Look for "Исходящий остаток" in footer
        for record in &records[footer_start..] {
            for (i, field) in record.iter().enumerate() {
                if field.to_lowercase().contains(CLOSING_BALANCE_LABEL) {
                    // Amount is typically a few columns later - skip zeros
                    for offset in 1..MAX_BALANCE_SEARCH_OFFSET {
                        if let Some(amount_field) = record.get(i + offset) {
                            if let Ok(amount) = Self::parse_amount(amount_field) {
                                // Skip zero amounts - find the actual balance
                                if amount.abs() < MIN_AMOUNT_THRESHOLD {
                                    continue;
                                }

                                let indicator = if amount >= 0.0 {
                                    BalanceType::Credit
                                } else {
                                    BalanceType::Debit
                                };

                                // Try to extract date (often at end of row)
                                let date_str = Self::extract_date_from_record(record)?;

                                return Ok((amount.abs(), Self::parse_date(&date_str)?, indicator));
                            }
                        }
                    }
                }
            }
        }

        Err(ParseError::CsvError(ERROR_CLOSING_BALANCE_NOT_FOUND.into()))
    }

    /// Extract date from a record (looks for date patterns)
    fn extract_date_from_record(record: &csv::StringRecord) -> Result<String, ParseError> {
        for field in record.iter().rev() {
            let trimmed = field.trim();
            // Look for Russian date format like "01 января 2024 г."
            if trimmed.to_lowercase().contains(RUSSIAN_YEAR_SUFFIX)
                && trimmed.len() > MIN_DATE_STRING_LENGTH
            {
                // Extract year
                if let Some(year_pos) = trimmed.rfind(|c: char| c.is_ascii_digit()) {
                    let year_start = year_pos.saturating_sub(YEAR_EXTRACTION_OFFSET);
                    if let Some(year_str) = trimmed.get(year_start..=year_pos) {
                        if let Ok(year) = year_str.parse::<u32>() {
                            if (MIN_VALID_YEAR..=MAX_VALID_YEAR).contains(&year) {
                                // For now, return a simple date - full parsing would require month name mapping
                                return Ok(format!("{}-01-01", year));
                            }
                        }
                    }
                }
            }
        }
        Err(ParseError::CsvError(ERROR_DATE_NOT_FOUND.into()))
    }

    /// Write header section
    fn write_header<W: Write>(
        csv_writer: &mut csv::Writer<W>,
        account_number: &str,
        currency: &str,
    ) -> Result<(), ParseError> {
        // Write simplified header for output
        csv_writer.write_record(["", BANK_NAME_SBERBUSINESS])?;
        csv_writer.write_record(["", BANK_NAME_FULL])?;
        csv_writer.write_record(["", ""])?;
        csv_writer.write_record([
            "",
            STATEMENT_TITLE,
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            account_number,
        ])?;
        csv_writer.write_record(["", "", currency])?;
        csv_writer.write_record([""])?;

        // Column headers
        csv_writer.write_record([
            "",
            COLUMN_TRANSACTION_DATE,
            "",
            "",
            COLUMN_ACCOUNT,
            "",
            "",
            "",
            "",
            COLUMN_DEBIT_AMOUNT,
            "",
            "",
            "",
            COLUMN_CREDIT_AMOUNT,
            COLUMN_DOCUMENT_NUMBER,
            "",
            COLUMN_VO,
            COLUMN_BANK,
            "",
            "",
            COLUMN_PAYMENT_PURPOSE,
        ])?;
        csv_writer.write_record(["", "", "", "", LABEL_DEBIT, "", "", "", LABEL_CREDIT])?;

        Ok(())
    }

    /// Write transaction rows
    fn write_transactions<W: Write>(
        csv_writer: &mut csv::Writer<W>,
        transactions: &[Transaction],
    ) -> Result<(), ParseError> {
        for tx in transactions {
            let mut row = vec![String::new(); OUTPUT_ROW_COLUMNS];
            let booking_date = tx.booking_date;

            row[1] = booking_date.format("%d.%m.%Y").to_string();

            match tx.transaction_type {
                TransactionType::Debit => {
                    row[DEBIT_AMOUNT_COLUMN_INDEX] = format!("{:.2}", tx.amount)
                        .replace(DECIMAL_SEPARATOR_DOT, DECIMAL_SEPARATOR_COMMA);
                }
                TransactionType::Credit => {
                    row[CREDIT_AMOUNT_COLUMN_INDEX] = format!("{:.2}", tx.amount)
                        .replace(DECIMAL_SEPARATOR_DOT, DECIMAL_SEPARATOR_COMMA);
                }
            }

            if let Some(ref reference) = tx.reference {
                row[REFERENCE_COLUMN_INDEX] = reference.clone();
            }

            row[DESCRIPTION_COLUMN_INDEX] = tx.description.clone();

            csv_writer.write_record(&row)?;
        }

        Ok(())
    }

    /// Write footer section
    #[allow(clippy::too_many_arguments)]
    fn write_footer<W: Write>(
        csv_writer: &mut csv::Writer<W>,
        opening_balance: f64,
        opening_date: &DateTime<FixedOffset>,
        opening_indicator: &BalanceType,
        closing_balance: f64,
        closing_date: &DateTime<FixedOffset>,
        closing_indicator: &BalanceType,
        transactions: &[Transaction],
    ) -> Result<(), ParseError> {
        csv_writer.write_record([""])?;
        csv_writer.write_record(["", BALANCE_SHEET_MARKER])?;

        let debit_count = transactions
            .iter()
            .filter(|t| t.transaction_type == TransactionType::Debit)
            .count();
        let credit_count = transactions
            .iter()
            .filter(|t| t.transaction_type == TransactionType::Credit)
            .count();

        csv_writer.write_record([
            "",
            FOOTER_OPERATION_COUNT,
            "",
            "",
            "",
            "",
            &debit_count.to_string(),
            "",
            "",
            &credit_count.to_string(),
        ])?;

        let opening_sign = match opening_indicator {
            BalanceType::Credit => POSITIVE_SIGN,
            BalanceType::Debit => NEGATIVE_SIGN,
        };
        csv_writer.write_record([
            "",
            FOOTER_OPENING_BALANCE,
            "",
            "",
            "",
            "",
            &format!("{}{:.2}", opening_sign, opening_balance)
                .replace(DECIMAL_SEPARATOR_DOT, DECIMAL_SEPARATOR_COMMA),
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            &opening_date.format("%d.%m.%Y").to_string(),
        ])?;

        let closing_sign = match closing_indicator {
            BalanceType::Credit => POSITIVE_SIGN,
            BalanceType::Debit => NEGATIVE_SIGN,
        };
        csv_writer.write_record([
            "",
            FOOTER_CLOSING_BALANCE,
            "",
            "",
            "",
            "",
            &format!("{}{:.2}", closing_sign, closing_balance)
                .replace(DECIMAL_SEPARATOR_DOT, DECIMAL_SEPARATOR_COMMA),
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            &closing_date.format("%d.%m.%Y").to_string(),
        ])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let result = CsvStatement::parse_date("20.02.2024");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().format("%d.%m.%Y").to_string(), "20.02.2024");
    }

    #[test]
    fn test_parse_amount() {
        let result = CsvStatement::parse_amount("1540,00");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1540.0);
    }

    #[test]
    fn test_parse_empty_amount() {
        let result = CsvStatement::parse_amount("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_parse_invalid_date() {
        let result = CsvStatement::parse_date("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_amount() {
        let result = CsvStatement::parse_amount("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_csv() {
        let input = "";
        let mut reader = input.as_bytes();
        let result = CsvStatement::from_read(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_csv_statement_creation() {
        let statement = CsvStatement {
            account_number: "40702810440000030888".into(),
            currency: CURRENCY_RUB.into(),
            opening_balance: 1332.54,
            opening_date: CsvStatement::parse_date("2024-01-01").unwrap(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 5975.04,
            closing_date: CsvStatement::parse_date("2024-12-31").unwrap(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![],
        };

        assert_eq!(statement.account_number, "40702810440000030888");
        assert_eq!(statement.currency, CURRENCY_RUB);
    }

    #[test]
    fn test_parse_real_sberbank_csv() {
        use std::fs::File;
        use std::path::PathBuf;

        // Try to load the actual example file
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../example_files/example_of_account_statement.csv");

        if let Ok(mut file) = File::open(&path) {
            let result = CsvStatement::from_read(&mut file);

            match result {
                Ok(statement) => {
                    // Verify account number
                    assert_eq!(statement.account_number, "40702810440000030888");

                    // Verify currency
                    assert_eq!(statement.currency, CURRENCY_RUB);

                    // Verify we parsed transactions
                    assert!(
                        !statement.transactions.is_empty(),
                        "Should have parsed at least one transaction"
                    );

                    // Verify balances exist
                    assert!(statement.opening_balance >= 0.0);
                    assert!(statement.closing_balance >= 0.0);

                    println!("✓ Parsed {} transactions", statement.transactions.len());
                    println!("✓ Account: {}", statement.account_number);
                    println!("✓ Currency: {}", statement.currency);
                    println!(
                        "✓ Opening balance: {:.2} {}",
                        statement.opening_balance, statement.currency
                    );
                    println!(
                        "✓ Closing balance: {:.2} {}",
                        statement.closing_balance, statement.currency
                    );
                }
                Err(e) => {
                    panic!("Failed to parse real CSV file: {}", e);
                }
            }
        } else {
            // Skip test if example file doesn't exist
            println!("Skipping real CSV test - example file not found");
        }
    }
}
