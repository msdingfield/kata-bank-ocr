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
                                return Some(format!("{} bad checksum line {}", account_number, self.parser.get_line_number()))
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
        //     let status = parser.process_line(line);
        //
        //     match status {
        //         Status::Success(account_number) => {
        //             if is_checksum_valid(&account_number) {
        //                 return account_number
        //             } else {
        //                 return Ok(format!("{} bad checksum line {}", account_number, parser.get_line_number()))
        //             }
        //         }
        //         Status::Error(error) => {
        //             return Ok(format!("ERROR: {}:{}: row {}: {}", error.line_number, error.col, error.row, error.message))
        //         }
        //         Status::Incomplete => {
        //             // Keep going if parse of number is incomplete
        //             return Err("skip".to_string())
        //         }
        //     }
        // }
        // Option::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_multiple_lines() {
        let input = vec![
            "    _  _  _  _  _  _     _ ".to_string(),
            "|_||_|| || ||_   |  |  ||_ ".to_string(),
            "  | _||_||_||_|  |  |  | _|".to_string(),
            "".to_string()
        ];
        let iter = input.iter().map(|s| s.to_string());
        let output : Vec<String> = Processor::new(iter).collect();

        assert_eq!(output, vec!["490067715 bad checksum line 4"]);
    }

}