// Parser of Bank OCR account numbers
pub struct Parser {
    register: [u8; 10],
    line_number: usize,
    skip: bool,
}

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

// Parsing status for current entry
#[derive(Debug)]
pub enum Status {
    // Entry parsed successfully.  Account number is available.
    Success(String),

    // Error occurred.  Error field is populated with details.  Partially parsed account number
    // may be available depending on error.
    Error(Error),

    // Not all rows of current entry have been parsed.  Continue parsing lines.
    Incomplete
}

// Hold error information
#[derive(PartialEq, Debug)]
pub struct Error {
    // Message describing the error
    pub message : String,

    // Line number of input where error occurred
    pub line_number : usize,

    // Column number of input where error occurred
    pub col : usize,

    // Row within the entry where error occurred
    pub row : usize,
}

impl Parser {

    // Create a new parser
    pub fn new() -> Parser {
        Parser {
            register: [0; 10],
            line_number: 0,
            skip: false,
        }
    }

    pub fn get_line_number(&self) -> usize {
        self.line_number
    }

    // Process a line of input
    pub fn process_line(&mut self, line: String) -> Status {
        self.line_number += 1;

        let row = self.row();
        if row == 0 {
            self.skip = false;
            self.clear_register();
        } else if self.skip {
            return Status::Incomplete;
        }

        let mut col = 0;
        for ch in line.chars() {
            let pos = col % 3;
            let dig = col / 3;
            let on = on_char(row, pos);
            let bit_pos = bit_pos(row, pos);
            if on == '\0' {
                if ch != ' ' {
                    return self.build_error(format!("Expected space but found '{}'.", ch), col);
                }
            } else {
                if ch == on {
                    self.register[dig] |= 1 << bit_pos;
                } else if ch != ' ' {
                    return self.build_error(format!("Expected space or '{}' but found '{}'.", on, ch), col);
                }
            }
            col += 1;
        }

        return if row < 3 {
            Status::Incomplete
        } else {
            Status::Success(self.read_register())
        };
    }

    fn row(&self) -> usize {
        (self.line_number - 1) % 4
    }

    // Read account number from register and reset register to empty
    fn read_register(&mut self) -> String {
        let mut account_number = String::new();
        for index in 0..9 {
            account_number.push(reg_to_digit(self.register[index]));
        }
        account_number
    }

    fn clear_register(&mut self) {
        for index in 0..9 {
            self.register[index] = 0;
        }
    }

    // Build a parsing error
    fn build_error(&mut self, message : String, col : usize) -> Status {
        self.skip = true;
        Status::Error(Error {
            message,
            line_number: self.line_number,
            row: self.row(),
            col
        })
    }
}

// Determine the character that indicates an "on" element
pub fn on_char(row: usize, col: usize) -> char {
    match row * 10 + col {
        0 => '\0',
        1 => '_',
        2 => '\0',
        10 => '|',
        11 => '_',
        12 => '|',
        20 => '|',
        21 => '_',
        22 => '|',
        _ => '\0'
    }
}

// Determine the register bit which corresponds to an element
fn bit_pos(row: usize, col: usize) -> usize {
    match row * 10 + col {
        1 => 7,
        10 => 6,
        11 => 5,
        12 => 4,
        20 => 3,
        21 => 2,
        22 => 1,
        _ => 0
    }
}

// Determine the output character associated with a value in the register
fn reg_to_digit(val: u8) -> char {
    match val {
        222 => '0',
        18 => '1',
        188 => '2',
        182 => '3',
        114 => '4',
        230 => '5',
        238 => '6',
        146 => '7',
        254 => '8',
        246 => '9',
        _ => '?' // The value doesn't correspond to a numberical digit
    }
}

