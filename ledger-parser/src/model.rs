use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// Balance type indicator representing credit or debit position.
///
/// Used to indicate whether a balance represents a positive (credit) or negative (debit) position.
/// This enum is shared across all formats for consistency.
///
/// # Format Mappings
/// - **CAMT.053**: `CRDT` or `DBIT` in `<CdtDbtInd>` element
/// - **MT940**: `C` or `D` in balance tags (`:60F:`, `:62F:`)
/// - **CSV**: Derived from balance amount sign
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BalanceType {
    /// Positive balance (credit position)
    Credit,
    /// Negative balance (debit position)
    Debit,
}

/// Transaction type indicating whether money was received or paid out.
///
/// Used to classify individual transactions as incoming (credit) or outgoing (debit).
/// Note that the amount field in `Transaction` is always positive; this enum provides direction.
///
/// # Format Mappings
/// - **CAMT.053**: `CRDT` or `DBIT` in `<CdtDbtInd>` element
/// - **MT940**: `C` or `D` in transaction line (`:61:`)
/// - **CSV**: Separate debit/credit columns merged into single type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Money received (incoming transaction)
    Credit,
    /// Money paid out (outgoing transaction)
    Debit,
}

/// Individual transaction entry shared across all statement formats.
///
/// Represents a single financial transaction with all relevant details.
/// Fields are designed to accommodate data from CSV, MT940, and CAMT.053 formats.
///
/// # Field Details
/// - **booking_date**: When the transaction was posted to the account
/// - **value_date**: Optional value date (when funds become available)
/// - **amount**: Transaction amount (always positive; see `transaction_type` for direction)
/// - **transaction_type**: Whether this is incoming (Credit) or outgoing (Debit)
/// - **description**: Human-readable transaction description
/// - **reference**: Optional transaction reference or ID
/// - **counterparty_name**: Optional name of the other party (debtor/creditor)
/// - **counterparty_account**: Optional account number/IBAN of the other party
///
/// # Example
/// ```
/// use ledger_parser::{Transaction, TransactionType};
/// use chrono::{DateTime, FixedOffset, TimeZone};
///
/// let transaction = Transaction {
///     booking_date: FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2025, 1, 15, 0, 0, 0).unwrap(),
///     value_date: Some("2025-01-15".to_string()),
///     amount: 100.50,
///     transaction_type: TransactionType::Credit,
///     description: "Payment received".to_string(),
///     reference: Some("REF123".to_string()),
///     counterparty_name: Some("John Doe".to_string()),
///     counterparty_account: Some("GB29NWBK60161331926819".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Date when the transaction was posted to the account
    pub booking_date: DateTime<FixedOffset>,
    /// Optional value date (when funds become available)
    pub value_date: Option<String>,
    /// Transaction amount (always positive number)
    pub amount: f64,
    /// Whether this is a credit (incoming) or debit (outgoing) transaction
    pub transaction_type: TransactionType,
    /// Human-readable transaction description/narrative
    pub description: String,
    /// Optional transaction reference or ID
    pub reference: Option<String>,
    /// Optional name of the other party (debtor for credits, creditor for debits)
    pub counterparty_name: Option<String>,
    /// Optional account number/IBAN of the other party
    pub counterparty_account: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::formats::utils;

    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction {
            booking_date: utils::parse_date("2025-01-15").unwrap(),
            value_date: Some("2025-01-15".into()),
            amount: 100.50,
            transaction_type: TransactionType::Credit,
            description: "Payment received".into(),
            reference: Some("REF123".into()),
            counterparty_name: Some("John Doe".into()),
            counterparty_account: Some("IBAN123".into()),
        };
        assert_eq!(tx.amount, 100.50);
        assert_eq!(tx.transaction_type, TransactionType::Credit);
    }

    #[test]
    fn test_balance_type_creation() {
        let credit = BalanceType::Credit;
        let debit = BalanceType::Debit;
        assert_eq!(credit, BalanceType::Credit);
        assert_eq!(debit, BalanceType::Debit);
        assert_ne!(credit, debit);
    }

    #[test]
    fn test_transaction_type_creation() {
        let credit = TransactionType::Credit;
        let debit = TransactionType::Debit;
        assert_eq!(credit, TransactionType::Credit);
        assert_eq!(debit, TransactionType::Debit);
        assert_ne!(credit, debit);
    }

    #[test]
    fn test_transaction_serialization() {
        let tx = Transaction {
            booking_date: utils::parse_date("2025-01-15").unwrap(),
            value_date: None,
            amount: 250.75,
            transaction_type: TransactionType::Debit,
            description: "Purchase".into(),
            reference: None,
            counterparty_name: None,
            counterparty_account: None,
        };

        // Test that it can be serialized and deserialized
        let serialized = serde_json::to_string(&tx).unwrap();
        let deserialized: Transaction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tx, deserialized);
    }
}
