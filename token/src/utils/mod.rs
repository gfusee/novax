use crate::error::token::TokenError;

pub fn parse_identifier(token_identifier: &str) -> Result<(String, u64), TokenError> {
    let parts = token_identifier.split('-').collect::<Vec<&str>>();
    let parts_len = parts.len();

    let (identifier, nonce) = if parts_len == 2 || parts_len == 3 {
        let identifier = format!("{}-{}", parts[0], parts[1]);

        let nonce = if parts_len == 2 {
            0
        } else {
            let Ok(nonce) = u64::from_str_radix(parts[2], 16) else {
                return Err(TokenError::InvalidTokenIdentifier { identifier: token_identifier.to_string() })
            };

            nonce
        };

        (identifier, nonce)
    } else {
        return Err(TokenError::InvalidTokenIdentifier { identifier: token_identifier.to_string() })
    };

    Ok((identifier, nonce))
}

#[cfg(test)]
mod tests {
    use crate::error::token::TokenError;
    use crate::utils::parse_identifier;

    #[test]
    fn test_valid_fungible() {
        let result = parse_identifier("WEGLD-abcdef").unwrap();

        let expected = ("WEGLD-abcdef".to_string(), 0u64);

        assert_eq!(result, expected)
    }

    #[test]
    fn test_valid_non_fungible() {
        let result = parse_identifier("LKMEX-abcdef-09").unwrap();

        let expected = ("LKMEX-abcdef".to_string(), 9u64);

        assert_eq!(result, expected)
    }

    #[test]
    fn test_valid_non_fungible_hex_uppercase_nonce() {
        let result = parse_identifier("LKMEX-abcdef-9F2").unwrap();

        let expected = ("LKMEX-abcdef".to_string(), 2546u64);

        assert_eq!(result, expected)
    }

    #[test]
    fn test_valid_non_fungible_hex_lowercase_nonce() {
        let result = parse_identifier("LKMEX-abcdef-9f2").unwrap();

        let expected = ("LKMEX-abcdef".to_string(), 2546u64);

        assert_eq!(result, expected)
    }

    #[test]
    fn test_non_fungible_invalid_hex_nonce() {
        let result = parse_identifier("LKMEX-abcdef-test").unwrap_err();

        let expected: TokenError = TokenError::InvalidTokenIdentifier { identifier: "LKMEX-abcdef-test".to_string() };

        assert_eq!(result, expected)
    }

    #[test]
    fn test_too_short_identifier() {
        let result = parse_identifier("WEGLD").unwrap_err();

        let expected: TokenError = TokenError::InvalidTokenIdentifier { identifier: "WEGLD".to_string() };

        assert_eq!(result, expected)
    }

    #[test]
    fn test_too_long_identifier() {
        let result = parse_identifier("WEGLD-abcdef-05-a").unwrap_err();

        let expected: TokenError = TokenError::InvalidTokenIdentifier { identifier: "WEGLD-abcdef-05-a".to_string() };

        assert_eq!(result, expected)
    }
}