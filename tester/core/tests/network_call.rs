mod utils;

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex;
use novax::{Address, Wallet};
use novax::errors::NovaXError;
use num_bigint::BigUint;
use novax::tester::tester::{CustomEnum, CustomEnumWithFields, CustomEnumWithValues, CustomStruct, CustomStructWithStructAndVec, TesterContract};
use novax::executor::{BaseTransactionNetworkExecutor, BlockchainInteractor, NetworkExecutor, SendableTransactionConvertible};
use novax_mocking::{ScCallStep, ScDeployStep, TxResponse};
use crate::utils::decode_scr_data::decode_scr_data_or_panic;

const CALLER_PRIVATE_KEY: &str = "69417ce717e43d0d3a598f68b5e562d7d2a532a5a3ac1e8b3342515e0b2d950f"; // to anyone reading : this has been generated only for the tests below
const CALLER: &str = "erd12wf7tlsk2z895vwmndheaknkp3uaqa7xuq847numkwlmcvy60wxql2ndlk";
const TESTER_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgq9wmk04e90fkhcuzns0pgwm33sdtxze346vpsq0ka9p";

struct MockInteractor;

#[async_trait]
impl BlockchainInteractor for MockInteractor {
    async fn new(_gateway_url: &str) -> Self {
        MockInteractor
    }

    fn register_wallet(&mut self, _wallet: Wallet) -> Address {
        Address::from_bech32_string(CALLER).unwrap()
    }

