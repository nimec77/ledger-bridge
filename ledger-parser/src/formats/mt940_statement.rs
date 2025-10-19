use crate::{BalanceType, ParseError, Transaction, TransactionType};
use chrono::{DateTime, FixedOffset, NaiveDate, Offset, Utc};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// MT940 SWIFT message structure.
///
/// Parses from and writes to MT940 format using manual tag-based parsing.
/// Fields are identical to CsvStatement/Camt053 for seamless conversions.
///
/// This implementation handles the SWIFT MT940 format with:
/// - Block structure (`{1:...}{2:...}{4:...}`)
/// - Tag-based fields (`:20:`, `:25:`, `:60F:`, `:61:`, `:86:`, `:62F:`)
/// - YYMMDD date format with century inference
/// - Multi-line `:86:` fields
/// - Both comma and dot as decimal separators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mt940Statement {
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

impl Mt940Statement {
    /// Parse MT940 from any Read source (file, stdin, buffer).
    ///
    /// Handles both full SWIFT format with blocks and simplified tag-only format.
    ///
    /// # Errors
    ///
    /// Returns `ParseError::Mt940Error` if:
    /// - The MT940 structure is invalid
    /// - Required tags are missing
    /// - Field values cannot be parsed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ledger_parser::Mt940Statement;
    /// use std::fs::File;
    ///
    /// let mut file = File::open("statement.mt940").unwrap();
    /// let statement = Mt940Statement::from_read(&mut file).unwrap();
    /// ```
    pub fn from_read<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // Read entire content
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        if content.trim().is_empty() {
            return Err(ParseError::Mt940Error("Empty input".into()));
        }

        // Extract Block 4 (contains actual data)
        let block4 = Self::extract_block4(&content)?;

        // Parse tags from Block 4
        let tags = Self::parse_tags(&block4)?;

        // Extract required fields
        let account_number = Self::extract_account_number(&tags)?;
        let (opening_balance, opening_date, opening_indicator, currency) =
            Self::extract_opening_balance(&tags)?;
        let (closing_balance, closing_date, closing_indicator) =
            Self::extract_closing_balance(&tags, &currency)?;
        let transactions = Self::extract_transactions(&tags, &currency)?;

