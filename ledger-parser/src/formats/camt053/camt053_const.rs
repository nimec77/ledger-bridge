/// XML tag name constants for CAMT.053 format.
///
/// These remain in the canonical ISO 20022 casing; the writer emits these tags
/// verbatim, and the parser compares them directly. Changing the literals (for
/// example uppercasing everything) breaks both read and write flows.
pub(crate) const DOCUMENT_TAG: &str = "Document";
pub(crate) const BK_TO_CSTM_STMT_TAG: &str = "BkToCstmrStmt";
pub(crate) const STMT_TAG: &str = "Stmt";
pub(crate) const ACCT_TAG: &str = "Acct";
pub(crate) const ID_TAG: &str = "Id";
pub(crate) const IBAN_TAG: &str = "IBAN";
pub(crate) const CCY_TAG: &str = "Ccy";
pub(crate) const BAL_TAG: &str = "Bal";
pub(crate) const TP_TAG: &str = "Tp";
pub(crate) const CD_OR_PRTY_TAG: &str = "CdOrPrtry";
pub(crate) const CD_TAG: &str = "Cd";
pub(crate) const AMT_TAG: &str = "Amt";
pub(crate) const CDT_DBT_IND_TAG: &str = "CdtDbtInd";
pub(crate) const DT_TAG: &str = "Dt";
pub(crate) const NTRY_TAG: &str = "Ntry";
pub(crate) const NTRY_REF_TAG: &str = "NtryRef";
pub(crate) const BOOKG_DT_TAG: &str = "BookgDt";
pub(crate) const VAL_DT_TAG: &str = "ValDt";
pub(crate) const NTRY_DTLS_TAG: &str = "NtryDtls";
pub(crate) const TX_DTLS_TAG: &str = "TxDtls";
pub(crate) const REFS_TAG: &str = "Refs";
pub(crate) const TX_ID_TAG: &str = "TxId";
pub(crate) const RMT_INF_TAG: &str = "RmtInf";
pub(crate) const USTRD_TAG: &str = "Ustrd";
pub(crate) const STRD_TAG: &str = "Strd";
pub(crate) const CDTR_REF_INF_TAG: &str = "CdtrRefInf";
pub(crate) const REF_TAG: &str = "Ref";
pub(crate) const RLT_PTIES_TAG: &str = "RltdPties";
pub(crate) const DBTR_TAG: &str = "Dbtr";
pub(crate) const CDTR_TAG: &str = "Cdtr";
pub(crate) const DBTR_ACCT_TAG: &str = "DbtrAcct";
pub(crate) const CDTR_ACCT_TAG: &str = "CdtrAcct";
pub(crate) const NM_TAG: &str = "Nm";
pub(crate) const ADDTL_TX_INF_TAG: &str = "AddtlTxInf";
pub(crate) const OTHR_TAG: &str = "Othr";

// Balance type constants
pub(crate) const OPBD_BALANCE_TYPE: &str = "OPBD";
pub(crate) const CLBD_BALANCE_TYPE: &str = "CLBD";

// Credit/Debit indicator constants
pub(crate) const CRDT_INDICATOR: &str = "CRDT";
pub(crate) const DBIT_INDICATOR: &str = "DBIT";
