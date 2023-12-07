use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use novax::errors::NovaXError;
use novax_mocking::world::infos::{ScenarioWorldInfosEsdtTokenAmount, ScenarioWorldInfos};
use num_bigint::{BigInt, BigUint};
use novax::Address;
use novax::tester::tester::{CustomEnum, CustomEnumWithFields, CustomEnumWithValues, CustomStruct, CustomStructWithStructAndVec, TesterContract, TestTokenProperties};
use novax::executor::StandardMockExecutor;

const TESTER_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgq9wmk04e90fkhcuzns0pgwm33sdtxze346vpsq0ka9p";

fn get_executor() -> Arc<StandardMockExecutor> {
    let mut infos = ScenarioWorldInfos::from_file(Path::new("tests/data/adder_world_dump.json")).unwrap();

    let tester_contract_address_bytes = Address::from_bech32_string(TESTER_CONTRACT_ADDRESS).unwrap().to_bytes();
    let mut tester_contract_balances = infos.address_balances
        .get(&tester_contract_address_bytes)
        .cloned()
        .unwrap_or_default();

    tester_contract_balances.push(
        ScenarioWorldInfosEsdtTokenAmount {
            token_identifier: "TEST-abcdef".to_string(),
            nonce: 0,
            amount: BigUint::from(25u8),
            opt_attributes: None,
        }
    );

    tester_contract_balances.push(
        ScenarioWorldInfosEsdtTokenAmount {
            token_identifier: "NFT-abcdef".to_string(),
            nonce: 6,
            amount: BigUint::from(1u8),
            opt_attributes: Some(b"AAAAC3Rlc3QgYnVmZmVyAAAAAQo=".to_vec()),
        }
    );

    infos.address_balances.insert(
        tester_contract_address_bytes,
        tester_contract_balances
    );

    let world = infos.into_world(|_, code_expr, world| {
        world.register_contract(code_expr, tester_contract::ContractBuilder)
    });

    let executor = StandardMockExecutor::new(
        Arc::new(Mutex::new(world)),
        None
    );

    Arc::new(executor)
}

#[tokio::test]
async fn test_simple_query() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .get_sum()
        .await?;

    assert_eq!(result, BigUint::from(5u8));

    Ok(())
}

#[tokio::test]
async fn test_simple_query_with_argument() -> Result<(), NovaXError> {
    let executor = get_executor();

    TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .add(&BigUint::from(10u8))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_query_doesnt_modify_state() -> Result<(), NovaXError> {
    let executor = get_executor();

    let contract = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    );

    contract
        .clone()
        .query(executor.clone())
        .add(&BigUint::from(10u8))
        .await?;

    let result = contract
        .query(executor)
        .get_sum()
        .await?;

    assert_eq!(result, BigUint::from(5u8));

    Ok(())
}

