use crate::error::ParseError;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ElementName {
    Document,
    BkToCstmrStmt,
    Stmt,
    Acct,
    Id,
    Iban,
    Currency,
    Balance,
    BalanceType,
    CodeOrProprietary,
    Code,
    Amount,
    CreditDebit,
    Date,
    Entry,
    EntryRef,
    BookingDate,
    ValueDate,
    EntryDetails,
    TransactionDetails,
    References,
    TransactionId,
    RemittanceInfo,
    UnstructuredRemittance,
    StructuredRemittance,
    CreditorReferenceInfo,
    ReferenceValue,
    RelatedParties,
    Debtor,
    Creditor,
    DebtorAccount,
    CreditorAccount,
    Name,
    AdditionalInfo,
    Other,
}

impl ElementName {
    pub fn from_name_bytes(raw: &[u8]) -> Result<Self, ParseError> {
        let name = std::str::from_utf8(raw).map_err(|err| {
            ParseError::Camt053Error(format!("Invalid XML tag name encoding: {}", err))
        })?;
        let normalized = name.rsplit(':').next().unwrap_or(name);
        Ok(ElementName::from_name(normalized))
    }

    fn from_name(name: &str) -> Self {
        match name {
            "Document" => ElementName::Document,
            "BkToCstmrStmt" => ElementName::BkToCstmrStmt,
            "Stmt" => ElementName::Stmt,
            "Acct" => ElementName::Acct,
            "Id" => ElementName::Id,
            "IBAN" => ElementName::Iban,
            "Ccy" => ElementName::Currency,
            "Bal" => ElementName::Balance,
            "Tp" => ElementName::BalanceType,
            "CdOrPrtry" => ElementName::CodeOrProprietary,
            "Cd" => ElementName::Code,
            "Amt" => ElementName::Amount,
            "CdtDbtInd" => ElementName::CreditDebit,
            "Dt" => ElementName::Date,
            "Ntry" => ElementName::Entry,
            "NtryRef" => ElementName::EntryRef,
            "BookgDt" => ElementName::BookingDate,
            "ValDt" => ElementName::ValueDate,
            "NtryDtls" => ElementName::EntryDetails,
            "TxDtls" => ElementName::TransactionDetails,
            "Refs" => ElementName::References,
            "TxId" => ElementName::TransactionId,
            "RmtInf" => ElementName::RemittanceInfo,
            "Ustrd" => ElementName::UnstructuredRemittance,
            "Strd" => ElementName::StructuredRemittance,
            "CdtrRefInf" => ElementName::CreditorReferenceInfo,
            "Ref" => ElementName::ReferenceValue,
            "RltdPties" => ElementName::RelatedParties,
            "Dbtr" => ElementName::Debtor,
            "Cdtr" => ElementName::Creditor,
            "DbtrAcct" => ElementName::DebtorAccount,
            "CdtrAcct" => ElementName::CreditorAccount,
            "Nm" => ElementName::Name,
            "AddtlTxInf" => ElementName::AdditionalInfo,
            "Othr" => ElementName::Other,
            _ => ElementName::Other,
        }
    }
}
