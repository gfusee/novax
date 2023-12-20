mod utils;

use std::sync::Arc;
use async_trait::async_trait;
use hyper::StatusCode;
use tokio::sync::Mutex;
use novax::Address;
use novax::errors::NovaXError;
use num_bigint::{BigInt, BigUint};
use reqwest::{Error, Response};
use serde::Serialize;
use novax::tester::tester::{CustomEnum, CustomEnumWithFields, CustomEnumWithValues, CustomStruct, CustomStructWithStructAndVec, TesterContract};
use novax::executor::{BaseSimulationNetworkExecutor, BlockchainInteractor, SendableTransactionConvertible, SimulationNetworkExecutor};
use novax_request::gateway::client::GatewayClient;

const CALLER: &str = "erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g";
const TESTER_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0";

fn get_response_from_data(status: StatusCode, data: String) -> Response {
    let hyper_response = hyper::Response::builder()
        .status(status)
        .body(data)
        .unwrap();

    Response::from(hyper_response)
}

fn get_caller_infos() -> Response {
    let status = StatusCode::OK;
    let data = r#"{"data":{"account":{"address":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","nonce":5,"balance":"49893375980000000000","username":"","code":"","codeHash":null,"rootHash":null,"codeMetadata":null,"developerReward":"0","ownerAddress":""},"blockInfo":{"nonce":1514622,"hash":"119621492bad699ac2a60ad276720d1735c1d0eebfe70a82498d8a613a22063a","rootHash":"6ba976a765877a1d9183ca270fc0897ff6b23f30411125243394ed39b309a0b1"}},"error":"","code":"successful"}"#.to_string();

    get_response_from_data(status, data)
}

fn get_network_config() -> Response {
    let status = StatusCode::OK;
    let data = r#"{"data":{"config":{"erd_adaptivity":"false","erd_chain_id":"D","erd_denomination":18,"erd_extra_gas_limit_guarded_tx":50000,"erd_gas_per_data_byte":1500,"erd_gas_price_modifier":"0.01","erd_hysteresis":"0.200000","erd_latest_tag_software_version":"D1.6.6.1","erd_max_gas_per_transaction":600000000,"erd_meta_consensus_group_size":58,"erd_min_gas_limit":50000,"erd_min_gas_price":1000000000,"erd_min_transaction_version":1,"erd_num_metachain_nodes":58,"erd_num_nodes_in_shard":58,"erd_num_shards_without_meta":3,"erd_rewards_top_up_gradient_point":"2000000000000000000000000","erd_round_duration":6000,"erd_rounds_per_epoch":2400,"erd_shard_consensus_group_size":21,"erd_start_time":1694000000,"erd_top_up_factor":"0.500000"}},"error":"","code":"successful"}"#.to_string();

    get_response_from_data(status, data)
}

fn get_return_caller_simulation_data() -> Response {
    let status = StatusCode::OK;
    let data = r#"{"data":{"txGasUnits":2384920,"returnMessage":"","smartContractResults":{"4b34385c5a43aa4e2f8b66f63f0e1786aef3e2acff288bd4c2669e71f9078deb":{"nonce":6,"value":26150800000000,"receiver":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","sender":"erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0","data":"@6f6b@e5f5ec2bf6b925565fd1ed99e958858250ce40fd73b12d5792e68bbda679a73c","prevTxHash":"0e888497084b1d6581698537b0a1f1c6f50841fbb5188b5e9b8ebc338662a49f","originalTxHash":"0e888497084b1d6581698537b0a1f1c6f50841fbb5188b5e9b8ebc338662a49f","gasLimit":0,"gasPrice":1000000000,"callType":0,"operation":"transfer","isRefund":true}}},"error":"","code":"successful"}"#.to_string();

    get_response_from_data(status, data)
}

struct MockClient {
    url: String
}

impl MockClient {
    pub fn new() -> Self {
        Self {
            url: "".to_string(),
        }
    }
}

#[async_trait]
impl GatewayClient for MockClient {
    type Owned = Self;

    fn get_gateway_url(&self) -> &str {
        &self.url
    }

    fn with_appended_url(&self, url: &str) -> Self::Owned {
        Self {
            url: format!("{}{}", self.url, url),
        }
    }

    async fn get(&self) -> Result<Response, Error> {
        let url = self.get_gateway_url();

        let response = if url == format!("/address/{CALLER}") {
            get_caller_infos()
        } else if url == "/network/config" {
            get_network_config()
        } else {
            todo!()
        };

        Ok(response)
    }

    async fn post<Body>(&self, body: &Body) -> Result<Response, Error> where Body: Serialize + Send + Sync {
        let data = serde_json::to_string(body).unwrap();

        let response = if data == r#"{"nonce":5,"value":"0","receiver":"erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0","sender":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","gasPrice":1000000000,"gasLimit":600000000,"data":"cmV0dXJuQ2FsbGVy","chainId":"D","version":1}"# {
            get_return_caller_simulation_data()
        } else if data == "ok" {
            panic!()
        } else {
            todo!()
        };

        Ok(response)
    }
}

fn get_executor() -> Arc<Mutex<BaseSimulationNetworkExecutor<MockClient>>> {
    let executor = BaseSimulationNetworkExecutor::new(
        MockClient::new(),
        Address::from_bech32_string(CALLER).unwrap()
    );

    Arc::new(Mutex::new(executor))
}

// The below test is a success if it compiles
#[tokio::test]
async fn test_clone_simulation_executor() -> Result<(), NovaXError> {
    let executor = SimulationNetworkExecutor::new("".to_string(), Address::from(CALLER));
    #[allow(clippy::redundant_clone)]
    let _executor2 = executor.clone();

    Ok(())
}

// The below test is a success if it compiles
#[tokio::test]
async fn test_debug_network_executor() -> Result<(), NovaXError> {
    let executor = SimulationNetworkExecutor::new("".to_string(), Address::from(CALLER));

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

    let contract = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    );

    contract.call(executor.clone(), 600000000);
    contract.call(executor.clone(), 600000000);

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS.to_string()
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
async fn test_call_with_bigint_arg_result() -> Result<(), NovaXError> {
    let executor = get_executor();

    let result = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    )
        .call(executor, 600000000)
        .return_big_int_arg(&BigInt::from(43i8))
        .await?;

    assert!(result.response.is_success());
    assert_eq!(result.result, Some(BigInt::from(43i8)));

    Ok(())
}