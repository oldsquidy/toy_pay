#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

/*!
 * The `toy_pay` crate is an application for processing a CSV file of transactional info and
 * producing CSV output of the closing balances of all accounts included in the input file
 */

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;

use std::{io, process};

use std::ffi::OsString;
mod transaction;

/// run starts the main functionality of the toy_pay app
///
/// It will parse the requested csv file into the Record struct
/// processing each row into the relevant accounts
fn run() -> Result<(), Box<dyn Error>> {
    // Create an empty hashmap to store the accounts in
    let mut accounts: HashMap<u16, transaction::Account> = HashMap::new();

    // Read the input provided via command line argument
    let input_file = get_input_file()?;
    let mut file_reader = csv::ReaderBuilder::new()
        .flexible(false)
        .trim(csv::Trim::All)
        .from_path(input_file)?;

    // Loop through each record of the provided csv
    for result in file_reader.records() {
        let record: transaction::Record = result?.deserialize(None)?;

        // Check to see if we already have the specified account, creating one if not
        let account = match accounts.entry(record.client) {
            Entry::Occupied(acc) => acc.into_mut(),
            Entry::Vacant(acc) => {
                let new_account = transaction::new_account(record.client);
                acc.insert(new_account)
            }
        };
        // Process the record into the relevent account
        account.process_transaction(record);
    }
    // Output the found accounts
    output_accounts(accounts)
}

// output_accounts will serealize the populated accounts into a CSV format
// before outputing them to stdout
fn output_accounts(accounts: HashMap<u16, transaction::Account>) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for account in accounts.values() {
        wtr.serialize(account)?;
    }
    wtr.flush()?;
    Ok(())
}

// get_input_file checks we have been provided with enough command line
// arguments and returns the correct one
fn get_input_file() -> Result<OsString, Box<dyn Error>> {
    match std::env::args_os().nth(1) {
        None => Err(From::from("No input file supplied")),
        Some(file_path) => Ok(file_path),
    }
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1)
    }
}
