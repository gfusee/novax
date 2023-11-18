use base64::Engine;
use num_bigint::BigUint;
use novax::errors::{CodingError, NovaXError};
use novax::tester::tester::TestTokenProperties;
use novax::tester::tester::TestEnumProperties;
use novax::tester::tester::TestEnumPropertiesWithFields;

#[test]
fn test_decode_struct_from_esdt_attributes() {
    let bytes_attributes = base64::engine::general_purpose::STANDARD.decode("AAAAC3Rlc3QgYnVmZmVyAAAAAQo=").unwrap();
    let result = TestTokenProperties::from_esdt_attributes(
        &bytes_attributes
    )
        .unwrap();

    let expected = TestTokenProperties {
        buffer: "test buffer".to_string(),
        integer: BigUint::from(10u8),
    };

    assert_eq!(result, expected);
}

#[test]
fn test_decode_struct_from_esdt_attributes_invalid() {
    let bytes_attributes = b"yooooo";
    let result = TestTokenProperties::from_esdt_attributes(
        bytes_attributes
    )
        .unwrap_err();

    let expected = NovaXError::Coding(CodingError::CannotDecodeEsdtAttributes);

    assert_eq!(result, expected);
}

#[test]
fn test_decode_enum_no_value_from_empty_esdt_attributes() {
    let bytes_attributes = base64::engine::general_purpose::STANDARD.decode("").unwrap();
    let result = TestEnumProperties::from_esdt_attributes(
        &bytes_attributes
    )
        .unwrap();

    let expected = TestEnumProperties::First;

    assert_eq!(result, expected);
}

#[test]
fn test_decode_enum_no_value_from_esdt_attributes() {
    let bytes_attributes = base64::engine::general_purpose::STANDARD.decode("AA==").unwrap();
    let result = TestEnumProperties::from_esdt_attributes(
        &bytes_attributes
    )
        .unwrap();

    let expected = TestEnumProperties::First;

    assert_eq!(result, expected);
}

#[test]
fn test_decode_enum_with_values_from_esdt_attributes() {
    let bytes_attributes = base64::engine::general_purpose::STANDARD.decode("AQAAAAt0ZXN0IGJ1ZmZlcgAAAAEK").unwrap();
    let result = TestEnumProperties::from_esdt_attributes(
        &bytes_attributes
    )
        .unwrap();

    let expected = TestEnumProperties::Second(
        "test buffer".to_string(),
        BigUint::from(10u8)
    );

    assert_eq!(result, expected);
}

#[test]
fn test_decode_enum_with_fields_from_esdt_attributes() {
    let bytes_attributes = base64::engine::general_purpose::STANDARD.decode("AAAAAAt0ZXN0IGJ1ZmZlcgAAAAEK").unwrap();
    let result = TestEnumPropertiesWithFields::from_esdt_attributes(
        &bytes_attributes
    )
        .unwrap();

    let expected = TestEnumPropertiesWithFields::First {
        buffer_value: "test buffer".to_string(),
        integer: BigUint::from(10u8),
    };

    assert_eq!(result, expected);
}

#[test]
fn test_decode_enum_from_esdt_attributes_invalid() {
    let bytes_attributes = b"yooooo";
    let result = TestEnumProperties::from_esdt_attributes(
        bytes_attributes
    )
        .unwrap_err();

    let expected = NovaXError::Coding(CodingError::CannotDecodeEsdtAttributes);

    assert_eq!(result, expected);
}