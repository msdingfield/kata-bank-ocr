use crate::{bank_ocr, Status};
use crate::bank_ocr::is_checksum_valid;

pub fn do_tests() {
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

    assert_eq!("ERROR: 2:7: row 1: Expected space or '_' but found 'x'.", parse_to_string([
        "    _  _     _  _  _  _  _ ",
        "  | _| x||_||_ |_   ||_||_|",
        "  ||_  _|  | _||_|  ||_| _|",
        ""
    ]));

    assert_eq!("ERROR: 1:21: row 0: Expected space but found 'Q'.", parse_to_string([
        "    _  _     _  _  _ Q_  _ ",
        "  | _| x||_||_ |_   ||_||_|",
        "  ||_  _|  | _||_|  ||_| _|",
        ""
    ]));

    // Error recovery
    let mut parser = bank_ocr::Parser::new();
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

    assert!(!is_checksum_valid(&"00000019".to_string()), "checksome");
    assert!( is_checksum_valid(&"000000019".to_string()), "checksome");
    assert!(!is_checksum_valid(&"0000000019".to_string()), "checksome");

    assert!(is_checksum_valid(&"000000000".to_string()), "checksome");
    assert!(!is_checksum_valid(&"000000001".to_string()), "checksome");
    assert!(is_checksum_valid(&"500000301".to_string()), "checksome");
    assert!(is_checksum_valid(&"135802539".to_string()), "checksome");
}

fn get_account_number(status: Status) -> String {
    match status {
        Status::Success(account_number) => account_number,
        _ => panic!("Status is not success"),
    }
}

fn is_incomplete(status : Status) -> bool {
    match status {
        Status::Incomplete => true,
        _ => false,
    }
}

fn is_success(status : Status) -> bool {
    match status {
        Status::Success(_) => true,
        _ => false,
    }
}

fn is_error(status : Status) -> bool {
    match status {
        Status::Error(_) => true,
        _ => false,
    }
}

fn parse_entry(lines : [&str; 4]) -> Status {
    let mut parser = bank_ocr::Parser::new();

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