#[tokio::test]
async fn test_query_sc_fungible_balance() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_fungible_balance()
        .await?;

    let expected = BigUint::from(25u8);

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_sc_non_fungible_balance() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_non_fungible_balance()
        .await?;

    let expected = BigUint::from(1u8);

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_sc_non_fungible_attributes() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_nft_properties()
        .await?;

    let expected = TestTokenProperties {
        buffer: "test buffer".to_string(),
        integer: BigUint::from(10u8),
    };

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_buffer_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_managed_buffer()
        .await?;

    let expected = "test";

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_biguint_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_biguint()
        .await?;

    let expected = BigUint::from(10u8).pow(18);

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_u8_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_u_8()
        .await?;

    let expected = 3u8;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_u16_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_u_16()
        .await?;

    let expected = 500u16;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_u32_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_u_32()
        .await?;

    let expected = 200000u32;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_u64_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_u_64()
        .await?;

    let expected = 9000000000u64;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_u32_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_u_32_vec()
        .await?;

    let expected = vec![10u32, 200000u32];

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_u64_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_u_64_vec()
        .await?;

    let expected = vec![10u64, 9000000000u64];

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_buffer_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_buffer_vec()
        .await?;

    let expected = vec!["test1", "test2"];

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_biguint_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_biguint_vec()
        .await?;

    let expected = vec![
        BigUint::from(10u8).pow(18),
        BigUint::from(10u8).pow(18) * BigUint::from(2u8)
    ];

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_two_u64_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_two_u_64()
        .await?;

    let expected = (10u64, 9000000000u64);

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_two_buffers_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_two_buffers()
        .await?;

    let expected = ("test1".to_string(), "test2".to_string());

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_one_buffer_one_u64_and_one_biguint_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_one_buffer_one_u_64_and_one_biguint()
        .await?;

    let expected = (
        "test1".to_string(),
        9000000000u64,
        BigUint::from(10u8).pow(18)
    );

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_double_of_u64_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_double_of_u_64_arg(&9000000000u64)
        .await?;

    let expected = 18000000000u64;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_double_of_biguint_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_double_of_biguint_arg(&BigUint::from(10u8).pow(18))
        .await?;

    let expected = BigUint::from(10u8).pow(18) * BigUint::from(2u8);

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_appended_buffer_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_appended_buffer_arg(&"test!".to_string())
        .await?;

    let expected = "test!test";

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_sum_of_two_biguint_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = BigUint::from(10u8).pow(18);
    let second_arg = BigUint::from(10u8).pow(18) * BigUint::from(2u8);

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_sum_two_biguint_args(&first_arg, &second_arg)
        .await?;

    let expected = first_arg + second_arg;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_concat_multi_buffer_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = "test1".to_string();
    let second_arg = "test2".to_string();
    let args = vec![first_arg.clone(), second_arg.clone()];

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_concat_multi_buffer_args(&args)
        .await?;

    let expected = format!("{first_arg}{second_arg}");

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_sum_multi_u64_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = 10u64;
    let second_arg = 9000000000u64;
    let args = vec![first_arg, second_arg];

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_sum_multi_u_64_args(&args)
        .await?;

    let expected = first_arg + second_arg;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_sum_multi_biguint_args_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let first_arg = BigUint::from(10u8).pow(18);
    let second_arg = BigUint::from(10u8).pow(18) * BigUint::from(2u8);
    let args = vec![first_arg.clone(), second_arg.clone()];

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_sum_multi_biguint_args(&args)
        .await?;

    let expected = first_arg + second_arg;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_return_optional_value_bool_arg_some_true() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor.clone())
        .return_optional_value_bool_arg(&Some(true))
        .await?;

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
        .query(executor.clone())
        .return_optional_value_bool_arg(&Some(false))
        .await?;

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
        .query(executor.clone())
        .return_optional_value_bool_arg(&None)
        .await?;

    let expected = None;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_custom_struct_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_custom_struct()
        .await?;

    let expected = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_custom_struct_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomStruct {
        first: "test".to_string(),
        second: 9000000000,
        third: BigUint::from(10u8).pow(18),
    };

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_custom_struct_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_custom_struct_with_struct_and_vec_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
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

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_custom_struct_with_struct_and_vec_arg_result() -> Result<(), NovaXError> {
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
        .query(executor)
        .return_custom_struct_with_struct_and_vec_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_custom_enum_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_custom_enum()
        .await?;

    let expected = CustomEnum::Second;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_custom_enum_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomEnum::Third;

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_custom_enum_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_first_custom_enum_with_values_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_first_custom_enum_with_values()
        .await?;

    let expected = CustomEnumWithValues::First(
        "test".to_string(),
        9000000000,
        BigUint::from(10u8).pow(18),
    );

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_second_custom_enum_with_values_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
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

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_first_custom_enum_with_values_arg_result() -> Result<(), NovaXError> {
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
        .query(executor)
        .return_custom_enum_with_values_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_second_custom_enum_with_values_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomEnumWithValues::First(
        "test".to_string(),
        9000000000,
        BigUint::from(10u8).pow(18),
    );

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_custom_enum_with_values_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_first_custom_enum_with_fields_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_first_custom_enum_with_fields()
        .await?;

    let expected = CustomEnumWithFields::First {
        first_first: "test".to_string(),
        first_second: 9000000000,
        first_third: BigUint::from(10u8).pow(18),
    };

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_second_custom_enum_with_fields_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
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

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_first_custom_enum_with_fields_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let input = CustomEnumWithFields::First {
        first_first: "test".to_string(),
        first_second: 9000000000,
        first_third: BigUint::from(10u8).pow(18),
    };

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_custom_enum_with_fields_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_second_custom_enum_with_fields_arg_result() -> Result<(), NovaXError> {
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
        .query(executor)
        .return_custom_enum_with_fields_arg(&input)
        .await?;

    let expected = input;

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_query_bigint_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_big_int_arg(&BigInt::from(45i8))
        .await?;

    let expected = BigInt::from(45i8);

    assert_eq!(result, expected);

    Ok(())
}