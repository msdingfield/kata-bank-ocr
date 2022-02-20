mod parse;
mod checksum;
mod process;

use parse::*;
use checksum::*;
pub use process::*;

pub fn format_line(line : Result) -> String {
    match line {
        Result::Success {account_number, line_number: _} => account_number,
        Result::BadChecksum {account_number, line_number, mut alternates} => {
            match alternates.len() {
                0 => format!("{} ERR [line {}]", account_number, line_number),
                1 => alternates.pop().unwrap(),
                _ => format!("{} AMB [line {} could be {:?}]",account_number, line_number, alternates),
            }
        }
        Result::BadDigits {account_number, line_number, mut alternates} => {
            match alternates.len() {
                0 => format!("{} ILL [line {}]", account_number, line_number),
                1 => alternates.pop().unwrap(),
                _ => format!("{} AMB [line {} could be {:?}]",account_number, line_number, alternates),
            }
        },
        Result::InvalidCharacter {error} => format!("ERROR: {}:{}: row {}: {}", error.line_number, error.col, error.row, error.message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrecognizable_digit_no_alt() {
        let input = [
            "    _  _  _  _  _        _ ",
            "|_||_|| |   |_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ];

        let output = process(input);
        assert_eq!(output, "490?67115 ILL [line 4]");
    }

    #[test]
    fn unrecognizable_digit_single_alt() {
        let input = [
            "    _  _  _  _  _        _ ",
            "|_ |_|| || ||_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ];

        let output = process(input);
        assert_eq!(output, "490067115");
    }

    #[test]
    fn unrecognizable_digit_single_alt_with_valid_checksum() {
        let input = [
            "    _  _  _  _  _        _ ",
            "|_||_|| || ||    |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ];

        let output = process(input);
        assert_eq!(output, "490067115");
    }

    #[test]
    fn invalid_character() {
        let input = [
            "    _  _  _  _  _        _ ",
            "|_ |_|| || ||X   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ];

        let output = process(input);
        assert_eq!(output, "ERROR: 2:13: row 1: Expected space or '_' but found 'X'.");
    }

    #[test]
    fn too_few_digits() {
        let input = [
            "    _  _  _  _  _       ",
            "|_||_|| || ||_   |  |  |",
            "  | _||_||_||_|  |  |  |",
        ];

        let output = process(input);
        assert_eq!(output, "49006711? ILL [line 4]");
    }

    #[test]
    fn too_many_digits() {
        let input = [
            "    _  _  _  _  _           _ ",
            "|_||_|| || ||_   |  |  |  || |",
            "  | _||_||_||_|  |  |  |  ||_|",
        ];

        let output = process(input);
        assert_eq!(output, "ERROR: 1:28: row 0: Input line is too long.");
    }

    #[test]
    fn valid_account_number() {
        let input = [
            "    _  _  _  _  _        _ ",
            "|_||_|| || ||_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ];

        let output = process(input);
        assert_eq!(output, "490067115");
    }

    #[test]
    fn bad_checksum_no_alt() {
        let input = [
            "    _  _  _  _     _     _ ",
            "|_||_||_|| ||_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ];

        let output = process(input);
        assert_eq!(output, "498061715 ERR [line 4]");
    }

    #[test]
    fn bad_checksum_single_alt() {
        let input = [ // 723456789
            " _  _  _     _  _  _  _  _ ",
            "  | _| _||_||_ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _|",
        ];

        let output = process(input);
        assert_eq!(output, "123456789");
    }

    #[test]
    fn bad_checksum_multiple_alts() {
        let input = [
            "    _  _  _  _  _  _     _ ",
            "|_||_|| || ||_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ];

        let output = process(input);
        assert_eq!(output, "490067715 AMB [line 4 could be [\"490867715\", \"490067115\", \"490067719\"]]");
    }

    #[test]
    fn multiple_lines() {
        let input = [[
            "    _  _  _  _  _        _ ",
            "|  |_|| || ||_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ], [
            "    _  _  _  _  _        _ ",
            "|_ |_|| || ||X   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ], [
            "    _  _  _  _  _       ",
            "|_||_|| || ||_   |  |  |",
            "  | _||_||_||_|  |  |  |",
        ], [
            "    _  _  _  _  _           _ ",
            "|_||_|| || ||_   |  |  |  || |",
            "  | _||_||_||_|  |  |  |  ||_|",
        ], [
            "    _  _  _  _  _        _ ",
            "|_||_|| || ||_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ], [
            "    _  _  _  _     _     _ ",
            "|_||_||_|| ||_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ], [ // 723456789
            " _  _  _     _  _  _  _  _ ",
            "  | _| _||_||_ |_   ||_||_|",
            "  ||_  _|  | _||_|  ||_| _|",
        ], [
            "    _  _  _  _  _  _     _ ",
            "|_||_|| || ||_   |  |  ||_ ",
            "  | _||_||_||_|  |  |  | _|",
        ]];

        let r = input
            .iter()
            .flat_map(|b| [b[0], b[1], b[2], ""])
            .map(|x| x.to_string())
            .collect();

        let output = processx(r);
        assert_eq!(output, ["?90067115 ILL [line 4]",
                   "ERROR: 6:13: row 1: Expected space or '_' but found 'X'.",
                   "49006711? ILL [line 12]",
                   "ERROR: 13:28: row 0: Input line is too long.",
                   "490067115",
                   "498061715 ERR [line 24]",
                   "123456789",
                   "490067715 AMB [line 32 could be [\"490867715\", \"490067115\", \"490067719\"]]"]);
    }

    fn process(input : [&str; 3]) -> String {
        processx(
            Vec::from([input[0], input[1], input[2], ""].map(|x| x.to_string()))
        ).pop().unwrap()
    }

    fn processx(input: Vec<String>) -> Vec<String> {
        let iter = input.iter().map(|s| s.to_string());
        let output: Vec<String> = Processor::new(iter).map(|line| format_line(line)).collect();
        output
    }

}