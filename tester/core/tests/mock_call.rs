use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use novax::Address;
use novax::errors::NovaXError;
use novax_mocking::world::infos::ScenarioWorldInfos;
use num_bigint::{BigInt, BigUint};
use novax::tester::tester::{CustomEnum, CustomEnumWithFields, CustomEnumWithValues, CustomStruct, CustomStructWithStructAndVec, TesterContract};
use novax::executor::StandardMockExecutor;
use novax_mocking::ScenarioWorld;

const CALLER: &str = "bech32:erd1h4uhy73dev6qrfj7wxsguapzs8632mfwqjswjpsj6kzm2jfrnslqsuduqu";
const TESTER_CONTRACT_ADDRESS: &str = "bech32:erd1qqqqqqqqqqqqqpgq9wmk04e90fkhcuzns0pgwm33sdtxze346vpsq0ka9p";

fn get_executor() -> Arc<Mutex<StandardMockExecutor>> {
    let infos = ScenarioWorldInfos::from_file(Path::new("tests/data/adder_world_dump.json")).unwrap();
    let world = infos.into_world(|_, code_expr, world| {
        world.register_contract(code_expr, tester_contract::ContractBuilder)
    });

    let executor = StandardMockExecutor::new(
        Arc::new(Mutex::new(world)),
        Some(CALLER.to_string())
    );

    Arc::new(Mutex::new(executor))
}

fn get_executor_without_caller() -> Arc<Mutex<StandardMockExecutor>> {
    let infos = ScenarioWorldInfos::from_file(Path::new("tests/data/adder_world_dump.json")).unwrap();
    let world = infos.into_world(|_, code_expr, world| {
        world.register_contract(code_expr, tester_contract::ContractBuilder)
    });

    let executor = StandardMockExecutor::new(
        Arc::new(Mutex::new(world)),
        None
    );

    Arc::new(Mutex::new(executor))
}

// useless, should compile only
// allow to test ScenarioWorld re-export
#[allow(dead_code)]
fn get_useless_executor() -> StandardMockExecutor {
    let world = ScenarioWorld::new();

    StandardMockExecutor::new(
        Arc::new(Mutex::new(world)),
        None
    )
}

#[tokio::test]
async fn test_call_return_caller() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_caller()
        .await?;

    assert!(result.response.is_success());
    assert_eq!(result.result, Some(Address::from(CALLER)));

    Ok(())
}

#[tokio::test]
async fn test_call_return_caller_no_caller_set() -> Result<(), NovaXError> {
    let executor = get_executor_without_caller();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_caller()
        .await?;

    assert!(result.response.is_success());
    assert_eq!(result.result, Some(Address::from(TESTER_CONTRACT_ADDRESS)));

    Ok(())
}

#[tokio::test]
async fn test_call_with_biguint_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
        )
        .call(executor, 600000000)
        .get_sum()
        .await?;

    assert!(result.response.is_success());
    assert_eq!(result.result, Some(BigUint::from(5u8)));

    Ok(())
}

#[tokio::test]
async fn test_call_with_biguint_argument() -> Result<(), NovaXError> {
    let executor = get_executor();

    let contract = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    );

    contract
        .clone()
        .call(executor.clone(), 600000000)
        .add(&BigUint::from(10u8))
        .await?;

    let result = contract
        .query(executor)
        .get_sum()
        .await?;

    assert_eq!(result, BigUint::from(15u8));

    Ok(())
}