    async fn sc_call<S>(&mut self, mut sc_call_step: S) where S: AsMut<ScCallStep> + Send {
        let mut return_data: Option<String> = None;
        let call_step = sc_call_step.as_mut();
        let sendable_transaction = call_step.tx.to_sendable_transaction();
        let data = sendable_transaction.data;
        if data == "returnCaller" {
            return_data = Some("@6f6b@5393e5fe16508e5a31db9b6f9eda760c79d077c6e00f5f4f9bb3bfbc309a7b8c".to_string());
        } else if data == "getSum" {
            return_data = Some("@6f6b@05".to_string());
        } else if data == "add@0a" {
            return_data = Some("@6f6b@".to_string())
        } else if data == "returnManagedBuffer" {
            return_data = Some("@6f6b@74657374".to_string())
        } else if data == "returnBiguint" {
            return_data = Some("@6f6b@0de0b6b3a7640000".to_string())
        } else if data == "returnBiguintVec" {
            return_data = Some("@6f6b@000000080de0b6b3a7640000000000081bc16d674ec80000".to_string())
        } else if data == "returnBufferVec" {
            return_data = Some("@6f6b@000000057465737431000000057465737432".to_string())
        } else if data == "returnCustomEnum" {
            return_data = Some("@6f6b@01".to_string())
        } else if data == "returnCustomStruct" {
            return_data = Some("@6f6b@00000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnCustomStructWithStructAndVec" {
            return_data = Some("@6f6b@00000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnFirstCustomEnumWithFields" || data == "returnFirstCustomEnumWithValues" {
            return_data = Some("@6f6b@0000000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnOneBufferOneU64AndOneBiguint" {
            return_data = Some("@6f6b@7465737431@0218711a00@0de0b6b3a7640000".to_string())
        } else if data == "returnSecondCustomEnumWithFields" || data == "returnSecondCustomEnumWithValues" {
            return_data = Some("@6f6b@0100000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnTwoBuffers" {
            return_data = Some("@6f6b@7465737431@7465737432".to_string())
        } else if data == "returnTwoU64" {
            return_data = Some("@6f6b@0a@0218711a00".to_string())
        } else if data == "returnU16" {
            return_data = Some("@6f6b@01f4".to_string())
        } else if data == "returnU32" {
            return_data = Some("@6f6b@030d40".to_string())
        } else if data == "returnU32Vec" {
            return_data = Some("@6f6b@0000000a00030d40".to_string())
        } else if data == "returnU64" {
            return_data = Some("@6f6b@0218711a00".to_string())
        } else if data == "returnU64Vec" {
            return_data = Some("@6f6b@000000000000000a0000000218711a00".to_string())
        } else if data == "returnU8" {
            return_data = Some("@6f6b@03".to_string())
        } else if data == "returnAppendedBufferArg@7465737421" {
            return_data = Some("@6f6b@746573742174657374".to_string())
        } else if data == "returnConcatMultiBufferArgs@7465737431@7465737432" {
            return_data = Some("@6f6b@74657374317465737432".to_string())
        } else if data == "returnCustomEnumArg@02" {
            return_data = Some("@6f6b@02".to_string())
        } else if data == "returnCustomStructArg@00000004746573740000000218711a00000000080de0b6b3a7640000" {
            return_data = Some("@6f6b@00000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnCustomStructWithStructAndVecArg@00000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000" {
            return_data = Some("@6f6b@00000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnDoubleOfBiguintArg@0de0b6b3a7640000" {
            return_data = Some("@6f6b@1bc16d674ec80000".to_string())
        } else if data == "returnDoubleOfU64Arg@0218711a00" {
            return_data = Some("@6f6b@0430e23400".to_string())
        } else if data == "returnCustomEnumWithFieldsArg@0000000004746573740000000218711a00000000080de0b6b3a7640000" {
            return_data = Some("@6f6b@0000000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnCustomEnumWithValuesArg@0100000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000"
            || data == "returnCustomEnumWithFieldsArg@0100000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000" {
            return_data = Some("@6f6b@0100000002000000000000000a0000000218711a000000000200000005746573743100000005746573743200000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnCustomEnumWithValuesArg@0000000004746573740000000218711a00000000080de0b6b3a7640000" {
            return_data = Some("@6f6b@0000000004746573740000000218711a00000000080de0b6b3a7640000".to_string())
        } else if data == "returnSumMultiBiguintArgs@0de0b6b3a7640000@1bc16d674ec80000" {
            return_data = Some("@6f6b@29a2241af62c0000".to_string())
        } else if data == "returnSumMultiU64Args@0a@0218711a00" {
            return_data = Some("@6f6b@0218711a0a".to_string())
        } else if data == "returnSumTwoBiguintArgs@0de0b6b3a7640000@1bc16d674ec80000" {
            return_data = Some("@6f6b@29a2241af62c0000".to_string())
        }

        let Some(return_data) = return_data else {
            panic!("Unknown data for : \"{data}\"");
        };

        let response = TxResponse::from_raw_results(decode_scr_data_or_panic(&return_data));

        call_step.response = Some(response);
    }

    async fn sc_deploy<S>(&mut self, _sc_deploy_step: S) where S: AsMut<ScDeployStep> + Send {
        todo!()
    }
}

fn get_executor() -> Arc<Mutex<BaseTransactionNetworkExecutor<MockInteractor>>> {
    let wallet = Wallet::from_private_key(CALLER_PRIVATE_KEY).unwrap();

    let executor = BaseTransactionNetworkExecutor::new(
        "",
        &wallet
    );

    Arc::new(Mutex::new(executor))
}

// The below test is a success if it compiles
#[tokio::test]
async fn test_clone_network_executor() -> Result<(), NovaXError> {
    let wallet = Wallet::from_private_key(CALLER_PRIVATE_KEY).unwrap();
    let executor = NetworkExecutor::new("", &wallet);
    #[allow(clippy::redundant_clone)]
    let _executor2 = executor.clone();

    Ok(())
}

// The below test is a success if it compiles
#[tokio::test]
async fn test_debug_network_executor() -> Result<(), NovaXError> {
    let wallet = Wallet::from_private_key(CALLER_PRIVATE_KEY).unwrap();
    let executor = NetworkExecutor::new("", &wallet);

    println!("{executor:?}");

    Ok(())
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
    assert_eq!(result.result, Some(Address::from_bech32_string(CALLER).unwrap()));

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
        .call(executor, 600000000)
        .add(&BigUint::from(10u8))
        .await?;

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