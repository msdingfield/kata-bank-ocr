use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use bankocr::{is_checksum_valid, Parser, Status};

fn main() -> io::Result<()> {
    let args : Vec<String> = env::args().collect();
    if args.len() == 3 {
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

    let mut parser = Parser::new();
    for lr in reader.lines() {
        if lr.is_ok() {
            let line = lr.unwrap();
            let result = parser.process_line(line);

            match result {
                Status::Success(account_number) => {
                    if is_checksum_valid(&account_number) {
                        println!("{}", account_number);
                    } else {
                        println!("{} bad checksum line {}", account_number, parser.get_line_number());
                    }
                }
                Status::Error(error) => {
                    println!("ERROR: {}:{}: row {}: {}", error.line_number, error.col, error.row, error.message);
                }
                _ => {

                }
            }
        } else {
            println!("Error reading line.");
        }
    }
}
