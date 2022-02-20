mod parse;
mod checksum;

use parse::*;
use checksum::*;

// Transforms an input iterator into a processed output iterator
pub struct Processor<I>
    where I: Iterator<Item = String>
{
    lines: I,
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
}

impl<I> Iterator for Processor<I>
    where I: Iterator<Item = String>
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.lines.next();
            match next {
                Option::Some(line) => {
                    let status = self.parser.process_line(line);

                    match status {
                        Status::Success(account_number) => {
                            if is_checksum_valid(&account_number) {
                                return Some(account_number)
                            } else {
                                let mut alts = find_adjacent(&account_number);
                                match alts.len() {
                                    0 => return Some(format!("{} ERR [line {}]", account_number, self.parser.get_line_number())),
                                    1 => return Some(alts.remove(0)),
                                    _ => return Some(format!("{} AMB [line {} could be {:?}]",account_number, self.parser.get_line_number(), alts))
                                }
                            }
                        }
                        Status::Error(error) => {
                            return Some(format!("ERROR: {}:{}: row {}: {}", error.line_number, error.col, error.row, error.message))
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
        let output : Vec<String> = Processor::new(iter).collect();

        assert_eq!(output, vec!["490067715 AMB [line 4 could be [\"490867715\", \"490067115\", \"490067719\"]]"]);
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
        let output : Vec<String> = Processor::new(iter).collect();

        assert_eq!(output, vec!["123456789"]);
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
        let output : Vec<String> = Processor::new(iter).collect();

        assert_eq!(output, vec!["498061715 ERR [line 4]"]);
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
        let output : Vec<String> = Processor::new(iter).collect();

        assert_eq!(output, vec!["490067115"]);
    }

}