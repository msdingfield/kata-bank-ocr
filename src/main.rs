use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use bankocr::{format_line, Processor};

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

    Processor::new(
        reader.lines().flat_map(|line| line)
    ).map(format_line).for_each(|out_line| {
        let result = writeln!(writer, "{}", out_line);
        if let Err(error) = result {
            panic!("Error writing to output: {}", error);
        }
    });

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