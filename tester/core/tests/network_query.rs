use std::sync::Arc;
use async_trait::async_trait;
use novax::errors::NovaXError;
use num_bigint::{BigInt, BigUint};
use novax::{VMOutputApi, VmValueRequest, VmValuesResponseData};
use novax::tester::tester::{CustomEnum, CustomEnumWithFields, CustomEnumWithValues, CustomStruct, CustomStructWithStructAndVec, TesterContract};
use novax::executor::{BlockchainProxy, ExecutorError, QueryNetworkExecutor};

const TESTER_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgq9wmk04e90fkhcuzns0pgwm33sdtxze346vpsq0ka9p";

#[derive(Clone)]
struct MockProxy;

#[async_trait]
impl BlockchainProxy for MockProxy {
    fn new(_gateway_url: &str) -> Self {
        MockProxy
    }

    async fn execute_vmquery(&self, vm_request: &VmValueRequest) -> Result<VmValuesResponseData, ExecutorError> {
        let mut return_data: Option<Vec<String>> = None;
        if vm_request.func_name == "getSum" {
            return_data = Some(vec!["BQ==".to_string()]);
        } else if vm_request.func_name == "add" {
            if vm_request.args == vec!["0a"] {
                return_data = Some(vec![]);
            }
        } else if vm_request.func_name == "returnManagedBuffer" {
            return_data = Some(vec!["dGVzdA==".to_string()])
        } else if vm_request.func_name == "returnBiguint" {
            return_data = Some(vec!["DeC2s6dkAAA=".to_string()])
        } else if vm_request.func_name == "returnU8" {
            return_data = Some(vec!["Aw==".to_string()])
        } else if vm_request.func_name == "returnU16" {
            return_data = Some(vec!["AfQ=".to_string()])
        } else if vm_request.func_name == "returnU32" {
            return_data = Some(vec!["Aw1A".to_string()])
        } else if vm_request.func_name == "returnU64" {
            return_data = Some(vec!["AhhxGgA=".to_string()])
        } else if vm_request.func_name == "returnU32Vec" {
            return_data = Some(vec!["AAAACgADDUA=".to_string()])
        } else if vm_request.func_name == "returnU64Vec" {
            return_data = Some(vec!["AAAAAAAAAAoAAAACGHEaAA==".to_string()])
        } else if vm_request.func_name == "returnAppendedBufferArg" {
            if vm_request.args == vec!["7465737421".to_string()] {
                return_data = Some(vec!["dGVzdCF0ZXN0".to_string()])
            }
        } else if vm_request.func_name == "returnBiguintVec" {
            return_data = Some(vec!["AAAACA3gtrOnZAAAAAAACBvBbWdOyAAA".to_string()])
        } else if vm_request.func_name == "returnBufferVec" {
            return_data = Some(vec!["AAAABXRlc3QxAAAABXRlc3Qy".to_string()])
        } else if vm_request.func_name == "returnCustomEnum" {
            return_data = Some(vec!["AQ==".to_string()])
        } else if vm_request.func_name == "returnCustomStruct" {
            return_data = Some(vec!["AAAABHRlc3QAAAACGHEaAAAAAAgN4Lazp2QAAA==".to_string()])
        } else if vm_request.func_name == "AAAABHRlc3QAAAACGHEaAAAAAAgN4Lazp2QAAA==" {
            return_data = Some(vec!["AAAAAgAAAAAAAAAKAAAAAhhxGgAAAAACAAAABXRlc3QxAAAABXRlc3QyAAAABHRlc3QAAAACGHEaAAAAAAgN4Lazp2QAAA==".to_string()])
        } else if vm_request.func_name == "returnFirstCustomEnumWithFields" || vm_request.func_name == "returnFirstCustomEnumWithValues" {
            return_data = Some(vec!["AAAAAAR0ZXN0AAAAAhhxGgAAAAAIDeC2s6dkAAA=".to_string()])
        } else if vm_request.func_name == "returnOneBufferOneU64AndOneBiguint" {
            return_data = Some(vec!["dGVzdDE=".to_string(), "AhhxGgA=".to_string(), "DeC2s6dkAAA=".to_string()])
        } else if vm_request.func_name == "returnCustomStructWithStructAndVec" {
            return_data = Some(vec!["AAAAAgAAAAAAAAAKAAAAAhhxGgAAAAACAAAABXRlc3QxAAAABXRlc3QyAAAABHRlc3QAAAACGHEaAAAAAAgN4Lazp2QAAA==".to_string()])
        } else if vm_request.func_name == "returnSecondCustomEnumWithFields" || vm_request.func_name == "returnSecondCustomEnumWithValues" {
            return_data = Some(vec!["AQAAAAIAAAAAAAAACgAAAAIYcRoAAAAAAgAAAAV0ZXN0MQAAAAV0ZXN0MgAAAAR0ZXN0AAAAAhhxGgAAAAAIDeC2s6dkAAA=".to_string()])
        } else if vm_request.func_name == "returnTwoBuffers" {
            return_data = Some(vec!["dGVzdDE=".to_string(), "dGVzdDI=".to_string()])
        } else if vm_request.func_name == "returnTwoU64" {
            return_data = Some(vec!["Cg==".to_string(), "AhhxGgA=".to_string()])
        } else if vm_request.func_name == "returnConcatMultiBufferArgs" {
            if vm_request.args == vec!["7465737431".to_string(), "7465737432".to_string()] {
                return_data = Some(vec!["dGVzdDF0ZXN0Mg==".to_string()])
            }
        } else if vm_request.func_name == "returnCustomEnumArg" {
            if vm_request.args == vec!["02".to_string()] {
                return_data = Some(vec!["Ag==".to_string()])
            }
        } else if vm_request.func_name == "returnCustomStructArg" {
            if vm_request.args == vec!["00000004746573740000000218711a00000000080de0b6b3a7640000".to_string()] {
                return_data = Some(vec!["AAAABHRlc3QAAAACGHEaAAAAAAgN4Lazp2QAAA==".to_string()])
            }
        } else if vm_request.func_name == "returnCustomStructWithStructAndVecArg" {
            if vm_request.args == vec!["00000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string()] {
                return_data = Some(vec!["AAAAAgAAAAAAAAAKAAAAAhhxGgAAAAACAAAABXRlc3QxAAAABXRlc3QyAAAABHRlc3QAAAACGHEaAAAAAAgN4Lazp2QAAA==".to_string()])
            }
        } else if vm_request.func_name == "returnDoubleOfBiguintArg" {
            if vm_request.args == vec!["0de0b6b3a7640000".to_string()] {
                return_data = Some(vec!["G8FtZ07IAAA=".to_string()])
            }
        } else if vm_request.func_name == "returnDoubleOfU64Arg" {
            if vm_request.args == vec!["0218711a00".to_string()] {
                return_data = Some(vec!["BDDiNAA=".to_string()])
            }
        } else if vm_request.func_name == "returnCustomEnumWithValuesArg" || vm_request.func_name == "returnCustomEnumWithFieldsArg" {
            if vm_request.args == vec!["0100000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string()] {
                return_data = Some(vec!["AQAAAAIAAAAAAAAACgAAAAIYcRoAAAAAAgAAAAV0ZXN0MQAAAAV0ZXN0MgAAAAR0ZXN0AAAAAhhxGgAAAAAIDeC2s6dkAAA=".to_string()])
            } else if vm_request.args == vec!["0000000004746573740000000218711a00000000080de0b6b3a7640000".to_string()] {
                return_data = Some(vec!["AAAAAAR0ZXN0AAAAAhhxGgAAAAAIDeC2s6dkAAA=".to_string()])
            }
        } else if vm_request.func_name == "returnSumMultiBiguintArgs" {
            if vm_request.args == vec!["0de0b6b3a7640000".to_string(), "1bc16d674ec80000".to_string()] {
                return_data = Some(vec!["KaIkGvYsAAA=".to_string()])
            }
        } else if vm_request.func_name == "returnSumMultiU64Args" {
            if vm_request.args == vec!["0a".to_string(), "0218711a00".to_string()] {
                return_data = Some(vec!["AhhxGgo=".to_string()])
            }
        } else if vm_request.func_name == "returnSumTwoBiguintArgs" && vm_request.args == vec!["0de0b6b3a7640000".to_string(), "1bc16d674ec80000".to_string()] {
            return_data = Some(vec!["KaIkGvYsAAA=".to_string()])
        } else if vm_request.func_name == "returnOptionalValueBoolArg" && vm_request.args == vec!["01".to_string()] {
            return_data = Some(vec!["AQ==".to_string()])
        } else if vm_request.func_name == "returnOptionalValueBoolArg" && vm_request.args == vec!["".to_string()] {
            return_data = Some(vec!["".to_string()])
        } else if vm_request.func_name == "returnOptionalValueBoolArg" && vm_request.args.is_empty() {
            return_data = Some(vec![])
        } else if vm_request.func_name == "returnBigIntArg" {
            if vm_request.args == vec!["2e".to_string()] {
                return_data = Some(vec!["Lg==".to_string()])
            }
        }

        let Some(return_data) = return_data else {
            let args_string = vm_request.args.join(", ");
            panic!("Unknown data for : \n- function name \"{}\"\n- args: [{}]", vm_request.func_name, args_string);
        };

        Ok(get_success_vm_response_data(return_data))
    }
}

fn get_success_vm_response_data(return_data: Vec<String>) -> VmValuesResponseData {
    let output = VMOutputApi {
        return_data,
        return_code: "0".to_string(),
        return_message: "".to_string(),
        gas_remaining: 0,
        gas_refund: 0,
        output_accounts: Default::default(),
        deleted_accounts: None,
        touched_accounts: None,
        logs: None,
    };

    VmValuesResponseData {
        data: output,
    }
}

fn get_executor() -> Arc<QueryNetworkExecutor<MockProxy>> {
    let executor = QueryNetworkExecutor::new("");
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
async fn test_query_big_int_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .query(executor)
        .return_big_int_arg(&BigInt::from(46i8))
        .await?;

    let expected = BigInt::from(46i8);

    assert_eq!(result, expected);

    Ok(())
}