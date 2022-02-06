// Parser of Bank OCR account numbers
pub struct Parser {
    register: [u8; 10],
    line_number: usize,
    skip: bool,
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

impl Status {
    pub fn is_complete(&self) -> bool {
        match self {
            Status::Incomplete => false,
            _ => true,
        }
    }
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