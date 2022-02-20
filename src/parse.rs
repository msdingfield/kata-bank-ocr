use std::str;

// Parser of Bank OCR account numbers
pub struct Parser {
    register: [u8; 10],
    line_number: usize,
    skip: bool,
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

// Parsing status for current entry
#[derive(Debug)]
pub enum Status {
    // Entry parsed successfully.  Account number is available.
    Success(String),

    BadDigits {
        account_number: String,
        alternates: Vec<String>,
    },

    // Error occurred.  Error field is populated with details.  Partially parsed account number
    // may be available depending on error.
    Error(Error),

    // Not all rows of current entry have been parsed.  Continue parsing lines.
    Incomplete
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

            if !ch.is_whitespace() && dig > 8 {
                return self.build_error(format!("Input line is too long."), col);
            } else if on == '\0' {
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
            let (success, account_number, alternates) = self.read_register();
            if success {
                Status::Success(account_number)
            } else {
                Status::BadDigits {account_number, alternates}
            }
        };
    }

    fn row(&self) -> usize {
        (self.line_number - 1) % 4
    }

    // Read account number from register and reset register to empty
    fn read_register(&mut self) -> (bool, String, Vec<String>) {
        let mut buffer : [u8;9] = [0;9];

        let mut bad = 0;
        let mut last_bad = 0usize;
        for index in 0..9 {
            let digit = reg_to_digit(self.register[index]);
            buffer[index] = digit as u8;
            if digit == '?' {
                bad += 1;
                last_bad = index;
            }
        }
        let account_number = str::from_utf8(&buffer).unwrap().to_string();

        let mut alts = Vec::new();
        match bad {
            0 => (true, account_number, alts),
            1 => {
                let alt_digits = possible_digits(self.register[last_bad]);
                alt_digits.iter().for_each(|dig| {
                    buffer[last_bad] = *dig;
                    alts.push(str::from_utf8(&buffer).unwrap().to_string());
                });
                (false, account_number, alts)
            }
            _ => (false, account_number, alts)
        }


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
fn on_char(row: usize, col: usize) -> char {
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

fn possible_digits(reg: u8) -> Vec<u8> {
    let mut alts = Vec::new();
    for n in 1..=7 {
        let alt = reg_to_digit(reg ^ (1 << n));
        if alt != '?' {
            alts.push(alt as u8);
        }
    }
    alts
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
        _ => '?' // The value doesn't correspond to a numerical digit
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn possible_alternative_digits() {
        assert_eq!(possible_digits(100), vec![]);
        assert_eq!(possible_digits(94), vec!['0' as u8]);
        assert_eq!(possible_digits(2), vec!['1' as u8]);
        assert_eq!(possible_digits(60), vec!['2' as u8]);
        assert_eq!(possible_digits(54), vec!['3' as u8]);
        assert_eq!(possible_digits(98), vec!['4' as u8]);
        assert_eq!(possible_digits(102), vec!['5' as u8]);
        assert_eq!(possible_digits(110), vec!['6' as u8]);
        assert_eq!(possible_digits(18), vec!['7' as u8]);
        assert_eq!(possible_digits(126), vec!['8' as u8]);
        assert_eq!(possible_digits(182), vec!['9' as u8]);
        assert_eq!(possible_digits(82), vec!['4' as u8, '1' as u8]);
        assert_eq!(possible_digits(190), vec!['2' as u8, '3' as u8, '8' as u8]);
    }

    #[test]
    fn too_many_digits_produces_an_error() {
        assert_eq!("ERROR: 1:28: row 0: Input line is too long.", parse_to_string([
            " _  _  _  _  _  _  _  _  _  _ ",
            "| || || || || || || || || || |",
            "|_||_||_||_||_||_||_||_||_||_|",
            ""
        ]));
    }

    #[test]
    fn too_few_digits_is_treated_as_illegible() {
        assert_eq!("ILLEGIBLE: 00000000? []", parse_to_string([
            " _  _  _  _  _  _  _  _ ",
            "| || || || || || || || |",
            "|_||_||_||_||_||_||_||_|",
            ""
        ]));
    }

    #[test]
    fn can_parse_a_correctly_formatted_number() {

        assert_eq!("SUCCESS: 000000000", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "| || || || || || || || || |",
            "|_||_||_||_||_||_||_||_||_|",
            ""
        ]));

        assert_eq!("SUCCESS: 111111111", parse_to_string([
            "                           ",
            "  |  |  |  |  |  |  |  |  |",
            "  |  |  |  |  |  |  |  |  |",
            ""
        ]));

        assert_eq!("SUCCESS: 222222222", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            " _| _| _| _| _| _| _| _| _|",
            "|_ |_ |_ |_ |_ |_ |_ |_ |_ ",
            ""
        ]));

