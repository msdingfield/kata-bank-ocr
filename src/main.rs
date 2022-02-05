// https://codingdojo.org/kata/BankOCR/

/*
012345678901234567890123456
    _  _     _  _  _  _  _
  | _| _||_||_ |_   ||_||_|
  ||_  _|  | _||_|  ||_| _|

 */
mod bank_ocr;
mod bank_ocr_test;

use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use bank_ocr_test::do_tests;

fn main() -> io::Result<()> {
    let args : Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "--test" {
        println!("Running tests...");
        do_tests();
    } else if args.len() == 3 {
        println!("Parsing {}", args[1]);
        parse(&args[1], &args[2]);
    } else {
        println!("Usage: bank_ocr <input file> <output file>");
    }
    Ok(())
}

fn parse(input: &String, _output: &String) {
    let input_result = File::open(input);
    if input_result.is_err() {
        println!("Error opening input file {}.", input);
        return;
    }

    let file = input_result.unwrap();
    let reader = BufReader::new(file);

    let mut parser = bank_ocr::Parser::new();
    for lr in reader.lines() {
        if lr.is_ok() {
            let line = lr.unwrap();
            let result = parser.process_line(line);
            if result.error.is_some() {
                let e = result.error.unwrap();
                println!("ERROR: {}:{}: row {}: {}", e.line_number, e.row, e.col, e.message);
            } else if result.account_number.is_some() {
                let buf = result.account_number.unwrap();
                println!("{}", buf);
            }
        } else {
            println!("Error reading line.");
        }
    }
}
