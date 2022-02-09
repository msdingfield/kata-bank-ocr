use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use bankocr::{read_account_numbers};

fn main() -> io::Result<()> {
    let args : Vec<String> = env::args().collect();
    if args.len() == 3 {
        process_file(&args[1], &args[2])?;
    } else {
        println!("Usage: bank_ocr <input file> <output file>");
    }
    Ok(())
}

fn process_file(input: &String, output: &String) -> io::Result<()> {
    println!("Parsing {} into {}", input, output);

    let reader = open_input(input)?;
    let mut writer = open_output(output)?;

    read_account_numbers(
        reader.lines().flat_map(|line| line),
        |out_line| {
            let result = writeln!(writer, "{}", out_line);
            if result.is_err() {
                panic!("Error writing to output: {}", result.err().unwrap())
            }
        }
    );

    Ok(())
}

fn open_input(input: &String) -> io::Result<BufReader<File>> {
    let input_result = File::open(input);
    match input_result {
        Result::Err(error) => {
            println!("Error opening input file {}.", input);
            return io::Result::Err(error);
        }

        Result::Ok(file) => {
            return io::Result::Ok(BufReader::new(file))
        }
    }
}

fn open_output(output: &String) -> io::Result<File> {
    let output_result = File::create(output);
    match output_result {
        Result::Err(error) => {
            println!("Error opening output file {}.", output);
            return io::Result::Err(error);
        }

        Result::Ok(file) => {
            return io::Result::Ok(file);
        }
    }
}