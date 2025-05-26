use csv::ReaderBuilder;
use exchange_test::engine::ExchangeEngine;
use std::{env, error::Error, fs::File};

// Reads transactions from a CSV file provided as a command line argument
// Outputs the final state of all accounts in CSV format to stdout
fn main() -> Result<(), Box<dyn Error>> {
    // NOTE: Currently I have one engine, but in a larger system you would shard at the account level
    let mut engine = ExchangeEngine::default();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <input_csv>");
        std::process::exit(1);
    }
    let input_file = &args[1];
    let file = File::open(input_file)?;
    let mut rdr = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .comment(Some(b'#'))
        .from_reader(file);

    // Stream each record one at a time to avoid loading the entire file into memory
    for result in rdr.deserialize() {
        match result {
            Ok(record) => {
                if let Err(e) = engine.process_transaction(&record) {
                    // ignore invalid disputes, resolves, and chargebacks
                    // just printing an error
                    eprintln!("Failed to process transaction: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to deserialize record: {}", e);
            }
        }
    }

    engine.write_accounts_to_csv()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use exchange_test::models::{Record, TxType};
    use rust_decimal_macros::dec;

    #[test]
    fn test_large_csv_with_edge_cases() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut csv_path = std::path::PathBuf::from(manifest_dir);
        csv_path.push("tests/data/test_data.csv");

        let file = File::open(&csv_path).expect("Failed to open test data CSV file");
        let mut rdr = ReaderBuilder::new().comment(Some(b'#')).from_reader(file);

        let mut engine = ExchangeEngine::default();

        // Keep track of error count
        let mut error_count = 0;
        for result in rdr.deserialize() {
            let record: Record = match result {
                Ok(rec) => rec,
                Err(_) => {
                    error_count += 1;
                    continue;
                }
            };
            if engine.process_transaction(&record).is_err() {
                error_count += 1;
            }
        }
        assert_eq!(error_count, 13);

        let account1 = engine.accounts.get(&1).unwrap();
        assert_eq!(account1.available, dec!(1300.00));
        assert_eq!(account1.held, dec!(0.00));
        assert_eq!(account1.total, dec!(1300.00));
        assert!(!account1.locked);

        let account2 = engine.accounts.get(&2).unwrap();
        assert_eq!(account2.available, dec!(0.0000));
        assert_eq!(account2.held, dec!(0.0000));
        assert_eq!(account2.total, dec!(0.0000));
        assert!(account2.locked);

        assert!(!engine.accounts.contains_key(&3));
        assert!(!engine.accounts.contains_key(&4));
    }

    #[test]
    fn test_account_deposit_and_withdrawal() {
        let mut engine = ExchangeEngine::default();

        // Deposit
        let deposit = Record {
            tx_type: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(dec!(10.00)),
        };
        engine.process_transaction(&deposit).unwrap();

        // Withdraw
        let withdrawal = Record {
            tx_type: TxType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(dec!(5.00)),
        };
        engine.process_transaction(&withdrawal).unwrap();

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(5.00));
        assert_eq!(account.total, dec!(5.00));
        assert_eq!(account.held, dec!(0.00));
    }

    #[test]
    fn test_account_dispute_and_resolve() {
        let mut engine = ExchangeEngine::default();

        // Deposit
        let deposit = Record {
            tx_type: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(dec!(10.00)),
        };
        engine.process_transaction(&deposit).unwrap();

        // Dispute
        let dispute = Record {
            tx_type: TxType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        engine.process_transaction(&dispute).unwrap();

        // Resolve
        let resolve = Record {
            tx_type: TxType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
        };
        engine.process_transaction(&resolve).unwrap();

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10.00));
        assert_eq!(account.held, dec!(0.00));
        assert_eq!(account.total, dec!(10.00));
    }

    #[test]
    fn test_account_chargeback() {
        let mut engine = ExchangeEngine::default();

        // Deposit
        let deposit = Record {
            tx_type: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(dec!(10.00)),
        };
        engine.process_transaction(&deposit).unwrap();

        // Dispute
        let dispute = Record {
            tx_type: TxType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        engine.process_transaction(&dispute).unwrap();

        // Chargeback
        let chargeback = Record {
            tx_type: TxType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        };
        engine.process_transaction(&chargeback).unwrap();

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(0.00));
        assert_eq!(account.held, dec!(0.00));
        assert_eq!(account.total, dec!(0.00));
        assert!(account.locked);
    }
}
