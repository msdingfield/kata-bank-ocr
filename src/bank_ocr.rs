// Parser of Bank OCR account numbers
pub struct Parser {
    register: [u8; 10],
    line_number: usize,
    error : Option<Error>,
}

// Parsing status for current entry
#[derive(PartialEq)]
pub enum Status {
    // Entry parsed successfully.  Account number is available.
    Success,

    // Error occurred.  Error field is populated with details.  Partially parsed account number
    // may be available depending on error.
    Error,

    // Not all rows of current entry have been parsed.  Continue parsing lines.
    Incomplete
}

// Entry for each account number parsed
pub struct Entry {
    // Parsing status of the current entry
    pub status: Status,

    // Parse account number, if any
    pub account_number: Option<String>,

    // Parse error, if any
    pub error : Option<Error>,
}

impl Entry {
    // Test if parsing of current entry is complete
    pub fn is_complete(&self) -> bool {
        self.status != Status::Incomplete
    }
}

// Hold error information
#[derive(Clone)]
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
            error: Option::None,
        }
    }

    // Process a line of input
    pub fn process_line(&mut self, line: String) -> Entry {
        let row = self.line_number % 4;

        let mut col = 0;
        for ch in line.chars() {
            let pos = col % 3;
            let dig = col / 3;
            let on = on_char(row, pos);
            let bit_pos = bit_pos(row, pos);
            if on == '\0' {
                if ch != ' ' {
                    self.emit_error(
                        format!("Expected space but found '{}'.", ch),
                        row,
                        col
                    );
                    break;
                }
            } else {
                if ch == on {
                    self.register[dig] |= 1 << bit_pos;
                } else if ch != ' ' {
                    self.emit_error(
                        format!("Expected space or '{}' but found '{}'.", on, ch),
                        row,
                        col,
                    );
                    break;
                }
            }
            col += 1;
        }

        let entry = self.build_entry(row);

        self.error = Option::None;
        self.line_number += 1;

        return entry;
    }

    // Build entry response for the current line
    fn build_entry(&mut self, row : usize) -> Entry {
        let account_number =
            if row == 3 {
                Option::Some(self.read_and_clear_register())
            } else {
                Option::None
            };

        let status =
            if self.error.is_some() {
                Status::Error
            } else if row < 3 {
                Status::Incomplete
            } else {
                Status::Success
            };

        Entry {
            status,
            account_number,
            error: self.error.clone(),
        }
    }

    // Read account number from register and reset register to empty
    fn read_and_clear_register(&mut self) -> String {
        let mut account_number = String::new();
        for index in 0..9 {
            account_number.push(reg_to_digit(self.register[index]));
            self.register[index] = 0;
        }
        account_number
    }

    // Emit a parsing error
    fn emit_error(&mut self, message : String, row : usize, col : usize) {
        self.error = Option::Some(
            Error {
                message,
                line_number: self.line_number,
                row,
                col
            }
        );
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

