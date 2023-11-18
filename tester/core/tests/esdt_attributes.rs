use base64::Engine;
use num_bigint::BigUint;
use novax::tester::tester::TestTokenProperties;

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