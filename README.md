# Toy Pay

Toy pay is a simple toy payment engine, it loads a set of transactions from a csv file and outputs the resulting account information to stdout

## Usage

Toy pay takes a single argument which is the path to the input csv file

e.g. cargo run input_file.csv > output.csv

## Assumtions

During the creation of Toy Pay I made the following assuming about the input data and the functionality of the application

 - Any transaction made against an account already locked is to be ignored
 - On any transactions that did not supply an amount in the input csv will still provide the comma after the missing amount
 - The application will only be reading from one file at a time so no mutex or locking is needed on the accounts themselves