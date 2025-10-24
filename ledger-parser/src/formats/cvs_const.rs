/// Constants for CSV statement parsing and formatting.
///
/// This module contains all magic numbers used in the Russian Sberbank CSV format
/// to improve code maintainability, readability, and documentation.
///
/// The constants are organized by their purpose:
/// - CSV structure validation
/// - Column index mappings
/// - Balance extraction parameters
/// - Date parsing constraints
///
/// ## CSV Structure Constants
///
/// These constants define the expected structure of the Russian Sberbank CSV format.
///
/// Minimum number of lines required for a valid CSV statement.
/// The Sberbank format has a fixed header structure that requires at least 12 lines.
pub const MIN_CSV_LINES: usize = 12;

/// Minimum number of lines required before we can safely extract the account number.
/// Account number appears in the header section, typically after line 5.
pub const MIN_LINES_FOR_ACCOUNT: usize = 5;

/// Maximum number of lines to search for the account number.
/// We limit the search to the first 10 lines to avoid false positives in transaction data.
pub const MAX_ACCOUNT_SEARCH_LINES: usize = 10;

/// Expected length of Russian bank account numbers.
/// Sberbank account numbers are exactly 20 digits long.
pub const ACCOUNT_NUMBER_LENGTH: usize = 20;

/// Number of lines to skip after finding the transaction header.
/// After finding "Дата проводки" (transaction date), we skip 2 lines (header + sub-header).
pub const TRANSACTION_HEADER_SKIP_LINES: usize = 2;

/// ## Column Index Constants
///
/// These constants map the column positions in the Sberbank CSV transaction format.
/// Column indices are 0-based.
///
/// Column index for the transaction date field.
/// Date appears in column 1 (second column) of each transaction row.
pub const DATE_COLUMN_INDEX: usize = 1;

/// Column index for the debit amount field.
/// Debit amounts appear in column 9 (tenth column) of transaction rows.
pub const DEBIT_AMOUNT_COLUMN_INDEX: usize = 9;

/// Column index for the credit amount field.
/// Credit amounts appear in column 13 (fourteenth column) of transaction rows.
pub const CREDIT_AMOUNT_COLUMN_INDEX: usize = 13;

/// Column index for the document/reference number field.
/// Document numbers appear in column 14 (fifteenth column) of transaction rows.
pub const REFERENCE_COLUMN_INDEX: usize = 14;

/// Starting column index for searching transaction descriptions.
/// Descriptions can appear in various columns starting from index 18.
pub const DESCRIPTION_SEARCH_START_INDEX: usize = 18;

/// Column index for the transaction description field in output format.
/// When writing CSV, descriptions are placed in column 20 (twenty-first column).
pub const DESCRIPTION_COLUMN_INDEX: usize = 20;

/// Total number of columns in the output CSV row format.
/// The Sberbank format uses 21 columns for transaction rows.
pub const OUTPUT_ROW_COLUMNS: usize = 21;

/// ## Balance Extraction Constants
///
/// These constants control how we extract balance information from the footer section.
///
/// Maximum offset to search for balance amounts in footer rows.
/// We search up to 15 columns after finding balance labels like "Входящий остаток".
pub const MAX_BALANCE_SEARCH_OFFSET: usize = 15;

/// Minimum amount threshold to consider a balance as valid.
/// Amounts below 0.01 are considered zero or invalid in the Russian banking system.
pub const MIN_AMOUNT_THRESHOLD: f64 = 0.01;

/// ## Date Parsing Constants
///
/// These constants help parse Russian date formats in the CSV.
///
/// Minimum length for a valid date string.
/// Russian dates like "01 января 2024 г." are typically longer than 10 characters.
pub const MIN_DATE_STRING_LENGTH: usize = 10;

/// Offset for extracting year from Russian date strings.
/// When parsing "01 января 2024 г.", we look 3 characters back from the last digit.
pub const YEAR_EXTRACTION_OFFSET: usize = 3;

/// Minimum valid year for date parsing.
/// Bank statements are unlikely to contain dates before year 2000.
pub const MIN_VALID_YEAR: u32 = 2000;

/// Maximum valid year for date parsing.
/// Bank statements are unlikely to contain dates after year 2100.
pub const MAX_VALID_YEAR: u32 = 2100;

/// ## Header Section Constants
///
/// These constants define positions in the CSV header section.
///
/// Line index (0-based) where currency information appears.
/// Currency information is in line 9 (index 8) of the header section.
pub const CURRENCY_LINE_INDEX: usize = 8;

/// ## Currency and Language Constants
///
/// These constants define currency codes and their Russian language equivalents.
///
/// Russian Ruble currency code (ISO 4217)
pub const CURRENCY_RUB: &str = "RUB";

