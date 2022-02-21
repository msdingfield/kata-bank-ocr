use std::str;

const ILLEGIBLE : u8 = '?' as u8;

// Parser of Bank OCR account numbers
pub struct Parser {
    // Register to hold the state of each segment of the 9 digits
    // Each digit is imagined as a 7 segment LED display so the status of the 7 segments can
    // be stored in a byte.  Input is scanned and the bits corresponding to "on" segments are set.
    // Patterns of bits corresponding to valid numbers are mapped to the corresponding character.
    register: [u8; 9],

    // The current line number
    line_number: usize,

    // Flag to skip over lines in case of error
    skip: bool,
}

// Parsing status for current entry
#[derive(Debug)]
pub enum Status {
    // Entry parsed successfully.  Account number is available.
    Success(String),

    // Entry parsed but one or more digits are unreadable
    BadDigits {

        // parsed number with '?' in place of unreadable digits
        account_number: String,

        // possible account numbers that are very similar to the parsed number
        alternates: Vec<String>,
    },

    // Error occurred.  Error field is populated with details.
    // For example an invalid character will produce an error.
    Error {
        // Message describing the error
        message : String,

        // Line number of input where error occurred
        line_number : usize,

        // Column number of input where error occurred
        col : usize,

        // Row within the entry where error occurred
        row : usize,
    },

    // Not all rows of current entry have been parsed.  Continue parsing lines.
    Incomplete
}

impl Parser {

    // Create a new parser
    pub fn new() -> Parser {
        Parser {
            register: [0; 9],
            line_number: 0,
            skip: false,
        }
    }

    // Get the current line number
    pub fn get_line_number(&self) -> usize {
        self.line_number
    }

    // Process a line of input
    pub fn process_line(&mut self, line: &str) -> Status {
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

    // Get row within the entry currently being parsed
    fn row(&self) -> usize {
        (self.line_number - 1) % 4
    }

    // Read account number from register and reset register to empty
    fn read_register(&self) -> (bool, String, Vec<String>) {
        let mut buffer : [u8;9] = [0;9];

        let mut bad_cnt = 0; // count of illegible digits
        let mut last_bad = 0usize; // index of last illegible digit
        for index in 0..9 {
            let digit = read_register_digit(self.register[index]);
            buffer[index] = digit;
            if digit == ILLEGIBLE {
                bad_cnt += 1;
                last_bad = index;
            }
        }
        let account_number = str::from_utf8(&buffer).unwrap().to_string();

        let mut alts = Vec::new();
        match bad_cnt {
            0 => (true, account_number, alts),
            1 => {
                let alt_digits = find_register_digit_close_matches(self.register[last_bad]);
                alt_digits.iter().for_each(|dig| {
                    buffer[last_bad] = *dig;
                    alts.push(str::from_utf8(&buffer).unwrap().to_string());
                });
                (false, account_number, alts)
            }
            _ => (false, account_number, alts)
        }
    }

    // Clear contents of register
    fn clear_register(&mut self) {
        self.register.fill(0);
    }

    // Build a parsing error
    fn build_error(&mut self, message : String, col : usize) -> Status {
        self.skip = true;
        Status::Error {
            message,
            line_number: self.line_number,
            col,
            row: self.row(),
        }
    }
}

// Determine the character that indicates an "on" element
fn on_char(row: usize, col: usize) -> char {
    match row << 4 | col {
        0x00 => '\0',
        0x01 => '_',
        0x02 => '\0',
        0x10 => '|',
        0x11 => '_',
        0x12 => '|',
        0x20 => '|',
        0x21 => '_',
        0x22 => '|',
        _ => '\0'
    }
}

// Find digits that are a close match to the register element
fn find_register_digit_close_matches(reg_element: u8) -> Vec<u8> {
    let mut close_matches = Vec::new();
    for n in 0..=6 {
        let close_match = read_register_digit(reg_element ^ (1 << n));
        if close_match != ILLEGIBLE {
            close_matches.push(close_match as u8);
        }
    }
    close_matches
}

// Determine the output character associated with a value in the register
fn read_register_digit(reg_element: u8) -> u8 {
    /*
    Bit positions for each segment
    -0-
    123
    456
     */
    match reg_element {
        0b01111011 => '0' as u8,
        0b01001000 => '1' as u8,
        0b00111101 => '2' as u8,
        0b01101101 => '3' as u8,
        0b01001110 => '4' as u8,
        0b01100111 => '5' as u8,
        0b01110111 => '6' as u8,
        0b01001001 => '7' as u8,
        0b01111111 => '8' as u8,
        0b01101111 => '9' as u8,
        _ => ILLEGIBLE // The value doesn't correspond to a numerical digit
    }
}

// Determine the register bit which corresponds to an element
fn bit_pos(row: usize, col: usize) -> usize {
    match row << 4 | col {
        0x01 => 0,
        0x10 => 1,
        0x11 => 2,
        0x12 => 3,
        0x20 => 4,
        0x21 => 5,
        0x22 => 6,
        _  => 7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn possible_alternative_digits() {
        assert_eq!(find_register_digit_close_matches(0b00100110), vec![]);
        assert_eq!(find_register_digit_close_matches(0b01111010), vec!['0' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01000000), vec!['1' as u8]);
        assert_eq!(find_register_digit_close_matches(0b00111100), vec!['2' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01101100), vec!['3' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01000110), vec!['4' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01100110), vec!['5' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01110110), vec!['6' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01001000), vec!['7' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01111110), vec!['8' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01101101), vec!['9' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01001010), vec!['1' as u8, '4' as u8]);
        assert_eq!(find_register_digit_close_matches(0b01111101), vec!['8' as u8, '3' as u8, '2' as u8]);
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
        assert_eq!("ILLEGIBLE: 49006771? [\"490067715\", \"490067713\"]", parse_to_string([
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
        assert!(is_incomplete(&parser.process_line("    _  _  _  _  _  _     _ ")));
        assert!(is_incomplete(&parser.process_line("|_||_|| || ||_   |  |  ||_ ")));
        assert!(is_incomplete(&parser.process_line("  | _||_||_||_|  |  |  | _|")));
        assert_eq!("490067715".to_string(), get_account_number(parser.process_line("")));
        assert!(is_incomplete(&parser.process_line("    _  _     _  _  _  _  _ ")));
        assert!(is_error     (&parser.process_line("  | _| x||_||_ |_   ||_||_|")));
        assert!(is_incomplete(&parser.process_line("  | _| x||_||_ |_   ||_||_|")));
        assert!(is_incomplete(&parser.process_line("")));
        assert!(is_incomplete(&parser.process_line("    _  _  _  _  _  _     _ ")));
        assert!(is_incomplete(&parser.process_line("|_||_|| || ||_   |  |  ||_ ")));
        assert!(is_incomplete(&parser.process_line("  | _||_||_||_|  |  |  ||_|")));
        assert_eq!("490067716".to_string(), get_account_number(parser.process_line("")));

    }

    fn parse_entry(lines : [&str; 4]) -> Status {
        let mut parser = Parser::new();

        for line in lines {
            let status = parser.process_line(line);
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
            Status::Error { message , line_number, col, row} => {
                format!("ERROR: {}:{}: row {}: {}", line_number, col, row, message)
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
            Status::Error{..} => true,
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