        assert_eq!("SUCCESS: 333333333", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            " _| _| _| _| _| _| _| _| _|",
            " _| _| _| _| _| _| _| _| _|",
            ""
        ]));

        assert_eq!("SUCCESS: 444444444", parse_to_string([
            "                           ",
            "|_||_||_||_||_||_||_||_||_|",
            "  |  |  |  |  |  |  |  |  |",
            ""
        ]));

        assert_eq!("SUCCESS: 555555555", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "|_ |_ |_ |_ |_ |_ |_ |_ |_ ",
            " _| _| _| _| _| _| _| _| _|",
            ""
        ]));

        assert_eq!("SUCCESS: 666666666", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "|_ |_ |_ |_ |_ |_ |_ |_ |_ ",
            "|_||_||_||_||_||_||_||_||_|",
            ""
        ]));

        assert_eq!("SUCCESS: 777777777", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "  |  |  |  |  |  |  |  |  |",
            "  |  |  |  |  |  |  |  |  |",
            ""
        ]));

        assert_eq!("SUCCESS: 888888888", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "|_||_||_||_||_||_||_||_||_|",
            "|_||_||_||_||_||_||_||_||_|",
            ""
        ]));

        assert_eq!("SUCCESS: 999999999", parse_to_string([
            " _  _  _  _  _  _  _  _  _ ",
            "|_||_||_||_||_||_||_||_||_|",
            " _| _| _| _| _| _| _| _| _|",
            ""
        ]));

        assert_eq!("SUCCESS: 123456789", parse_to_string([
            "    _  _     _  _  _  _  _ ",
            "  | _| _||_||_ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _|",
            ""
        ]));

        assert_eq!("SUCCESS: 000000051", parse_to_string([
            " _  _  _  _  _  _  _  _    ",
            "| || || || || || || ||_   |",
            "|_||_||_||_||_||_||_| _|  |",
            ""
        ]));

    }

    #[test]
    fn illegible_digits_receive_placeholder() {
        assert_eq!("ILLEGIBLE: 49006771? [\"490067713\", \"490067715\"]", parse_to_string([
            "    _  _  _  _  _  _     _ ",
            "|_||_|| || ||_   |  |  | _ ",
            "  | _||_||_||_|  |  |  | _|",
            ""
        ]));

        assert_eq!("ILLEGIBLE: 1234?678? []", parse_to_string([
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
        assert!(is_incomplete(&parser.process_line("    _  _  _  _  _  _     _ ".to_string())));
        assert!(is_incomplete(&parser.process_line("|_||_|| || ||_   |  |  ||_ ".to_string())));
        assert!(is_incomplete(&parser.process_line("  | _||_||_||_|  |  |  | _|".to_string())));
        assert_eq!("490067715".to_string(), get_account_number(parser.process_line("".to_string())));
        assert!(is_incomplete(&parser.process_line("    _  _     _  _  _  _  _ ".to_string())));
        assert!(is_error     (&parser.process_line("  | _| x||_||_ |_   ||_||_|".to_string())));
        assert!(is_incomplete(&parser.process_line("  | _| x||_||_ |_   ||_||_|".to_string())));
        assert!(is_incomplete(&parser.process_line("".to_string())));
        assert!(is_incomplete(&parser.process_line("    _  _  _  _  _  _     _ ".to_string())));
        assert!(is_incomplete(&parser.process_line("|_||_|| || ||_   |  |  ||_ ".to_string())));
        assert!(is_incomplete(&parser.process_line("  | _||_||_||_|  |  |  ||_|".to_string())));
        assert_eq!("490067716".to_string(), get_account_number(parser.process_line("".to_string())));

    }

    fn parse_entry(lines : [&str; 4]) -> Status {
        let mut parser = Parser::new();

        for line in lines {
            let status = parser.process_line(String::from(line));
            if is_complete(&status) {
                return status;
            }
        }

        Status::Incomplete
    }

    fn parse_to_string(lines : [&str; 4]) -> String {
        let status = parse_entry(lines);
        match status {
            Status::Success(account_number) => {
                format!("SUCCESS: {}", account_number)
            }
            Status::BadDigits { account_number, alternates } => {
                format!("ILLEGIBLE: {} {:?}", account_number, alternates)
            }
            Status::Error(error) => {
                format!("ERROR: {}:{}: row {}: {}", error.line_number, error.col, error.row, error.message)
            }
            Status::Incomplete => {
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

    fn is_complete(status : &Status) -> bool {
        match status {
            Status::Incomplete => false,
            _ => true,
        }
    }

    fn is_error(status : &Status) -> bool {
        match status {
            Status::Error(_) => true,
            _ => false,
        }
    }

    fn is_incomplete(status : &Status) -> bool {
        match status {
            Status::Incomplete => true,
            _ => false,
        }
    }

}