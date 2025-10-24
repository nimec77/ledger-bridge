use std::str::FromStr;

use strum_macros::{Display, EnumString};

use crate::error::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub(super) enum ElementName {
    #[strum(serialize = "Document")]
    Document,
    #[strum(serialize = "BkToCstmrStmt")]
    BkToCstmrStmt,
    #[strum(serialize = "Stmt")]
    Stmt,
    #[strum(serialize = "Acct")]
    Acct,
    #[strum(serialize = "Id")]
    Id,
    #[strum(serialize = "IBAN")]
    Iban,
    #[strum(serialize = "Ccy")]
    Currency,
    #[strum(serialize = "Bal")]
    Balance,
    #[strum(serialize = "Tp")]
    BalanceType,
    #[strum(serialize = "CdOrPrtry")]
    CodeOrProprietary,
    #[strum(serialize = "Cd")]
    Code,
    #[strum(serialize = "Amt")]
    Amount,
    #[strum(serialize = "CdtDbtInd")]
    CreditDebit,
    #[strum(serialize = "Dt")]
    Date,
    #[strum(serialize = "Ntry")]
    Entry,
    #[strum(serialize = "NtryRef")]
    EntryRef,
    #[strum(serialize = "BookgDt")]
    BookingDate,
    #[strum(serialize = "ValDt")]
    ValueDate,
    #[strum(serialize = "NtryDtls")]
    EntryDetails,
    #[strum(serialize = "TxDtls")]
    TransactionDetails,
    #[strum(serialize = "Refs")]
    References,
    #[strum(serialize = "TxId")]
    TransactionId,
    #[strum(serialize = "RmtInf")]
    RemittanceInfo,
    #[strum(serialize = "Ustrd")]
    UnstructuredRemittance,
    #[strum(serialize = "Strd")]
    StructuredRemittance,
    #[strum(serialize = "CdtrRefInf")]
    CreditorReferenceInfo,
    #[strum(serialize = "Ref")]
    ReferenceValue,
    #[strum(serialize = "RltdPties")]
    RelatedParties,
    #[strum(serialize = "Dbtr")]
    Debtor,
    #[strum(serialize = "Cdtr")]
    Creditor,
    #[strum(serialize = "DbtrAcct")]
    DebtorAccount,
    #[strum(serialize = "CdtrAcct")]
    CreditorAccount,
    #[strum(serialize = "Nm")]
    Name,
    #[strum(serialize = "AddtlTxInf")]
    AdditionalInfo,
    Other,
}

impl ElementName {
    pub(super) fn from_name_bytes(raw: &[u8]) -> Result<Self, ParseError> {
        let name = std::str::from_utf8(raw).map_err(|err| {
            ParseError::Camt053Error(format!("Invalid XML tag name encoding: {}", err))
        })?;
        let normalized = name.rsplit(':').next().unwrap_or(name);

        // Try to parse as known element, fall back to Other for unknown tags
        match ElementName::from_str(normalized) {
            Ok(element) => Ok(element),
            Err(_) => Ok(ElementName::Other), // Unknown tags become Other
        }
    }
}
