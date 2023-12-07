use std::sync::Arc;
use num_bigint::{BigInt, BigUint};
use tokio::sync::Mutex;
use novax::Address;
use novax::errors::NovaXError;
use novax::tester::tester::{CustomEnum, CustomEnumWithFields, CustomEnumWithValues, CustomStruct, CustomStructWithStructAndVec, TesterContract};
use novax::executor::{DummyExecutor, DummyTransactionExecutor, SendableTransaction};

const CALLER: &str = "erd1h4uhy73dev6qrfj7wxsguapzs8632mfwqjswjpsj6kzm2jfrnslqsuduqu";
const TESTER_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgq9wmk04e90fkhcuzns0pgwm33sdtxze346vpsq0ka9p";

fn get_executor() -> Arc<Mutex<DummyTransactionExecutor>> {
    let executor = DummyExecutor::new(
        &Some(Address::from_bech32_string(CALLER).unwrap())
    );

    Arc::new(Mutex::new(executor))
}

#[tokio::test]
async fn test_call_with_biguint_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
        )
        .call(executor.clone(), 600000000)
        .get_sum()
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "getSum".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_with_another_gas_limit() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 10)
        .get_sum()
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 10u64,
        data: "getSum".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_with_biguint_argument() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .add(&BigUint::from(10u8))
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "add@0a".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_double_of_u64_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_double_of_u_64_arg(&9000000000u64)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnDoubleOfU64Arg@0218711a00".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_double_of_biguint_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_double_of_biguint_arg(&BigUint::from(10u8).pow(18))
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnDoubleOfBiguintArg@0de0b6b3a7640000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_sum_of_two_biguint_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = BigUint::from(10u8).pow(18);
    let second_arg = BigUint::from(10u8).pow(18) * BigUint::from(2u8);

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_sum_two_biguint_args(&first_arg, &second_arg)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnSumTwoBiguintArgs@0de0b6b3a7640000@1bc16d674ec80000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_concat_multi_buffer_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = "test1".to_string();
    let second_arg = "test2".to_string();
    let args = vec![first_arg.clone(), second_arg.clone()];

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_concat_multi_buffer_args(&args)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnConcatMultiBufferArgs@7465737431@7465737432".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_sum_multi_u64_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = 10u64;
    let second_arg = 9000000000u64;
    let args = vec![first_arg, second_arg];

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_sum_multi_u_64_args(&args)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnSumMultiU64Args@0a@0218711a00".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_return_optional_value_bool_arg_some_true() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_optional_value_bool_arg(&Some(true))
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnOptionalValueBoolArg@01".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_return_optional_value_bool_arg_some_false() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_optional_value_bool_arg(&Some(false))
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnOptionalValueBoolArg@".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_return_optional_value_bool_arg_none() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_optional_value_bool_arg(&None)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnOptionalValueBoolArg".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_sum_multi_biguint_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = BigUint::from(10u8).pow(18);
    let second_arg = BigUint::from(10u8).pow(18) * BigUint::from(2u8);
    let args = vec![first_arg.clone(), second_arg.clone()];

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_sum_multi_biguint_args(&args)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnSumMultiBiguintArgs@0de0b6b3a7640000@1bc16d674ec80000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_custom_struct_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_custom_struct_arg(&input)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnCustomStructArg@00000004746573740000000218711a00000000080de0b6b3a7640000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_custom_struct_with_struct_and_vec_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_vec = vec![10u64, 9000000000u64];
    let second_vec = vec!["test1".to_string(), "test2".to_string()];
    let custom_struct = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    let input = CustomStructWithStructAndVec {
        first: first_vec,
        second: second_vec,
        third: custom_struct
    };

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_custom_struct_with_struct_and_vec_arg(&input)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnCustomStructWithStructAndVecArg@00000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_custom_enum_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomEnum::Third;

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_custom_enum_arg(&input)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnCustomEnumArg@02".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_first_custom_enum_with_values_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_vec = vec![10u64, 9000000000u64];
    let second_vec = vec!["test1".to_string(), "test2".to_string()];
    let custom_struct = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    let input = CustomEnumWithValues::Second(
        first_vec,
        second_vec,
        custom_struct,
    );

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_custom_enum_with_values_arg(&input)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnCustomEnumWithValuesArg@0100000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_second_custom_enum_with_values_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomEnumWithValues::First(
        "test".to_string(),
        9000000000,
        BigUint::from(10u8).pow(18),
    );

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_custom_enum_with_values_arg(&input)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnCustomEnumWithValuesArg@0000000004746573740000000218711a00000000080de0b6b3a7640000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_first_custom_enum_with_fields_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomEnumWithFields::First {
        first_first: "test".to_string(),
        first_second: 9000000000,
        first_third: BigUint::from(10u8).pow(18),
    };

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_custom_enum_with_fields_arg(&input)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnCustomEnumWithFieldsArg@0000000004746573740000000218711a00000000080de0b6b3a7640000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_second_custom_enum_with_fields_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_vec = vec![10u64, 9000000000u64];
    let second_vec = vec!["test1".to_string(), "test2".to_string()];
    let custom_struct = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    let input = CustomEnumWithFields::Second {
        second_first: first_vec,
        second_second: second_vec,
        second_third: custom_struct,
    };

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_custom_enum_with_fields_arg(&input)
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnCustomEnumWithFieldsArg@0100000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_with_bigint_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_big_int_arg(&BigInt::from(44i8))
        .await?;

    let tx = executor.lock().await.get_transaction_details();

    let expected = SendableTransaction {
        receiver: TESTER_CONTRACT_ADDRESS.to_string(),
        egld_value: 0u8.into(),
        gas_limit: 600000000u64,
        data: "returnBigIntArg@2c".to_string(),
    };

    assert_eq!(tx, expected);

    Ok(())
}