pub fn is_checksum_valid(account_number : &String) -> bool {
    if account_number.len() != 9 {
        return false;
    }

    let mut checksum = 0;
    let mut coefficient : u32 = 1;
    for ch in account_number.chars().rev() {
        match ch.to_digit(10) {
            Some(digit) => checksum += digit * coefficient,
            _ => return false
        }
        coefficient += 1;
    }

    checksum % 11 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Status {
        pub fn is_complete(&self) -> bool {
            match self {
                Status::Incomplete => false,
                _ => true,
            }
        }
    }

    #[test]
    fn can_parse_a_correctly_formatted_number() {

        assert_eq!("000000000", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "| || || || || || || || || |",
            "|_||_||_||_||_||_||_||_||_|",
            ""
        ]));

        assert_eq!("111111111", parse_to_string([
            "                           ",
            "  |  |  |  |  |  |  |  |  |",
            "  |  |  |  |  |  |  |  |  |",
            ""
        ]));

        assert_eq!("222222222", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            " _| _| _| _| _| _| _| _| _|",
            "|_ |_ |_ |_ |_ |_ |_ |_ |_ ",
            ""
        ]));

        assert_eq!("333333333", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            " _| _| _| _| _| _| _| _| _|",
            " _| _| _| _| _| _| _| _| _|",
            ""
        ]));

        assert_eq!("444444444", parse_to_string([
            "                           ",
            "|_||_||_||_||_||_||_||_||_|",
            "  |  |  |  |  |  |  |  |  |",
            ""
        ]));

        assert_eq!("555555555", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "|_ |_ |_ |_ |_ |_ |_ |_ |_ ",
            " _| _| _| _| _| _| _| _| _|",
            ""
        ]));

        assert_eq!("666666666", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "|_ |_ |_ |_ |_ |_ |_ |_ |_ ",
            "|_||_||_||_||_||_||_||_||_|",
            ""
        ]));

        assert_eq!("777777777", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "  |  |  |  |  |  |  |  |  |",
            "  |  |  |  |  |  |  |  |  |",
            ""
        ]));

        assert_eq!("888888888", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "|_||_||_||_||_||_||_||_||_|",
            "|_||_||_||_||_||_||_||_||_|",
            ""
        ]));

        assert_eq!("999999999", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "|_||_||_||_||_||_||_||_||_|",
            " _| _| _| _| _| _| _| _| _|",
            ""
        ]));

        assert_eq!("123456789", parse_to_string([
            "    _  _     _  _  _  _  _ ",
            "  | _| _||_||_ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _|",
            ""
        ]));

        assert_eq!("000000051", parse_to_string([
            " _  _  _  _  _  _  _  _    ",
            "| || || || || || || ||_   |",
            "|_||_||_||_||_||_||_| _|  |",
            ""
        ]));

    }

    #[test]
    fn illegible_digits_receive_placeholder() {
        assert_eq!("49006771?", parse_to_string([
            "    _  _  _  _  _  _     _ ",
            "|_||_|| || ||_   |  |  | _ ",
            "  | _||_||_||_|  |  |  | _|",
            ""
        ]));

        assert_eq!("1234?678?", parse_to_string([
            "    _  _     _  _  _  _  _ ",
            "  | _| _||_| _ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _ ",
            ""
        ]));
    }

    #[test]
    fn illegal_characters_produce_an_error() {
        // Character not in the set (' ', '|', '_')
        assert_eq!("ERROR: 2:7: row 1: Expected space or '_' but found 'x'.", parse_to_string([
            "    _  _     _  _  _  _  _ ",
            "  | _| x||_||_ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _|",
            ""
        ]));

        // top "corners" must be spaces
        assert_eq!("ERROR: 1:21: row 0: Expected space but found 'Q'.", parse_to_string([
            "    _  _     _  _  _ Q_  _ ",
            "  | _| _||_||_ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _|",
            ""
        ]));

        assert_eq!("ERROR: 1:23: row 0: Expected space but found '|'.", parse_to_string([
            "    _  _     _  _  _  _| _ ",
            "  | _| _||_||_ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _|",
            ""
        ]));

        // '_' and '|' flipped
        assert_eq!("ERROR: 3:22: row 2: Expected space or '_' but found '|'.", parse_to_string([
            "    _  _     _  _  _  _  _ ",
            "  | _| _||_||_ |_   ||_||_|",
            "  ||_  _|  | _||_|  |||| _|",
            ""
        ]));

        assert_eq!("ERROR: 2:12: row 1: Expected space or '|' but found '_'.", parse_to_string([
            "    _  _     _  _  _  _  _ ",
            "  | _| _||_|__ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _|",
            ""
        ]));
    }
    #[test]
    fn recovers_after_error() {
        // Error recovery
        let mut parser = Parser::new();
        assert!(is_incomplete(parser.process_line("    _  _  _  _  _  _     _ ".to_string())));
        assert!(is_incomplete(parser.process_line("|_||_|| || ||_   |  |  ||_ ".to_string())));
        assert!(is_incomplete(parser.process_line("  | _||_||_||_|  |  |  | _|".to_string())));
        assert_eq!("490067715".to_string(), get_account_number(parser.process_line("".to_string())));
        assert!(is_incomplete(parser.process_line("    _  _     _  _  _  _  _ ".to_string())));
        assert!(is_error     (parser.process_line("  | _| x||_||_ |_   ||_||_|".to_string())));
        assert!(is_incomplete(parser.process_line("  | _| x||_||_ |_   ||_||_|".to_string())));
        assert!(is_incomplete(parser.process_line("".to_string())));
        assert!(is_incomplete(parser.process_line("    _  _  _  _  _  _     _ ".to_string())));
        assert!(is_incomplete(parser.process_line("|_||_|| || ||_   |  |  ||_ ".to_string())));
        assert!(is_incomplete(parser.process_line("  | _||_||_||_|  |  |  ||_|".to_string())));
        assert_eq!("490067716".to_string(), get_account_number(parser.process_line("".to_string())));

    }

    #[test]
    fn can_validate_checksum() {
        assert!(!is_checksum_valid(&"00000019".to_string()), "checksome");
        assert!( is_checksum_valid(&"000000019".to_string()), "checksome");
        assert!(!is_checksum_valid(&"0000000019".to_string()), "checksome");

        assert!(is_checksum_valid(&"000000000".to_string()), "checksome");
        assert!(!is_checksum_valid(&"000000001".to_string()), "checksome");
        assert!(is_checksum_valid(&"500000301".to_string()), "checksome");
        assert!(is_checksum_valid(&"135802539".to_string()), "checksome");
    }

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
        println!("{:?}", output);
    }

    #[test]
    fn do_iter() {
        let input = vec![
            "    _  _  _  _  _  _     _ ".to_string(),
            "|_||_|| || ||_   |  |  ||_ ".to_string(),
            "  | _||_||_||_|  |  |  | _|".to_string(),
            "".to_string()
        ];
        let iter = input.iter().map(|s| s.to_string());
        let it = Processor::new(iter);
        for s in it {
            println!("{}", s);
        }
    }

    fn parse_entry(lines : [&str; 4]) -> Status {
        let mut parser = Parser::new();

        for line in lines {
            let status = parser.process_line(String::from(line));
            if status.is_complete() {
                return status;
            }
        }

        Status::Incomplete
    }

    fn parse_to_string(lines : [&str; 4]) -> String {
        let status = parse_entry(lines);
        match status {
            Status::Success(account_number) => {
                println!("{}", account_number);
                account_number
            }
            Status::Error(error) => {
                format!("ERROR: {}:{}: row {}: {}", error.line_number, error.col, error.row, error.message)
            }
            _ => {
                String::from("Unexpected")
            }
        }
    }

    fn get_account_number(status: Status) -> String {
        match status {
            Status::Success(account_number) => account_number,
            _ => panic!("Status is not success"),
        }
    }

    fn is_error(status : Status) -> bool {
        match status {
            Status::Error(_) => true,
            _ => false,
        }
    }

    fn is_incomplete(status : Status) -> bool {
        match status {
            Status::Incomplete => true,
            _ => false,
        }
    }

}