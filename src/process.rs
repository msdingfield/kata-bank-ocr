use crate::{Error, find_adjacent, is_checksum_valid, Parser, Status};
use crate::process::Result::Success;
use crate::Result::{BadChecksum, BadDigits, InvalidCharacter};

pub enum Result {
    Success { account_number : String, line_number : u32 },
    BadChecksum { account_number : String, alternates : Vec<String>, line_number : u32 },
    BadDigits { account_number : String, alternates : Vec<String>, line_number : u32 },
    InvalidCharacter { error : Error },
}

// Transforms an input iterator into a processed output iterator
pub struct Processor<I>
    where I: Iterator<Item = String>
{
    pub lines: I,
    pub parser: Parser,
}

impl<I> Processor<I>
    where I: Iterator<Item = String>
{
    pub fn new(lines: I) -> Processor<I>{
        Processor {
            lines,
            parser: Parser::new()
        }
    }

    fn success(&self, account_number : String) -> Option<Result> {
        Some(Success {
           account_number,
            line_number: self.parser.get_line_number() as u32,
        })
    }

    fn bad_checksum(&self, account_number : String, alternates : Vec<String>) -> Option<Result> {
        Some(BadChecksum {
            account_number,
            line_number: self.parser.get_line_number() as u32,
            alternates
        })
    }

    fn bad_digits(&self, account_number : String, alternates : Vec<String>) -> Option<Result> {
        Some(BadDigits {
            account_number,
            line_number: self.parser.get_line_number() as u32,
            alternates
        })
    }

    fn invalid_character(&self, error : Error) -> Option<Result> {
        Some(InvalidCharacter { error })
    }
}

impl<I> Iterator for Processor<I>
    where I: Iterator<Item = String>
{
    type Item = Result;

    fn next(&mut self) -> Option<Self::Item> {

        loop {
            let next = self.lines.next();
            match next {
                Option::Some(line) => {
                    let status = self.parser.process_line(line);

                    match status {
                        Status::Success(account_number) => {
                            if is_checksum_valid(&account_number) {
                                return self.success(account_number)
                            } else {
                                let alts = find_adjacent(&account_number);
                                match alts.len() {
                                    0 => return self.bad_checksum(account_number, alts),
                                    1 => return self.bad_checksum(account_number, alts),
                                    _ => return self.bad_checksum(account_number, alts),
                                }
                            }
                        }
                        Status::BadDigits { account_number, alternates } => {
                            return self.bad_digits(
                                account_number,
                                alternates
                                    .into_iter()
                                    .filter(|alt| is_checksum_valid(alt))
                                    .collect()
                            );
                        }
                        Status::Error(error) => {
                            return self.invalid_character(error);
                            // return self.success(format!("ERROR: {}:{}: row {}: {}", error.line_number, error.col, error.row, error.message))
                        }
                        Status::Incomplete => {
                            // Keep going if parse of number is incomplete
                        }
                    }
                }

                Option::None => return Option::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_checksum_with_alts() {
        let input = vec![
            "    _  _  _  _  _  _     _ ".to_string(),
            "|_||_|| || ||_   |  |  ||_ ".to_string(),
            "  | _||_||_||_|  |  |  | _|".to_string(),
            "".to_string()
        ];
        let iter = input.iter().map(|s| s.to_string());
        let output : Vec<Result> = Processor::new(iter).collect();
        assert_eq!(output.len(), 1);

        if let BadChecksum { account_number, line_number, alternates} = &output[0] {
            assert_eq!(account_number, "490067715");
            assert_eq!(*line_number, 4);
            assert_eq!(*alternates, vec!["490867715", "490067115", "490067719"]);
        } else {
            panic!("Not BadChecksum variant");
        }
    }

    #[test]
    fn bad_checksum_single_alt() {
        let input = vec![ // 723456789
                          " _  _  _     _  _  _  _  _ ".to_string(),
                          "  | _| _||_||_ |_   ||_||_|".to_string(),
                          "  ||_  _|  | _||_|  ||_| _|".to_string(),
                          "".to_string()
        ];
        let iter = input.iter().map(|s| s.to_string());
        let output : Vec<Result> = Processor::new(iter).collect();
        assert_eq!(output.len(), 1);

        if let BadChecksum { account_number, line_number, alternates} = &output[0] {
            assert_eq!(account_number, "723456789");
            assert_eq!(*line_number, 4);
            assert_eq!(*alternates, vec!["123456789"]);
        } else {
            panic!("Not BadChecksum variant")
        }
    }

    #[test]
    fn bad_checksum_no_alt() {
        let input = vec![
            "    _  _  _  _     _     _ ".to_string(),
            "|_||_||_|| ||_   |  |  ||_ ".to_string(),
            "  | _||_||_||_|  |  |  | _|".to_string(),
            "".to_string()
        ];
        let iter = input.iter().map(|s| s.to_string());
        let output : Vec<Result> = Processor::new(iter).collect();
        assert_eq!(output.len(), 1);

        if let BadChecksum { account_number, line_number, alternates} = &output[0] {
            assert_eq!(account_number, "498061715");
            assert_eq!(*line_number, 4);
            assert_eq!(alternates.len(), 0);
        } else {
            panic!("Not BadChecksum variant")
        }
    }

    #[test]
    fn valid_number() {
        let input = vec![
            "    _  _  _  _  _        _ ".to_string(),
            "|_||_|| || ||_   |  |  ||_ ".to_string(),
            "  | _||_||_||_|  |  |  | _|".to_string(),
            "".to_string()
        ];
        let iter = input.iter().map(|s| s.to_string());
        let output : Vec<Result> = Processor::new(iter).collect();
        assert_eq!(output.len(), 1);

        if let Success { account_number, line_number} = &output[0] {
            assert_eq!(account_number, "490067115");
            assert_eq!(*line_number, 4);
        } else {
            panic!("Not Success variant")
        }
    }

}