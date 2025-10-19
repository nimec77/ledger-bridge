//! Type conversions from CsvStatement to other formats
//!
//! Implements the `From` trait to enable idiomatic conversions between CSV
//! and other format structures (MT940, CAMT.053).

use crate::{Camt053Statement, CsvStatement, Mt940};

/// Convert CSV to MT940 format
///
/// Performs a direct field-by-field conversion since both structures
/// share the same data model.
///
/// # Example
/// ```ignore
/// # use ledger_parser::{CsvStatement, Mt940};
/// let csv = CsvStatement { /* ... */ };
/// let mt940: Mt940 = csv.into();
/// ```
impl From<CsvStatement> for Mt940 {
    fn from(csv: CsvStatement) -> Self {
        Mt940 {
            account_number: csv.account_number,
            currency: csv.currency,
            opening_balance: csv.opening_balance,
            opening_date: csv.opening_date,
            opening_indicator: csv.opening_indicator,
            closing_balance: csv.closing_balance,
            closing_date: csv.closing_date,
            closing_indicator: csv.closing_indicator,
            transactions: csv.transactions,
        }
    }
}

/// Convert CSV to CAMT.053 format
///
/// Performs a direct field-by-field conversion since both structures
/// share the same data model.
///
/// # Example
/// ```ignore
/// # use ledger_parser::{CsvStatement, Camt053};
/// let csv = CsvStatement { /* ... */ };
/// let camt053: Camt053 = csv.into();
/// ```
impl From<CsvStatement> for Camt053Statement {
    fn from(csv: CsvStatement) -> Self {
        Camt053Statement {
            account_number: csv.account_number,
            currency: csv.currency,
            opening_balance: csv.opening_balance,
            opening_date: csv.opening_date,
            opening_indicator: csv.opening_indicator,
            closing_balance: csv.closing_balance,
            closing_date: csv.closing_date,
            closing_indicator: csv.closing_indicator,
            transactions: csv.transactions,
        }
    }
}
