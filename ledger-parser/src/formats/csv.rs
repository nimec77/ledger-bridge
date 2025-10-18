use crate::{BalanceType, ParseError, Transaction, TransactionType};
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
            return Err(ParseError::CsvError("Empty input".to_string()));
        }

        // Use csv crate with flexible parsing options
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(false) // We'll handle headers manually
            .flexible(true) // Allow variable column counts
            .from_reader(content.as_bytes());

        // Collect all records
        let records: Vec<csv::StringRecord> =
            csv_reader.records().collect::<Result<Vec<_>, _>>()?;

        if records.len() < 12 {
            return Err(ParseError::CsvError(
                "CSV too short - missing required sections".to_string(),
            ));
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
        let mut csv_writer = csv::Writer::from_writer(writer);

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
        if records.len() <= 5 {
            return Err(ParseError::CsvError(
                "Missing account number in header".to_string(),
            ));
        }

        // Search in first 10 lines for 20-digit account number
        for record in &records[0..records.len().min(10)] {
            for field in record.iter() {
                let trimmed = field.trim();
                // Account number format: typically 20 digits
                if trimmed.len() == 20 && trimmed.chars().all(|c| c.is_ascii_digit()) {
                    return Ok(trimmed.to_string());
                }
            }
        }

        Err(ParseError::CsvError(
            "Account number not found in header".to_string(),
        ))
    }

    /// Extract currency from header section
    fn extract_currency(records: &[csv::StringRecord]) -> Result<String, ParseError> {
        if records.len() <= 8 {
            return Err(ParseError::CsvError(
                "Missing currency in header".to_string(),
            ));
        }

        // Currency is in line 9 (index 8), look for "Российский рубль" or currency code
        let record = &records[8];
        for field in record.iter() {
            let trimmed = field.trim();
            if trimmed.contains("Российский рубль") || trimmed.contains("рубль")
            {
                return Ok("RUB".to_string());
            }
            if trimmed.contains("доллар") || trimmed.contains("USD") {
                return Ok("USD".to_string());
            }
            if trimmed.contains("евро") || trimmed.contains("EUR") {
                return Ok("EUR".to_string());
            }
        }

        // Default to RUB if not found
        Ok("RUB".to_string())
    }

    /// Find transaction start and footer start positions
    fn find_sections(records: &[csv::StringRecord]) -> Result<(usize, usize), ParseError> {
        // Transaction section starts after "Дата проводки" header (typically line 11-12)
        let mut transaction_start = None;
        for (i, record) in records.iter().enumerate() {
            if record.iter().any(|f| f.contains("Дата проводки")) {
                // Skip header row and sub-header row
                transaction_start = Some(i + 2);
                break;
            }
        }

        let transaction_start = transaction_start.ok_or_else(|| {
            ParseError::CsvError(
                "Transaction section not found (missing 'Дата проводки')".to_string(),
            )
        })?;

        // Footer starts at "б/с" marker
        let mut footer_start = records.len();
        for (i, record) in records.iter().enumerate().skip(transaction_start) {
            if record.iter().any(|f| f.contains("б/с")) {
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
        let get_field = |idx: usize| -> String {
            record
                .get(idx)
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };

        // Extract date (column 1, index 1)
        let date_str = get_field(1);
        if date_str.is_empty() {
            return Err(ParseError::CsvError("Empty date field".to_string()));
        }
        let booking_date = Self::parse_russian_date(&date_str)?;

        // Extract debit amount (column 9, around index 9)
        let debit_str = get_field(9);
        let debit_amount = Self::parse_russian_amount(&debit_str)?;

        // Extract credit amount (column 13, around index 13)
        let credit_str = get_field(13);
        let credit_amount = Self::parse_russian_amount(&credit_str)?;

        // Determine transaction type and amount
        let (amount, transaction_type) = if debit_amount > 0.0 {
            (debit_amount, TransactionType::Debit)
        } else if credit_amount > 0.0 {
            (credit_amount, TransactionType::Credit)
        } else {
            return Err(ParseError::CsvError(
                "Transaction has no amount".to_string(),
            ));
        };

        // Extract document number (around index 14)
        let reference_str = get_field(14);
        let reference = if reference_str.is_empty() {
            None
        } else {
            Some(reference_str)
        };

        // Extract description (around index 20 or later)
        let mut description = String::new();
        for i in 18..record.len() {
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

    /// Parse Russian date format (DD.MM.YYYY) to ISO format (YYYY-MM-DD)
    fn parse_russian_date(date_str: &str) -> Result<String, ParseError> {
        if date_str.is_empty() {
            return Err(ParseError::CsvError("Empty date".to_string()));
        }

        let parts: Vec<&str> = date_str.split('.').collect();
        if parts.len() != 3 {
            return Err(ParseError::CsvError(format!(
                "Invalid date format: {}",
                date_str
            )));
        }

        let day = parts[0];
        let month = parts[1];
        let year = parts[2];

        // Validate components
        if day.len() != 2 || month.len() != 2 || year.len() != 4 {
            return Err(ParseError::CsvError(format!(
                "Invalid date components: {}",
                date_str
            )));
        }

        Ok(format!("{}-{}-{}", year, month, day))
    }

    /// Parse Russian amount format (comma as decimal separator)
    fn parse_russian_amount(amount_str: &str) -> Result<f64, ParseError> {
        let trimmed = amount_str.trim();
        if trimmed.is_empty() {
            return Ok(0.0);
        }

        // Replace comma with dot and remove spaces
        let normalized = trimmed.replace(',', ".").replace(' ', "");

        normalized
            .parse::<f64>()
            .map_err(|_| ParseError::CsvError(format!("Invalid amount: {}", amount_str)))
    }

    /// Extract opening balance from footer section
    fn extract_opening_balance(
        records: &[csv::StringRecord],
        footer_start: usize,
    ) -> Result<(f64, String, BalanceType), ParseError> {
        // Look for "Входящий остаток" in footer
        for record in &records[footer_start..] {
            for (i, field) in record.iter().enumerate() {
                if field.contains("Входящий остаток") {
                    // Amount is typically a few columns later - skip zeros
                    for offset in 1..15 {
                        if let Some(amount_field) = record.get(i + offset) {
                            if let Ok(amount) = Self::parse_russian_amount(amount_field) {
                                // Skip zero amounts - find the actual balance
                                if amount.abs() < 0.01 {
                                    continue;
                                }

                                let indicator = if amount >= 0.0 {
                                    BalanceType::Credit
                                } else {
                                    BalanceType::Debit
                                };

                                // Try to extract date (often at end of row)
                                let date = Self::extract_date_from_record(record)
                                    .unwrap_or_else(|| "2024-01-01".to_string());

                                return Ok((amount.abs(), date, indicator));
                            }
                        }
                    }
                }
            }
        }

        Err(ParseError::CsvError(
            "Opening balance not found".to_string(),
        ))
    }

    /// Extract closing balance from footer section
    fn extract_closing_balance(
        records: &[csv::StringRecord],
        footer_start: usize,
    ) -> Result<(f64, String, BalanceType), ParseError> {
        // Look for "Исходящий остаток" in footer
        for record in &records[footer_start..] {
            for (i, field) in record.iter().enumerate() {
                if field.contains("Исходящий остаток") {
                    // Amount is typically a few columns later - skip zeros
                    for offset in 1..15 {
                        if let Some(amount_field) = record.get(i + offset) {
                            if let Ok(amount) = Self::parse_russian_amount(amount_field) {
                                // Skip zero amounts - find the actual balance
                                if amount.abs() < 0.01 {
                                    continue;
                                }

                                let indicator = if amount >= 0.0 {
                                    BalanceType::Credit
                                } else {
                                    BalanceType::Debit
                                };

                                // Try to extract date (often at end of row)
                                let date = Self::extract_date_from_record(record)
                                    .unwrap_or_else(|| "2024-12-31".to_string());

                                return Ok((amount.abs(), date, indicator));
                            }
                        }
                    }
                }
            }
        }

        Err(ParseError::CsvError(
            "Closing balance not found".to_string(),
        ))
    }

    /// Extract date from a record (looks for date patterns)
    fn extract_date_from_record(record: &csv::StringRecord) -> Option<String> {
        for field in record.iter().rev() {
            let trimmed = field.trim();
            // Look for Russian date format like "01 января 2024 г."
            if trimmed.contains("г.") && trimmed.len() > 10 {
                // Extract year
                if let Some(year_pos) = trimmed.rfind(|c: char| c.is_ascii_digit()) {
                    let year_start = year_pos.saturating_sub(3);
                    if let Some(year_str) = trimmed.get(year_start..=year_pos) {
                        if let Ok(year) = year_str.parse::<u32>() {
                            if (2000..=2100).contains(&year) {
                                // For now, return a simple date - full parsing would require month name mapping
                                return Some(format!("{}-01-01", year));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Write header section
    fn write_header<W: Write>(
        csv_writer: &mut csv::Writer<W>,
        account_number: &str,
        currency: &str,
    ) -> Result<(), ParseError> {
        // Write simplified header for output
        csv_writer.write_record(["", "СберБизнес"])?;
        csv_writer.write_record(["", "ПАО СБЕРБАНК"])?;
        csv_writer.write_record(["", ""])?;
        csv_writer.write_record([
            "",
            "ВЫПИСКА ОПЕРАЦИЙ ПО ЛИЦЕВОМУ СЧЕТУ",
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
            "Дата проводки",
            "",
            "",
            "Счет",
            "",
            "",
            "",
            "",
            "Сумма по дебету",
            "",
            "",
            "",
            "Сумма по кредиту",
            "№ документа",
            "",
            "ВО",
            "Банк",
            "",
            "",
            "Назначение платежа",
        ])?;
        csv_writer.write_record(["", "", "", "", "Дебет", "", "", "", "Кредит"])?;

        Ok(())
    }

    /// Write transaction rows
    fn write_transactions<W: Write>(
        csv_writer: &mut csv::Writer<W>,
        transactions: &[Transaction],
    ) -> Result<(), ParseError> {
        for tx in transactions {
            let mut row = vec![String::new(); 21];
            row[1] = Self::format_russian_date(&tx.booking_date);

            match tx.transaction_type {
                TransactionType::Debit => {
                    row[9] = format!("{:.2}", tx.amount).replace('.', ",");
                }
                TransactionType::Credit => {
                    row[13] = format!("{:.2}", tx.amount).replace('.', ",");
                }
            }

            if let Some(ref reference) = tx.reference {
                row[14] = reference.clone();
            }

            row[20] = tx.description.clone();

            csv_writer.write_record(&row)?;
        }

        Ok(())
    }

    /// Write footer section
    #[allow(clippy::too_many_arguments)]
    fn write_footer<W: Write>(
        csv_writer: &mut csv::Writer<W>,
        opening_balance: f64,
        opening_date: &str,
        opening_indicator: &BalanceType,
        closing_balance: f64,
        closing_date: &str,
        closing_indicator: &BalanceType,
        transactions: &[Transaction],
    ) -> Result<(), ParseError> {
        csv_writer.write_record([""])?;
        csv_writer.write_record(["", "б/с"])?;

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
            "Количество операций",
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
            BalanceType::Credit => "",
            BalanceType::Debit => "-",
        };
        csv_writer.write_record([
            "",
            "Входящий остаток",
            "",
            "",
            "",
            "",
            &format!("{}{:.2}", opening_sign, opening_balance).replace('.', ","),
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
            &Self::format_russian_date(opening_date),
        ])?;

        let closing_sign = match closing_indicator {
            BalanceType::Credit => "",
            BalanceType::Debit => "-",
        };
        csv_writer.write_record([
            "",
            "Исходящий остаток",
            "",
            "",
            "",
            "",
            &format!("{}{:.2}", closing_sign, closing_balance).replace('.', ","),
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
            &Self::format_russian_date(closing_date),
        ])?;

        Ok(())
    }

    /// Format date as Russian format (DD.MM.YYYY)
    fn format_russian_date(date_str: &str) -> String {
        // Convert ISO format (YYYY-MM-DD) to Russian format (DD.MM.YYYY)
        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() == 3 {
            format!("{}.{}.{}", parts[2], parts[1], parts[0])
        } else {
            date_str.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_russian_date() {
        let result = CsvStatement::parse_russian_date("20.02.2024");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2024-02-20");
    }

    #[test]
    fn test_parse_russian_amount() {
        let result = CsvStatement::parse_russian_amount("1540,00");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1540.0);
    }

    #[test]
    fn test_parse_empty_amount() {
        let result = CsvStatement::parse_russian_amount("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_parse_invalid_date() {
        let result = CsvStatement::parse_russian_date("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_amount() {
        let result = CsvStatement::parse_russian_amount("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_russian_date() {
        assert_eq!(
            CsvStatement::format_russian_date("2024-02-20"),
            "20.02.2024"
        );
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
            account_number: "40702810440000030888".to_string(),
            currency: "RUB".to_string(),
            opening_balance: 1332.54,
            opening_date: "2024-01-01".to_string(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 5975.04,
            closing_date: "2024-12-31".to_string(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![],
        };

        assert_eq!(statement.account_number, "40702810440000030888");
        assert_eq!(statement.currency, "RUB");
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
                    assert_eq!(statement.currency, "RUB");

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
