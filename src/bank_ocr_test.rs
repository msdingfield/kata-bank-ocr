use crate::bank_ocr;
use crate::bank_ocr::Entry;

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

    assert_eq!("ERROR: 1:7: row 1: Expected space or '_' but found 'x'.", parse_to_string([
        "    _  _     _  _  _  _  _ ",
        "  | _| x||_||_ |_   ||_||_|",
        "  ||_  _|  | _||_|  ||_| _|",
        ""
    ]));

    assert_eq!("ERROR: 0:21: row 0: Expected space but found 'Q'.", parse_to_string([
        "    _  _     _  _  _ Q_  _ ",
        "  | _| x||_||_ |_   ||_||_|",
        "  ||_  _|  | _||_|  ||_| _|",
        ""
    ]));
}

fn parse_entry(lines : [&str; 4]) -> Option<Entry> {
    let mut parser = bank_ocr::Parser::new();

    for line in lines {
        let entry = parser.process_line(String::from(line));
        if entry.is_complete() {
            return Option::Some(entry);
        }
    }

    Option::None
}

fn parse_to_string(lines : [&str; 4]) -> String {
    let result = parse_entry(lines);

    if result.is_none() {
        return String::from("No Entry");
    }

    let entry = result.unwrap();
    if entry.error.is_some() {
        let e = entry.error.unwrap();
        return format!("ERROR: {}:{}: row {}: {}", e.line_number, e.col, e.row, e.message);
    }

    if entry.account_number.is_some() {
        let an = entry.account_number.unwrap();
        println!("{}", an);
        return an;
    }

    String::from("Unexpected")
}
