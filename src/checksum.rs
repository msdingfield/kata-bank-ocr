
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

}