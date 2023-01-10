use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::io;

#[derive(Debug, Deserialize)]
pub struct Record {
    r#type: String,
    pub client: u16,
    tx: u32,
    amount: Option<f64>,
    frozen: bool,
}

pub struct AccountRegistry {
    accounts: HashMap<u16, LiveAccount>,
}

impl AccountRegistry {
    pub fn new() -> AccountRegistry {
        return AccountRegistry {
            accounts: HashMap::new(),
        };
    }

    fn add_account(&mut self, id: u16) -> &LiveAccount {
        let fresh_account = LiveAccount {
            transaction_record: HashMap::new(),
            account_details: AccountDetails {
                client: id,
                available: 0.0,
                held: 0.0,
                total: 0.0,
                locked: false,
            },
        };
        self.accounts.insert(id, fresh_account);
        &fresh_account
    }

    pub fn process_record(&mut self, record: Record) {
        let account = match self.accounts.entry(record.client) {
            Entry::Occupied(acc) => acc.into_mut(),
            Entry::Vacant(acc) => self.add_account(record.client),
        };

        account.process_transaction(record);
    }

    pub fn output_records(&self) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        for account in self.accounts.values() {
            wtr.serialize(account.account_details)?
        }
        wtr.flush()?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct AccountDetails {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl AccountDetails {
    // round_values ensures precision to four decimal places
    fn round_values(&mut self) {
        self.held = (self.held * 10000.0).round() / 10000.0;
        self.total = (self.total * 10000.0).round() / 10000.0;
        self.available = (self.available * 10000.0).round() / 10000.0;
    }

    fn recompute_total(&mut self) {
        self.total = self.available + self.held;
        self.round_values()
    }
}

pub struct LiveAccount {
    transaction_record: HashMap<u32, Record>,
    account_details: AccountDetails,
}

impl LiveAccount {
    // process_transaction receives a record and calls the relevant function
    // depending on what action is provided in the record
    pub fn process_transaction(&mut self, record: Record) {
        match record.r#type.as_str() {
            "deposit" => self.deposit(record),
            "withdrawal" => self.withdraw(record),
            "dispute" => self.dispute(record),
            "resolve" => self.resolve(record),
            "chargeback" => self.chargeback(record),
            _ => return,
        };
        // Recompute the total for the account
        self.account_details.recompute_total();
    }

    fn deposit(&mut self, record: Record) {
        if self.account_details.locked {
            return;
        }
        // Increase the available cash
        match record.amount {
            Some(amount) => self.account_details.available += amount,
            None => return,
        };
        // Add the transaction to the account's transaction list
        self.transaction_record.insert(record.tx, record);
    }
    
    fn withdraw(&mut self, record: Record) {
        if self.account_details.locked {
            return;
        }
        // Decrease the account's available cash
        match record.amount {
            Some(amount) => self.account_details.available -= amount,
            None => return,
        };
        // Add the transaction to the account's transaction list
        self.transaction_record.insert(record.tx, record);
    }
    fn dispute(&mut self, record: Record) {
        if self.account_details.locked{
            return;
        }
        // Remove the disputed transaction from the normal transaction list, if not found
        // then assume an error has occoured and do nothing
        let transaction: &Record = match self.transaction_record.get(&record.tx) {
            Some(record) => record,
            None => return,
        };

        if transaction.frozen{
            return;
        }
        // Decrease the amount of cash from the available pot and add it to the held pot
        match transaction.amount {
            Some(amount) => {
                self.account_details.available -= amount;
                self.account_details.held += amount;
            },
            None => return,
        }
        transaction.frozen = true;
    }
    fn resolve(&mut self, record: Record) {
        if self.account_details.locked {
            return;
        }
        // Remove the disputed transaction from the frozen transaction list, if not found
        // then assume an error has occoured and do nothing
        let transaction: &Record = match self.transaction_record.get(&record.tx) {
            Some(record) => record,
            None => return,
        };
        // If the transaction isn't frozen then this isn't a valid transaction
        if !transaction.frozen {
            return;
        }
        transaction.frozen = false;
        // Decrease the amount of cash from the held pot and add it to the available pot
        match transaction.amount {
            Some(amount) => {
                self.account_details.available += amount;
                self.account_details.held -= amount;
            },
            None => return,
        }
        // Add the previously frozen transaction to the normal transaction list
    }
    fn chargeback(&mut self, record: Record) {
        // Remove the disputed transaction from the frozen transaction list, if not found
        // then assume an error has occoured and do nothing
        let transaction: &Record = match self.transaction_record.get(&record.tx) {
            Some(record) => record,
            None => return,
        };
        if !transaction.frozen {
            return;
        }
        // Remove the amount in question from the held pot
        match transaction.amount {
            Some(amount) => {
                self.account_details.held -= amount;
            },
            None => return,
        }
        // Lock the account
        self.account_details.locked = true;
    }
}

mod tests {

