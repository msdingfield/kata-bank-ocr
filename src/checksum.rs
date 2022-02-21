use std::str;

// Test if the account number has a valid checksum
pub fn is_checksum_valid(account_number : &str) -> bool {
    assert!(is_numeric(account_number), "account_number must be numeric.");
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

fn is_numeric(account_number : &str) -> bool {
    for ch in account_number.chars() {
        if ! ch.is_numeric() {
            return false;
        }
    }
    return true;
}
// For a given digit, find other digits that can be formed by flipping a single segment
fn alternate_digits(ch : u8) -> &'static [u8] {
    match ch as char {
        '0' => &['8' as u8],
        '1' => &['7' as u8],
        '2' => &[],
        '3' => &['9' as u8],
        '4' => &[],
        '5' => &['6' as u8, '9' as u8],
        '6' => &['5' as u8, '8' as u8],
        '7' => &['1' as u8],
        '8' => &['0' as u8, '6' as u8, '9' as u8],
        '9' => &['3' as u8, '5' as u8, '8' as u8],
        _ => panic!("Character must be in range '0' .. '9'."),
    }
}

// Adjacent digits
// 0 <-> 8
//  _   _
// | | |_|
// |_| |_|
//
// 1 <-> 7
//
//      _
//   |   |
//   |   |
//
// 3 <-> 9
//
//  _   _
//  _| |_|
//  _|  _|
//
// 5 <-> 6, 9
//
//  _   _   _
// |_  |_  |_|
//  _| |_|  _|
//
// 6 <-> 5, 8
//
//  _   _
// |_  |_|
// |_| |_|
//
// 8 <-> 0, 6, 9
//
//  _   _
// |_| |_|
// |_|  _|
pub fn find_adjacent(account_number : &str) -> Vec<String> {
    assert_eq!(account_number.len(), 9, "account_number must be exactly 9 digits.");
    assert!(account_number.is_ascii(), "account_number must contain only characters '0' though '9'.");

    let mut matches = Vec::new();

    // Copy account number into mutable buffer
    let mut buffer = [0;9];
    buffer.copy_from_slice(account_number.as_bytes());

    // For each digit position try swapping with possible alternatives and test if checksum is valid
    for n in 0..buffer.len() {
        let ch = buffer[n];
        let alts = alternate_digits(ch);
        for alt in alts {
            buffer[n] = *alt;
            let result = str::from_utf8(&buffer);
            if let Ok(candidate) = result {
                if is_checksum_valid(&candidate) {
                    matches.push(candidate.to_string());
                }
            }
        }
        buffer[n] = ch;
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate_find_adjacent(account_number : &str, adjacents : Vec<&str>) {
        let adj = find_adjacent(account_number);
        assert_eq!(adj, adjacents);
        for adjacent in adjacents {
            assert!(is_checksum_valid(adjacent));
        }
    }

    #[test]
    fn finds_adjacent_numbers() {
        validate_find_adjacent("123456789", vec![]);
        validate_find_adjacent("723456789", vec!["123456789"]);
        validate_find_adjacent("129456789", vec!["123456789", "129456799"]);
        validate_find_adjacent("123466789", vec!["123456789", "123466709"]);
        validate_find_adjacent("123496789", vec!["123456789", "123496799"]);
        validate_find_adjacent("123455789", vec!["123456789", "123455189"]);
        validate_find_adjacent("123458789", vec!["123456789"]);
        validate_find_adjacent("123456189", vec!["123455189", "123456789", "123456169", "123456185"]);
        validate_find_adjacent("123456709", vec!["123466709", "123456789", "123456703"]);
        validate_find_adjacent("123456769", vec!["123456169", "123456789"]);
        validate_find_adjacent("123456799", vec!["129456799", "123496799", "123456789"]);
        validate_find_adjacent("123456788", vec!["123456789"]);
        validate_find_adjacent("123466789", vec!["123456789", "123466709"]);
    }

    #[test]
    fn can_validate_checksum() {
        assert!(!is_checksum_valid("00000019"), "checksome");
        assert!( is_checksum_valid("000000019"), "checksome");
        assert!(!is_checksum_valid("0000000019"), "checksome");

        assert!(is_checksum_valid("000000000"), "checksome");
        assert!(!is_checksum_valid("000000001"), "checksome");
        assert!(is_checksum_valid("500000301"), "checksome");
        assert!(is_checksum_valid("135802539"), "checksome");
    }

}