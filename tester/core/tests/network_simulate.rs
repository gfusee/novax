mod utils;

use std::sync::Arc;
use async_trait::async_trait;
use hyper::{http, StatusCode};
use tokio::sync::Mutex;
use novax::Address;
use novax::errors::NovaXError;
use num_bigint::BigUint;
use reqwest::{Body, Response};
use serde::Serialize;
use novax::tester::tester::TesterContract;
use novax::executor::{BaseSimulationNetworkExecutor, SimulationNetworkExecutor};
use novax_request::error::request::RequestError;
use novax_request::gateway::client::GatewayClient;

const CALLER: &str = "erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g";
const TESTER_CONTRACT_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0";

fn get_caller_infos() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"account":{"address":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","nonce":5,"balance":"49893375980000000000","username":"","code":"","codeHash":null,"rootHash":null,"codeMetadata":null,"developerReward":"0","ownerAddress":""},"blockInfo":{"nonce":1514622,"hash":"119621492bad699ac2a60ad276720d1735c1d0eebfe70a82498d8a613a22063a","rootHash":"6ba976a765877a1d9183ca270fc0897ff6b23f30411125243394ed39b309a0b1"}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

fn get_network_config() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"config":{"erd_adaptivity":"false","erd_chain_id":"D","erd_denomination":18,"erd_extra_gas_limit_guarded_tx":50000,"erd_gas_per_data_byte":1500,"erd_gas_price_modifier":"0.01","erd_hysteresis":"0.200000","erd_latest_tag_software_version":"D1.6.6.1","erd_max_gas_per_transaction":600000000,"erd_meta_consensus_group_size":58,"erd_min_gas_limit":50000,"erd_min_gas_price":1000000000,"erd_min_transaction_version":1,"erd_num_metachain_nodes":58,"erd_num_nodes_in_shard":58,"erd_num_shards_without_meta":3,"erd_rewards_top_up_gradient_point":"2000000000000000000000000","erd_round_duration":6000,"erd_rounds_per_epoch":2400,"erd_shard_consensus_group_size":21,"erd_start_time":1694000000,"erd_top_up_factor":"0.500000"}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

fn get_return_caller_simulation_data() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"txGasUnits":2384920,"returnMessage":"","smartContractResults":{"4b34385c5a43aa4e2f8b66f63f0e1786aef3e2acff288bd4c2669e71f9078deb":{"nonce":6,"value":26150800000000,"receiver":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","sender":"erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0","data":"@6f6b@e5f5ec2bf6b925565fd1ed99e958858250ce40fd73b12d5792e68bbda679a73c","prevTxHash":"0e888497084b1d6581698537b0a1f1c6f50841fbb5188b5e9b8ebc338662a49f","originalTxHash":"0e888497084b1d6581698537b0a1f1c6f50841fbb5188b5e9b8ebc338662a49f","gasLimit":0,"gasPrice":1000000000,"callType":0,"operation":"transfer","isRefund":true}}},"error":"","code":"successful"}"#.to_string();

    (status, data)
}

fn get_return_biguint_argument_simulation_data() -> (StatusCode, String) {
    let status = StatusCode::OK;
    let data = r#"{"data":{"txGasUnits":2442787,"returnMessage":"","smartContractResults":{}},"error":"","code":"successful"}"#.to_string();

    (status, data)
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

    async fn get(&self) -> Result<(StatusCode, Option<String>), RequestError> {
        let url = self.get_gateway_url();

        let result = if url == format!("/address/{CALLER}") {
            get_caller_infos()
        } else if url == "/network/config" {
            get_network_config()
        } else {
            unreachable!()
        };

        Ok((result.0, Some(result.1)))
    }

    async fn post<Body>(&self, body: &Body) -> Result<(StatusCode, Option<String>), RequestError> where Body: Serialize + Send + Sync {
        let data = serde_json::to_string(body).unwrap();

        let result = if data == r#"{"nonce":5,"value":"0","receiver":"erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0","sender":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","gasPrice":1000000000,"gasLimit":600000000,"data":"cmV0dXJuQ2FsbGVy","chainId":"D","version":1}"# {
            get_return_caller_simulation_data()
        } else if data == r#"{"nonce":5,"value":"0","receiver":"erd1qqqqqqqqqqqqqpgq7x53hfeg9558dmzjg9lqyfar77z8wrxf5u7qrawwh0","sender":"erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g","gasPrice":1000000000,"gasLimit":600000000,"data":"YWRkQDBh","chainId":"D","version":1}"# {
            get_return_biguint_argument_simulation_data()
        } else {
            unreachable!()
        };

        Ok((result.0, Some(result.1)))
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
async fn test_call_with_biguint_argument() -> Result<(), NovaXError> {
    let executor = get_executor();

    let contract = TesterContract::new(
        TESTER_CONTRACT_ADDRESS
    );

    contract
        .call(executor, 600000000)
        .add(&BigUint::from(10u8))
        .await?;

    Ok(())
}

// We don't need more tests for this executor