    use super::*;

    // Allowing dead code here as this struct is only used in testing
    #[allow(dead_code)]
    pub struct TestCase {
        record: Record,
        account: Account,
        expected_total: f32,
        expected_held: f32,
        expected_available: f32,
        expected_locked: bool,
    }

    // run_test_cases runs through a set of configured test cases
    // designed to ensure the main record processing actions completed
    // as expected
    #[test]
    fn run_test_cases() {
        let test_cases: Vec<TestCase> = vec![
            // Deposit
            TestCase {
                record: Record {
                    r#type: String::from("deposit"),
                    client: 1,
                    tx: 1,
                    amount: Some(10.0),
                },
                account: Account {
                    transactions: HashMap::new(),
                    frozen_transactions: HashMap::new(),
                    client: 1,
                    available: 0.0,
                    held: 0.0,
                    locked: false,
                    total: 0.0,
                },
                expected_total: 10.0,
                expected_held: 0.0,
                expected_available: 10.0,
                expected_locked: false,
            },
            // Withdraw
            TestCase {
                record: Record {
                    r#type: String::from("withdrawal"),
                    client: 1,
                    tx: 1,
                    amount: Some(10.0),
                },
                account: Account {
                    transactions: HashMap::new(),
                    frozen_transactions: HashMap::new(),
                    client: 1,
                    available: 20.0,
                    held: 0.0,
                    locked: false,
                    total: 0.0,
                },
                expected_total: 10.0,
                expected_held: 0.0,
                expected_available: 10.0,
                expected_locked: false,
            },
            // Dispute
            TestCase {
                record: Record {
                    r#type: String::from("dispute"),
                    client: 1,
                    tx: 1,
                    amount: None,
                },
                account: Account {
                    transactions: HashMap::from([(
                        1,
                        Record {
                            r#type: String::from("deposit"),
                            client: 1,
                            tx: 1,
                            amount: Some(10.0),
                        },
                    )]),
                    frozen_transactions: HashMap::new(),
                    client: 1,
                    available: 10.0,
                    held: 0.0,
                    locked: false,
                    total: 0.0,
                },
                expected_total: 10.0,
                expected_held: 10.0,
                expected_available: 0.0,
                expected_locked: false,
            },
            // Resolve
            TestCase {
                record: Record {
                    r#type: String::from("resolve"),
                    client: 1,
                    tx: 1,
                    amount: None,
                },
                account: Account {
                    transactions: HashMap::new(),
                    frozen_transactions: HashMap::from([(
                        1,
                        Record {
                            r#type: String::from("deposit"),
                            client: 1,
                            tx: 1,
                            amount: Some(10.0),
                        },
                    )]),
                    client: 1,
                    available: 0.0,
                    held: 10.0,
                    locked: false,
                    total: 10.0,
                },
                expected_total: 10.0,
                expected_held: 0.0,
                expected_available: 10.0,
                expected_locked: false,
            },
            // Chargeback
            TestCase {
                record: Record {
                    r#type: String::from("chargeback"),
                    client: 1,
                    tx: 1,
                    amount: None,
                },
                account: Account {
                    transactions: HashMap::new(),
                    frozen_transactions: HashMap::from([(
                        1,
                        Record {
                            r#type: String::from("deposit"),
                            client: 1,
                            tx: 1,
                            amount: Some(10.0),
                        },
                    )]),
                    client: 1,
                    available: 0.0,
                    held: 10.0,
                    locked: false,
                    total: 10.0,
                },
                expected_total: 0.0,
                expected_held: 0.0,
                expected_available: 0.0,
                expected_locked: true,
            },
        ];
        for test_case in test_cases {
            println!("Runing test case for {}", test_case.record.r#type);
            let mut test_account = test_case.account;
            let test_transaction = test_case.record;
            test_account.process_transaction(test_transaction);
            assert_eq!(test_account.total, test_case.expected_total);
            assert_eq!(test_account.held, test_case.expected_held);
            assert_eq!(test_account.available, test_case.expected_available);
            assert_eq!(test_account.locked, test_case.expected_locked);
        }
    }

    #[test]
    fn test_unsupported_action() {
        // Given a test account
        let mut test_account = new_account(1);
        // and a record with an unupported transaction
        let test_transaction = Record {
            r#type: String::from("unsupported_action"),
            client: 1,
            tx: 1,
            amount: Some(10.0),
        };
        // When the transaction is processed
        test_account.process_transaction(test_transaction);

        // Then the account is not updated
        test_account.transactions.is_empty();
        assert!(test_account.available == 0.0);
        assert!(test_account.total == 0.0);
        assert!(test_account.held == 0.0);
        assert!(test_account.locked == false);
    }
}