/// US Dollar currency code (ISO 4217)
pub const CURRENCY_USD: &str = "USD";

/// Euro currency code (ISO 4217)
pub const CURRENCY_EUR: &str = "EUR";

/// Russian text for "Russian Ruble"
pub const RUSSIAN_RUBLE_FULL: &str = "российский рубль";

/// Russian text for "Ruble" (short form)
pub const RUSSIAN_RUBLE_SHORT: &str = "рубль";

/// Russian text for "Dollar"
pub const RUSSIAN_DOLLAR: &str = "доллар";

/// Russian text for "Euro"
pub const RUSSIAN_EURO: &str = "евро";

/// ## CSV Structure Markers
///
/// These constants define key markers in the Sberbank CSV format.
///
/// Russian text for "Transaction Date" (header marker)
pub const TRANSACTION_DATE_HEADER: &str = "дата проводки";

/// Russian text for "Balance Sheet" marker
pub const BALANCE_SHEET_MARKER: &str = "б/с";

/// Russian text for "Opening Balance"
pub const OPENING_BALANCE_LABEL: &str = "входящий остаток";

/// Russian text for "Closing Balance"
pub const CLOSING_BALANCE_LABEL: &str = "исходящий остаток";

/// Russian date format suffix (year indicator)
pub const RUSSIAN_YEAR_SUFFIX: &str = "г.";

/// ## CSV Output Headers
///
/// These constants define the headers used when writing CSV output.
///
/// Bank name for output
pub const BANK_NAME_SBERBUSINESS: &str = "СберБизнес";

/// Full bank name for output
pub const BANK_NAME_FULL: &str = "ПАО СБЕРБАНК";

/// Statement title
pub const STATEMENT_TITLE: &str = "ВЫПИСКА ОПЕРАЦИЙ ПО ЛИЦЕВОМУ СЧЕТУ";

/// Column header for transaction date
pub const COLUMN_TRANSACTION_DATE: &str = "Дата проводки";

/// Column header for account
pub const COLUMN_ACCOUNT: &str = "Счет";

/// Column header for debit amount
pub const COLUMN_DEBIT_AMOUNT: &str = "Сумма по дебету";

/// Column header for credit amount
pub const COLUMN_CREDIT_AMOUNT: &str = "Сумма по кредиту";

/// Column header for document number
pub const COLUMN_DOCUMENT_NUMBER: &str = "№ документа";

/// Column header for VO (internal code)
pub const COLUMN_VO: &str = "ВО";

/// Column header for bank
pub const COLUMN_BANK: &str = "Банк";

/// Column header for payment purpose
pub const COLUMN_PAYMENT_PURPOSE: &str = "Назначение платежа";

/// Debit label
pub const LABEL_DEBIT: &str = "Дебет";

/// Credit label
pub const LABEL_CREDIT: &str = "Кредит";

/// Footer label for operation count
pub const FOOTER_OPERATION_COUNT: &str = "Количество операций";

/// Footer label for opening balance
pub const FOOTER_OPENING_BALANCE: &str = "Входящий остаток";

/// Footer label for closing balance
pub const FOOTER_CLOSING_BALANCE: &str = "Исходящий остаток";

/// ## Error Messages
///
/// Standardized error messages for CSV parsing.
///
/// Error message for empty input
pub const ERROR_EMPTY_INPUT: &str = "Empty input";

/// Error message for CSV too short
pub const ERROR_CSV_TOO_SHORT: &str = "CSV too short - missing required sections";

/// Error message for missing account number
pub const ERROR_MISSING_ACCOUNT: &str = "Missing account number in header";

/// Error message for account number not found
pub const ERROR_ACCOUNT_NOT_FOUND: &str = "Account number not found in header";

/// Error message for missing currency
pub const ERROR_MISSING_CURRENCY: &str = "Missing currency in header";

/// Error message for transaction section not found
pub const ERROR_TRANSACTION_SECTION_NOT_FOUND: &str =
    "Transaction section not found (missing 'Дата проводки')";

/// Error message for empty date field
pub const ERROR_EMPTY_DATE_FIELD: &str = "Empty date field";

/// Error message for transaction with no amount
pub const ERROR_NO_TRANSACTION_AMOUNT: &str = "Transaction has no amount";

/// Error message for opening balance not found
pub const ERROR_OPENING_BALANCE_NOT_FOUND: &str = "Opening balance not found";

/// Error message for closing balance not found
pub const ERROR_CLOSING_BALANCE_NOT_FOUND: &str = "Closing balance not found";

/// Error message for date not found
pub const ERROR_DATE_NOT_FOUND: &str = "Date not found";
