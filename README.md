## Some notes on this project.

- It is essential to process transactions serially per `client_id` to avoid conflicts and maintain transaction ordering for each client. Therefore scaling this project horizontally would involve sharding on `client_id` and having a separate `ExchangeEngine` running per shard. 

- Regarding scaling, I explored this in depth with a POC I wrote last year: https://github.com/AJTJ/high_frequency_trading_architecture, where I explore how I might scale a larger trading architecture maintaining account-level consistency.

- The parser streams the CSV one row at a time, rather than loading the whole file into memory (memory stays O(1)). For demo purposes, this handles very large CSV files and mimics stream ingestion.

- I've included some unit tests and a CSV file of edge cases such as: duplicate transactions, invalid disputes/chargebacks, insufficient funds for withdrawals, precision errors etc. Can be run with `cargo test`

- The separation of concerns between account manipulation in `Account` (managing account math) and transaction validation/management in `ExchangeEngine` is an idiomatic design choice that would scale more effectively. Modularity helps with sanity and future updates.

- Please see comments in my code, which is usually my preferred form of documentation.

- Regarding security, I've put a bit of care into ensuring proper precision and rejecting transactions that exceed it. In a real world scenario, overflow and underflow would likely need further scrutiny.

🦀