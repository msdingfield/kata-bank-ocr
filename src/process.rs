use crate::{find_adjacent, is_checksum_valid, Parser, Status};
use crate::Result::{Success, BadChecksum, BadDigits, Error};

// Result for a single entry
pub enum Result {
    // Account number parsed and passes checksum
    Success {
        account_number : String, // Parsed account number
        line_number : u32        // Line number of entry
    },

    // Account number parsed successfully but checksum failed
    BadChecksum {
        account_number : String,  // Parsed account number
        alternates : Vec<String>, // Numbers similar to the account number with valid checksum
                                  // it is likely there was a scanner misread and one of these is
                                  // the actual account number
        line_number : u32         // Line number of entry
    },

    // One or more digits was illegible
    BadDigits {
        account_number : String,  // Parsed account number. '?' character fills illegible digits
        alternates : Vec<String>, // Possible numbers found be looking for close matches for illegible digit
        line_number : u32         // Line number of entry
    },

    // Parse error, the input file is invalid
    Error {
        message : String,  // Message describing the nature of the error
        line_number : u32, // Line number where error occurred
        col : u32,         // Column number where error occurred
        row : u32          // Row within the entry being parsed where the error occurred
    },
}

// Transforms an input iterator into a processed output iterator
pub struct Processor<I>
    where I: Iterator<Item = String>
{
    // Iterator supplying input lines
    lines: I,

    // Input parser
    parser: Parser,
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

    // Create a Success result
    fn success(&self, account_number : String) -> Option<Result> {
        Some(Success {
           account_number,
            line_number: self.parser.get_line_number() as u32,
        })
    }

    // Create a BadChecksum result
    fn bad_checksum(&self, account_number : String, alternates : Vec<String>) -> Option<Result> {
        Some(BadChecksum {
            account_number,
            line_number: self.parser.get_line_number() as u32,
            alternates
        })
    }

    // Create a BadDigits result
    fn bad_digits(&self, account_number : String, alternates : Vec<String>) -> Option<Result> {
        Some(BadDigits {
            account_number,
            line_number: self.parser.get_line_number() as u32,
            alternates
        })
    }

    // Create an Error result
    fn error(&self, message : String, line_number : u32, col : u32, row : u32) -> Option<Result> {
        Some(Error { message, line_number, col, row})
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
                    let status = self.parser.process_line(&line);

                    match status {
                        Status::Success(account_number) => {
                            if is_checksum_valid(&account_number) {
                                return self.success(account_number)
                            } else {
                                let alternates = find_adjacent(&account_number);
                                return self.bad_checksum(account_number, alternates);
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
                        Status::Error{message, line_number, col, row} => {
                            return self.error(
                                message,
                                line_number as u32,
                                col as u32,
                                row as u32
                            );
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