        Ok(Mt940Statement {
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

    /// Write MT940 to any Write destination (file, stdout, buffer).
    ///
    /// # Errors
    ///
    /// Returns `ParseError::Mt940Error` if writing fails.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), ParseError> {
        // Write simplified MT940 format (Block 4 only with proper envelope)
        writeln!(
            writer,
            "{{1:F01BANKXXXXXX0000000000}}{{2:I940BANKXXXXXXN}}{{4:"
        )?;
        writeln!(writer, ":20:STATEMENT")?;
        writeln!(writer, ":25:{}", self.account_number)?;
        writeln!(writer, ":28C:1/1")?;

        // Opening balance
        let opening_indicator_char = match self.opening_indicator {
            BalanceType::Credit => 'C',
            BalanceType::Debit => 'D',
        };
        writeln!(
            writer,
            ":60F:{}{}{}{}",
            opening_indicator_char,
            Self::format_yymmdd(&self.opening_date),
            self.currency,
            Self::format_amount(self.opening_balance)
        )?;

        // Transactions
        for tx in &self.transactions {
            let tx_indicator = match tx.transaction_type {
                TransactionType::Credit => 'C',
                TransactionType::Debit => 'D',
            };

            writeln!(
                writer,
                ":61:{}{}{}NTRF{}",
                Self::format_yymmdd(&tx.booking_date),
                tx_indicator,
                Self::format_amount(tx.amount),
                tx.reference.as_ref().unwrap_or(&String::new())
            )?;

            // Description in :86: field
            writeln!(writer, ":86:{}", tx.description)?;
        }

        // Closing balance
        let closing_indicator_char = match self.closing_indicator {
            BalanceType::Credit => 'C',
            BalanceType::Debit => 'D',
        };
        writeln!(
            writer,
            ":62F:{}{}{}{}",
            closing_indicator_char,
            Self::format_yymmdd(&self.closing_date),
            self.currency,
            Self::format_amount(self.closing_balance)
        )?;

        writeln!(writer, "-}}")?;

        Ok(())
    }

    /// Extract Block 4 from MT940 content
    fn extract_block4(content: &str) -> Result<String, ParseError> {
        // Look for {4: ... -} or {4: ... }
        if let Some(start) = content.find("{4:") {
            let after_start = &content[start + 3..];

            // Find end marker (-} or })
            let end = after_start
                .find("-}")
                .or_else(|| after_start.find('}'))
                .ok_or_else(|| ParseError::Mt940Error("Block 4 not properly closed".into()))?;

            return Ok(after_start[..end].into());
        }

        // If no block structure, assume entire content is Block 4 data
        Ok(content.into())
    }

    /// Parse tags from Block 4 content
    fn parse_tags(block4: &str) -> Result<Vec<(String, String)>, ParseError> {
        let mut tags = Vec::new();
        let lines: Vec<&str> = block4.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines
            if line.is_empty() {
                i += 1;
                continue;
            }

            // Check if line starts with tag (colon followed by digits/letters and colon)
            if let Some(stripped) = line.strip_prefix(':') {
                if let Some(second_colon) = stripped.find(':') {
                    let tag = &stripped[..second_colon];
                    let value = &stripped[second_colon + 1..];

                    // Collect multi-line values (lines without leading colon are continuations)
                    let mut full_value: String = value.into();
                    i += 1;

                    while i < lines.len() {
                        let next_line = lines[i];
                        if next_line.trim().starts_with(':') {
                            break;
                        }
                        full_value.push('\n');
                        full_value.push_str(next_line);
                        i += 1;
                    }

                    tags.push((tag.into(), full_value));
                    continue;
                }
            }

            i += 1;
        }

        Ok(tags)
    }

    /// Extract account number from :25: tag
    fn extract_account_number(tags: &[(String, String)]) -> Result<String, ParseError> {
        tags.iter()
            .find(|(tag, _)| tag == "25")
            .map(|(_, value)| value.trim().into())
            .ok_or_else(|| ParseError::Mt940Error("Missing :25: account tag".into()))
    }

    /// Extract opening balance from :60F: or :60M: tag
    fn extract_opening_balance(
        tags: &[(String, String)],
    ) -> Result<(f64, DateTime<FixedOffset>, BalanceType, String), ParseError> {
        let balance_tag = tags
            .iter()
            .find(|(tag, _)| tag == "60F" || tag == "60M")
            .ok_or_else(|| ParseError::Mt940Error("Missing :60F: or :60M: tag".into()))?;

        Self::parse_balance_line(&balance_tag.1)
    }

    /// Extract closing balance from :62F: or :62M: tag
    fn extract_closing_balance(
        tags: &[(String, String)],
        _currency: &str,
    ) -> Result<(f64, DateTime<FixedOffset>, BalanceType), ParseError> {
        let balance_tag = tags
            .iter()
            .find(|(tag, _)| tag == "62F" || tag == "62M")
            .ok_or_else(|| ParseError::Mt940Error("Missing :62F: or :62M: tag".into()))?;

        let (amount, date, indicator, _) = Self::parse_balance_line(&balance_tag.1)?;
        Ok((amount, date, indicator))
    }

    /// Parse balance line format: C/D + YYMMDD + CCY + amount
    /// Example: C200101EUR444,29
    fn parse_balance_line(
        line: &str,
    ) -> Result<(f64, DateTime<FixedOffset>, BalanceType, String), ParseError> {
        let line = line.trim();

        if line.is_empty() {
            return Err(ParseError::Mt940Error("Empty balance line".into()));
        }

        // First char is C or D
        let indicator = match line.chars().next() {
            Some('C') => BalanceType::Credit,
            Some('D') => BalanceType::Debit,
            _ => return Err(ParseError::Mt940Error("Invalid balance indicator".into())),
        };

        let rest = &line[1..];

        // Next 6 chars are date (YYMMDD)
        if rest.len() < 6 {
            return Err(ParseError::Mt940Error("Balance line too short".into()));
        }

        let date_str = &rest[..6];
        let date = Self::parse_yymmdd_date(date_str)?;

        let rest = &rest[6..];

        // Next 3 chars are currency
        if rest.len() < 3 {
            return Err(ParseError::Mt940Error("Missing currency in balance".into()));
        }

        let currency = rest[..3].into();
        let amount_str = &rest[3..];

        let amount = Self::parse_amount(amount_str)?;

        Ok((amount, date, indicator, currency))
    }

