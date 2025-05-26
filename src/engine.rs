use crate::account::Account;
use crate::error::EngineErr;
use crate::models::{has_valid_precision, ClientId, Record, TransactionId, TxType};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io;

#[derive(Default)]
pub struct ExchangeEngine {
    pub accounts: HashMap<ClientId, Account>,
    pub transactions: HashMap<TransactionId, Record>,
    pub disputes: HashSet<TransactionId>,
}

impl ExchangeEngine {
    // PUBLIC
    pub fn process_transaction(&mut self, record: &Record) -> Result<(), EngineErr> {
        match record.tx_type {
            TxType::Deposit => self.process_deposit(&record),
            TxType::Withdrawal => self.process_withdrawal(&record),
            TxType::Dispute => self.process_dispute(&record),
            TxType::Resolve => self.process_resolve(&record),
            TxType::Chargeback => self.process_chargeback(&record),
        }
    }

    // Outputs client ID, available funds, held funds, total funds, and locked status.
    pub fn write_accounts_to_csv(&self) -> Result<(), EngineErr> {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        wtr.write_record(&["client", "available", "held", "total", "locked"])
            .map_err(|e| EngineErr::CsvWriteError(e.to_string()))?;

        for (client_id, account) in self.accounts.clone() {
            wtr.write_record(&[
                client_id.to_string(),
                format!("{:.4}", account.available),
                format!("{:.4}", account.held),
                format!("{:.4}", account.total),
                account.locked.to_string(),
            ])
            .map_err(|e| EngineErr::CsvWriteError(e.to_string()))?;
        }

        wtr.flush()
            .map_err(|e| EngineErr::CsvWriteError(e.to_string()))?;
        Ok(())
    }

    // PRIVATE METHODS
    fn process_deposit(&mut self, record: &Record) -> Result<(), EngineErr> {
        if self.transactions.contains_key(&record.tx) {
            return Err(EngineErr::DuplicateTx(record.tx));
        }

        if let Some(amount) = record.amount {
            if amount.is_sign_negative() {
                return Err(EngineErr::Precision);
            }

            if !has_valid_precision(&amount) {
                return Err(EngineErr::Precision);
            }

            match self.accounts.entry(record.client) {
                Entry::Occupied(mut e) => e.get_mut().deposit(amount),
                Entry::Vacant(e) => {
                    let mut acct = Account::new();
                    acct.deposit(amount)?;
                    e.insert(acct);
                    Ok(())
                }
            }?;

            self.transactions.insert(record.tx, record.clone());
            Ok(())
        } else {
            Err(EngineErr::Precision)
        }
    }

    fn process_withdrawal(&mut self, record: &Record) -> Result<(), EngineErr> {
        if self.transactions.contains_key(&record.tx) {
            return Err(EngineErr::DuplicateTx(record.tx));
        }

        if let Some(amount) = record.amount {
            if amount.is_sign_negative() {
                return Err(EngineErr::Precision);
            }

            if !has_valid_precision(&amount) {
                return Err(EngineErr::Precision);
            }

            let acct = self
                .accounts
                .get_mut(&record.client)
                .ok_or(EngineErr::UnknownAccount(record.tx))?;

            acct.withdraw(amount)?;
            self.transactions.insert(record.tx, record.clone());
            Ok(())
        } else {
            Err(EngineErr::Precision)
        }
    }

    fn process_dispute(&mut self, record: &Record) -> Result<(), EngineErr> {
        let disputed_tx = self
            .transactions
            .get(&record.tx)
            .ok_or(EngineErr::UnknownAccount(record.tx))?;

        if disputed_tx.tx_type != TxType::Deposit {
            return Err(EngineErr::UnknownAccount(record.tx));
        }

        if self.disputes.contains(&record.tx) {
            return Err(EngineErr::DuplicateTx(record.tx));
        }

        if let Some(amount) = disputed_tx.amount {
            self.accounts
                .get_mut(&record.client)
                .ok_or(EngineErr::UnknownAccount(record.tx))?
                .apply_dispute(amount)?;
            self.disputes.insert(record.tx);
            Ok(())
        } else {
            Err(EngineErr::Precision)
        }
    }

    fn process_resolve(&mut self, record: &Record) -> Result<(), EngineErr> {
        if !self.disputes.contains(&record.tx) {
            return Err(EngineErr::UnknownAccount(record.tx));
        }

        let disputed_tx = self
            .transactions
            .get(&record.tx)
            .ok_or(EngineErr::UnknownAccount(record.tx))?;

        if let Some(amount) = disputed_tx.amount {
            self.accounts
                .get_mut(&record.client)
                .ok_or(EngineErr::UnknownAccount(record.tx))?
                .resolve_dispute(amount)?;
            self.disputes.remove(&record.tx);
            Ok(())
        } else {
            Err(EngineErr::Precision)
        }
    }

    fn process_chargeback(&mut self, record: &Record) -> Result<(), EngineErr> {
        if !self.disputes.contains(&record.tx) {
            return Err(EngineErr::UnknownAccount(record.tx));
        }

        let disputed_tx = self
            .transactions
            .get(&record.tx)
            .ok_or(EngineErr::UnknownAccount(record.tx))?;

        if let Some(amount) = disputed_tx.amount {
            self.accounts
                .get_mut(&record.client)
                .ok_or(EngineErr::UnknownAccount(record.tx))?
                .chargeback(amount)?;
            self.disputes.remove(&record.tx);
            Ok(())
        } else {
            Err(EngineErr::Precision)
        }
    }
}
