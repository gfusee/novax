use multiversx_sc::types::{ManagedBuffer, MultiValueEncoded};
use multiversx_sc_scenario::DebugApi;
use novax_data::NativeConvertible;


#[test]
fn test_managed_vec_to_native() {
    DebugApi::dummy();
    let mut multi_value_encoded: MultiValueEncoded<DebugApi, ManagedBuffer<DebugApi>> = MultiValueEncoded::new();

    multi_value_encoded.push(ManagedBuffer::from("first"));
    multi_value_encoded.push(ManagedBuffer::from("second"));
    multi_value_encoded.push(ManagedBuffer::from("third"));

    let native = multi_value_encoded.to_native();
    let expected = vec![
        String::from("first"),
        String::from("second"),
        String::from("third")
    ];

    assert_eq!(
        native,
        expected
    )
}