    /// Extract transactions from :61: and :86: tag pairs
    fn extract_transactions(
        tags: &[(String, String)],
        _currency: &str,
    ) -> Result<Vec<Transaction>, ParseError> {
        let mut transactions = Vec::new();
        let mut i = 0;

        while i < tags.len() {
            if tags[i].0 == "61" {
                let transaction_line = &tags[i].1;

                // Look for following :86: tag (description)
                let description = if i + 1 < tags.len() && tags[i + 1].0 == "86" {
                    tags[i + 1].1.trim().into()
                } else {
                    String::new()
                };

                if let Ok(tx) = Self::parse_transaction_line(transaction_line, &description) {
                    transactions.push(tx);
                }
            }
            i += 1;
        }

        Ok(transactions)
    }

    /// Parse transaction line (:61:)
    /// Format: YYMMDD[MMDD]C/D[amount][type][reference]
    /// Example: 2001010101D65,00NOVBNL47INGB9999999999
    fn parse_transaction_line(line: &str, description: &str) -> Result<Transaction, ParseError> {
        let line = line.trim();

        if line.is_empty() {
            return Err(ParseError::Mt940Error("Empty transaction line".into()));
        }

        // Parse date (first 6 chars = YYMMDD)
        if line.len() < 6 {
            return Err(ParseError::Mt940Error("Transaction line too short".into()));
        }

        let date_str = &line[..6];
        let booking_date = Self::parse_yymmdd_date(date_str)?;

        let mut rest = &line[6..];

        // Optional booking date (MMDD) - skip if present
        if rest.len() >= 4 && rest[..4].chars().all(|c| c.is_ascii_digit()) {
            rest = &rest[4..];
        }

        // Next char is C or D
        if rest.is_empty() {
            return Err(ParseError::Mt940Error(
                "Missing transaction indicator".into(),
            ));
        }

        let transaction_type = match rest.chars().next() {
            Some('C') => TransactionType::Credit,
            Some('D') => TransactionType::Debit,
            _ => {
                return Err(ParseError::Mt940Error(
                    "Invalid transaction indicator".into(),
                ))
            }
        };

        rest = &rest[1..];

        // Parse amount (find first non-digit, non-comma, non-dot char)
        let amount_end = rest
            .find(|c: char| !c.is_ascii_digit() && c != ',' && c != '.')
            .unwrap_or(rest.len());

        if amount_end == 0 {
            return Err(ParseError::Mt940Error(
                "Missing amount in transaction".into(),
            ));
        }

        let amount_str = &rest[..amount_end];
        let amount = Self::parse_amount(amount_str)?;

        // Rest is transaction type code and reference (variable format)
        let reference = if amount_end < rest.len() {
            Some(rest[amount_end..].trim().into())
        } else {
            None
        };

        Ok(Transaction {
            booking_date,
            value_date: None,
            amount,
            transaction_type,
            description: description.into(),
            reference,
            counterparty_name: None,
            counterparty_account: None,
        })
    }

    /// Parse YYMMDD date with century inference
    /// 00-49 → 2000-2049, 50-99 → 1950-1999
    fn parse_yymmdd_date(date_str: &str) -> Result<DateTime<FixedOffset>, ParseError> {
        if date_str.len() != 6 || !date_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(ParseError::Mt940Error(format!(
                "Expected YYMMDD date, found '{}'",
                date_str
            )));
        }

