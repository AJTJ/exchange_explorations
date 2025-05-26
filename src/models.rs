use rust_decimal::Decimal;
use serde::Deserialize;
// use crate::error::EngineErr;

pub type ClientId = u16;
pub type TransactionId = u32;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

// Represents a transaction record parsed from the CSV input
#[derive(Debug, Deserialize, Clone)]
pub struct Record {
    #[serde(rename = "type")]
    pub tx_type: TxType,
    pub client: ClientId,
    pub tx: TransactionId,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub amount: Option<Decimal>,
}

pub fn has_valid_precision(amount: &Decimal) -> bool {
    amount.scale() <= 4 // Scale gives the number of decimal places
}
