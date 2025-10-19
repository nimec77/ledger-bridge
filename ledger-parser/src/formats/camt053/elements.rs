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
        match name.to_uppercase().as_str() {
            "DOCUMENT" | "document" => ElementName::Document,
            "BKTOCSTMRSMT" | "bktocstmrstmt" => ElementName::BkToCstmrStmt,
            "STMT" => ElementName::Stmt,
            "ACCT" => ElementName::Acct,
            "ID" => ElementName::Id,
            "IBAN" => ElementName::Iban,
            "CCY" => ElementName::Currency,
            "BAL" => ElementName::Balance,
            "TP" => ElementName::BalanceType,
            "CDORPRTRY" => ElementName::CodeOrProprietary,
            "CD" => ElementName::Code,
            "AMT" => ElementName::Amount,
            "CDTDBTIND" => ElementName::CreditDebit,
            "DT" => ElementName::Date,
            "NTRY" => ElementName::Entry,
            "NTRYREF" => ElementName::EntryRef,
            "BOOKGDT" => ElementName::BookingDate,
            "VALDT" => ElementName::ValueDate,
            "NTRYDTLS" => ElementName::EntryDetails,
            "TXDTLS" => ElementName::TransactionDetails,
            "REFS" => ElementName::References,
            "TXID" => ElementName::TransactionId,
            "RMTINF" => ElementName::RemittanceInfo,
            "USTRD" => ElementName::UnstructuredRemittance,
            "STRD" => ElementName::StructuredRemittance,
            "CDTRREFINF" => ElementName::CreditorReferenceInfo,
            "REF" => ElementName::ReferenceValue,
            "RLTDPTIES" => ElementName::RelatedParties,
            "DBTR" => ElementName::Debtor,
            "CDTR" => ElementName::Creditor,
            "DBTRACCT" => ElementName::DebtorAccount,
            "CDTRACCT" => ElementName::CreditorAccount,
            "NM" => ElementName::Name,
            "ADDTLTXINF" => ElementName::AdditionalInfo,
            "OTHR" => ElementName::Other,
            _ => ElementName::Other,
        }
    }
}