#[tokio::test]
async fn test_call_buffer_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_managed_buffer()
        .await?;

    let expected = "test";

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_contract_address_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_contract_address(&TESTER_CONTRACT_ADDRESS.into())
        .await?;

    let expected: Address = TESTER_CONTRACT_ADDRESS.into();

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_biguint_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_biguint()
        .await?;

    let expected = BigUint::from(10u8).pow(18);

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_u8_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_u_8()
        .await?;

    let expected = 3u8;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_u16_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_u_16()
        .await?;

    let expected = 500u16;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_u32_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_u_32()
        .await?;

    let expected = 200000u32;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_u64_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_u_64()
        .await?;

    let expected = 9000000000u64;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_u32_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_u_32_vec()
        .await?;

    let expected = vec![10u32, 200000u32];

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_u64_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_u_64_vec()
        .await?;

    let expected = vec![10u64, 9000000000u64];

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_buffer_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_buffer_vec()
        .await?;

    let expected = vec!["test1", "test2"];

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_biguint_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_biguint_vec()
        .await?;

    let expected = vec![
        BigUint::from(10u8).pow(18),
        BigUint::from(10u8).pow(18) * BigUint::from(2u8)
    ];

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_two_u64_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_two_u_64()
        .await?;

    let expected = (10u64, 9000000000u64);

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_two_buffers_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_two_buffers()
        .await?;

    let expected = ("test1".to_string(), "test2".to_string());

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_one_buffer_one_u64_and_one_biguint_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_one_buffer_one_u_64_and_one_biguint()
        .await?;

    let expected = (
        "test1".to_string(),
        9000000000u64,
        BigUint::from(10u8).pow(18)
    );

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_double_of_u64_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_double_of_u_64_arg(&9000000000u64)
        .await?;

    let expected = 18000000000u64;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_double_of_biguint_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_double_of_biguint_arg(&BigUint::from(10u8).pow(18))
        .await?;

    let expected = BigUint::from(10u8).pow(18) * BigUint::from(2u8);

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_appended_buffer_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_appended_buffer_arg(&"test!".to_string())
        .await?;

    let expected = "test!test";

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_sum_of_two_biguint_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = BigUint::from(10u8).pow(18);
    let second_arg = BigUint::from(10u8).pow(18) * BigUint::from(2u8);

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_sum_two_biguint_args(&first_arg, &second_arg)
        .await?;

    let expected = first_arg + second_arg;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_concat_multi_buffer_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = "test1".to_string();
    let second_arg = "test2".to_string();
    let args = vec![first_arg.clone(), second_arg.clone()];

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_concat_multi_buffer_args(&args)
        .await?;

    let expected = format!("{first_arg}{second_arg}");

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_sum_multi_u64_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = 10u64;
    let second_arg = 9000000000u64;
    let args = vec![first_arg, second_arg];

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_sum_multi_u_64_args(&args)
        .await?;

    let expected = first_arg + second_arg;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_sum_multi_biguint_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = BigUint::from(10u8).pow(18);
    let second_arg = BigUint::from(10u8).pow(18) * BigUint::from(2u8);
    let args = vec![first_arg.clone(), second_arg.clone()];

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_sum_multi_biguint_args(&args)
        .await?;

    let expected = first_arg + second_arg;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_return_optional_value_bool_arg_some_true() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_optional_value_bool_arg(&Some(true))
        .await?
        .result
        .unwrap();

    let expected = Some(true);

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_return_optional_value_bool_arg_some_false() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_optional_value_bool_arg(&Some(false))
        .await?
        .result
        .unwrap();

    let expected = Some(false);

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_return_optional_value_bool_arg_none() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_optional_value_bool_arg(&None)
        .await?
        .result
        .unwrap();

    let expected = None;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_call_custom_struct_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_struct()
        .await?;

    let expected = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    assert_eq!(result.result.unwrap(), expected);

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

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_struct_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_custom_struct_with_struct_and_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_struct_with_struct_and_vec()
        .await?;

    let expected_first_vec = vec![10u64, 9000000000u64];
    let expected_second_vec = vec!["test1".to_string(), "test2".to_string()];
    let expected_custom_struct = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    let expected = CustomStructWithStructAndVec {
        first: expected_first_vec,
        second: expected_second_vec,
        third: expected_custom_struct
    };

    assert_eq!(result.result.unwrap(), expected);

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

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_struct_with_struct_and_vec_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_custom_enum_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_enum()
        .await?;

    let expected = CustomEnum::Second;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_custom_enum_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomEnum::Third;

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_enum_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_first_custom_enum_with_values_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_first_custom_enum_with_values()
        .await?;

    let expected = CustomEnumWithValues::First(
        "test".to_string(),
        9000000000,
        BigUint::from(10u8).pow(18),
    );

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_second_custom_enum_with_values_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_second_custom_enum_with_values()
        .await?;

    let expected_first_vec = vec![10u64, 9000000000u64];
    let expected_second_vec = vec!["test1".to_string(), "test2".to_string()];
    let expected_custom_struct = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    let expected = CustomEnumWithValues::Second(
        expected_first_vec,
        expected_second_vec,
        expected_custom_struct,
    );

    assert_eq!(result.result.unwrap(), expected);

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

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_enum_with_values_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result.result.unwrap(), expected);

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

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_enum_with_values_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_first_custom_enum_with_fields_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_first_custom_enum_with_fields()
        .await?;

    let expected = CustomEnumWithFields::First {
        first_first: "test".to_string(),
        first_second: 9000000000,
        first_third: BigUint::from(10u8).pow(18),
    };

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}

#[tokio::test]
async fn test_call_second_custom_enum_with_fields_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_second_custom_enum_with_fields()
        .await?;

    let expected_first_vec = vec![10u64, 9000000000u64];
    let expected_second_vec = vec!["test1".to_string(), "test2".to_string()];
    let expected_custom_struct = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    let expected = CustomEnumWithFields::Second {
        second_first: expected_first_vec,
        second_second: expected_second_vec,
        second_third: expected_custom_struct,
    };

    assert_eq!(result.result.unwrap(), expected);

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

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_enum_with_fields_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result.result.unwrap(), expected);

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

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_custom_enum_with_fields_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result.result.unwrap(), expected);

    Ok(())
}


#[tokio::test]
async fn test_return_big_int_arg() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor.clone(), 600000000)
        .return_big_int_arg(&BigInt::from(42i8))
        .await?
        .result
        .unwrap();

    let expected = BigInt::from(42 as u8);

    assert_eq!(result, expected);

    Ok(())
}