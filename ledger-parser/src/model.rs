use serde::{Deserialize, Serialize};

/// Balance type indicator (credit or debit position)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BalanceType {
    Credit, // Positive balance (CRDT in CAMT, C in MT940)
    Debit,  // Negative balance (DBIT in CAMT, D in MT940)
}

/// Transaction type (credit/debit indicator)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    Credit, // Money received (CRDT in CAMT, C in MT940)
    Debit,  // Money paid (DBIT in CAMT, D in MT940)
}

/// Individual transaction entry (shared across all formats)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub booking_date: String,       // ISO 8601: "YYYY-MM-DD" (when booked)
    pub value_date: Option<String>, // ISO 8601: "YYYY-MM-DD" (value date, optional)
    pub amount: f64,                // Always positive number
    pub transaction_type: TransactionType, // Credit or Debit
    pub description: String,        // Transaction description/narrative
    pub reference: Option<String>,  // Optional reference/transaction ID
    pub counterparty_name: Option<String>, // Debtor/Creditor name
    pub counterparty_account: Option<String>, // Counterparty account/IBAN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction {
            booking_date: "2025-01-15".to_string(),
            value_date: Some("2025-01-15".to_string()),
            amount: 100.50,
            transaction_type: TransactionType::Credit,
            description: "Payment received".to_string(),
            reference: Some("REF123".to_string()),
            counterparty_name: Some("John Doe".to_string()),
            counterparty_account: Some("IBAN123".to_string()),
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
            booking_date: "2025-01-15".to_string(),
            value_date: None,
            amount: 250.75,
            transaction_type: TransactionType::Debit,
            description: "Purchase".to_string(),
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