        let year_part = &date_str[..2];
        let month_part = &date_str[2..4];
        let day_part = &date_str[4..];

        let yy: i32 = year_part.parse().map_err(|_| {
            ParseError::Mt940Error(format!(
                "Invalid year component in '{}': {}",
                date_str, year_part
            ))
        })?;
        let mm: u32 = month_part.parse().map_err(|_| {
            ParseError::Mt940Error(format!(
                "Invalid month component in '{}': {}",
                date_str, month_part
            ))
        })?;
        let dd: u32 = day_part.parse().map_err(|_| {
            ParseError::Mt940Error(format!(
                "Invalid day component in '{}': {}",
                date_str, day_part
            ))
        })?;

        let year = match yy {
            0..=49 => 2000 + yy,
            50..=99 => 1900 + yy,
            _ => {
                return Err(ParseError::Mt940Error(format!(
                    "Year component must be two digits in '{}': {}",
                    date_str, year_part
                )))
            }
        };

        let date = NaiveDate::from_ymd_opt(year, mm, dd).ok_or_else(|| {
            ParseError::Mt940Error(format!(
                "Invalid calendar date derived from '{}': {:04}-{:02}-{:02}",
                date_str, year, mm, dd
            ))
        })?;

        let datetime = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            ParseError::Mt940Error(format!(
                "Invalid time component derived from '{}': midnight",
                date_str
            ))
        })?;

        Ok(DateTime::<FixedOffset>::from_naive_utc_and_offset(
            datetime,
            Utc.fix(),
        ))
    }

    /// Parse amount (handle both comma and dot as decimal separator)
    fn parse_amount(amount_str: &str) -> Result<f64, ParseError> {
        let trimmed = amount_str.trim();

        if trimmed.is_empty() {
            return Ok(0.0);
        }

        // Replace comma with dot, remove spaces
        let normalized = trimmed.replace(',', ".").replace(' ', "");

        // Handle trailing dot or comma (e.g., "100," means 100.00)
        let normalized = if normalized.ends_with('.') {
            format!("{}00", normalized)
        } else {
            normalized
        };

        normalized
            .parse::<f64>()
            .map_err(|_| ParseError::Mt940Error(format!("Invalid amount: {}", amount_str)))
    }

    /// Format date as YYMMDD
    fn format_yymmdd(date: &DateTime<FixedOffset>) -> String {
        date.format("%y%m%d").to_string()
    }

    /// Format amount with comma as decimal separator
    fn format_amount(amount: f64) -> String {
        format!("{:.2}", amount).replace('.', ",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yymmdd_date() {
        // Test 21st century
        let result = Mt940Statement::parse_yymmdd_date("250218");
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.format("%Y-%m-%d").to_string(), "2025-02-18");

        // Test 20th century
        let result = Mt940Statement::parse_yymmdd_date("950315");
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.format("%Y-%m-%d").to_string(), "1995-03-15");
    }

    #[test]
    fn test_parse_yymmdd_date_century_inference() {
        let result = Mt940Statement::parse_yymmdd_date("230101").expect("Expected successful parse");
        assert_eq!(result.format("%Y-%m-%d").to_string(), "2023-01-01");
    }

    #[test]
    fn test_parse_yymmdd_date_invalid_input() {
        let result = Mt940Statement::parse_yymmdd_date("2A0101");
        assert!(matches!(result, Err(ParseError::Mt940Error(_))));
    }

    #[test]
    fn test_parse_amount_comma() {
        let result = Mt940Statement::parse_amount("1540,50");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1540.50);
    }

    #[test]
    fn test_parse_amount_dot() {
        let result = Mt940Statement::parse_amount("2500.75");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2500.75);
    }

    #[test]
    fn test_parse_amount_trailing_comma() {
        let result = Mt940Statement::parse_amount("100,");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100.00);
    }

    #[test]
    fn test_parse_balance_line() {
        let result = Mt940Statement::parse_balance_line("C200101EUR444,29");
        assert!(result.is_ok());
        let (amount, date, indicator, currency) = result.unwrap();
        assert_eq!(amount, 444.29);
        assert_eq!(date.format("%Y-%m-%d").to_string(), "2020-01-01");
        assert_eq!(indicator, BalanceType::Credit);
        assert_eq!(currency, "EUR");
    }

    #[test]
    fn test_parse_balance_line_debit() {
        let result = Mt940Statement::parse_balance_line("D110707CHF100,");
        assert!(result.is_ok());
        let (amount, date, indicator, currency) = result.unwrap();
        assert_eq!(amount, 100.00);
        assert_eq!(date.format("%Y-%m-%d").to_string(), "2011-07-07");
        assert_eq!(indicator, BalanceType::Debit);
        assert_eq!(currency, "CHF");
    }

    #[test]
    fn test_parse_transaction_line() {
        let result = Mt940Statement::parse_transaction_line(
            "2001010101D65,00NOVBNL47INGB9999999999",
            "Betaling sieraden",
        );
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.amount, 65.00);
        assert_eq!(tx.transaction_type, TransactionType::Debit);
        assert_eq!(tx.description, "Betaling sieraden");
        assert_eq!(tx.booking_date.format("%Y-%m-%d").to_string(), "2020-01-01");
    }

    #[test]
    fn test_parse_empty_mt940() {
        let input = "";
        let mut reader = input.as_bytes();
        let result = Mt940Statement::from_read(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_block4() {
        let input = "{1:F01TEST}{2:I940}{4:\n:20:REF\n:25:ACC123\n-}";
        let result = Mt940Statement::extract_block4(input);
        assert!(result.is_ok());
        let block4 = result.unwrap();
        assert!(block4.contains(":20:REF"));
        assert!(block4.contains(":25:ACC123"));
    }

    #[test]
    fn test_parse_real_mt940_gs() {
        use std::fs::File;
        use std::path::PathBuf;

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../example_files/mt 940 gs.mt940");

        if let Ok(mut file) = File::open(&path) {
            let result = Mt940Statement::from_read(&mut file);

            match result {
                Ok(statement) => {
                    println!("✓ Parsed Goldman Sachs MT940");
                    println!("✓ Account: {}", statement.account_number);
                    println!("✓ Currency: {}", statement.currency);
                    println!("✓ Transactions: {}", statement.transactions.len());

                    assert_eq!(statement.account_number, "107048825");
                    assert_eq!(statement.currency, "USD");
                    assert!(!statement.transactions.is_empty());
                }
                Err(e) => {
                    panic!("Failed to parse Goldman Sachs MT940: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_parse_real_mt940_asn() {
        use std::fs::File;
        use std::path::PathBuf;

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../example_files/MT940 github 1.mt940");

        if let Ok(mut file) = File::open(&path) {
            let result = Mt940Statement::from_read(&mut file);

            match result {
                Ok(statement) => {
                    println!("✓ Parsed ASN Bank MT940");
                    println!("✓ Account: {}", statement.account_number);
                    println!("✓ Currency: {}", statement.currency);
                    println!("✓ Transactions: {}", statement.transactions.len());

                    assert!(statement.account_number.contains("ASNB"));
                    assert_eq!(statement.currency, "EUR");
                }
                Err(e) => {
                    panic!("Failed to parse ASN Bank MT940: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_mt940_write() {
        let statement = Mt940Statement {
            account_number: "NL81ASNB9999999999".into(),
            currency: "EUR".into(),
            opening_balance: 444.29,
            opening_date: Mt940Statement::parse_yymmdd_date("200101").unwrap(),
            opening_indicator: BalanceType::Credit,
            closing_balance: 379.29,
            closing_date: Mt940Statement::parse_yymmdd_date("200101").unwrap(),
            closing_indicator: BalanceType::Credit,
            transactions: vec![],
        };

        let mut output = Vec::new();
        let result = statement.write_to(&mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains(":25:NL81ASNB9999999999"));
        assert!(output_str.contains(":60F:C200101EUR444,29"));
        assert!(output_str.contains(":62F:C200101EUR379,29"));
    }
}
