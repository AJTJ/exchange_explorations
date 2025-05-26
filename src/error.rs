use crate::models::TransactionId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineErr {
    #[error("Tx id {0} already seen")]
    DuplicateTx(TransactionId),
    #[error("Tx {0} refers to unknown account")]
    UnknownAccount(TransactionId),
    #[error("Insufficient available funds")]
    InsufficientFunds,
    #[error("Invalid precision")]
    Precision,
    #[error("Account is locked")]
    AccountLocked,
    #[error("Failed to deserialize record: {0}")]
    DeserializationError(String),
    #[error("Failed to write CSV output: {0}")]
    CsvWriteError(String),
}
