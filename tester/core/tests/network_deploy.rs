use async_trait::async_trait;
use num_bigint::BigUint;
use novax::code::DeployData;
use novax::{Address, CodeMetadata, Wallet};
use novax::errors::NovaXError;
use novax::tester::tester::TesterContract;
use novax::testerwithreturningdeploy::testerwithreturningdeploy::TesterWithReturningDeployContract;
use novax::transaction::CallResult;
use novax::executor::BlockchainInteractor;
use novax::executor::BaseTransactionNetworkExecutor;
use novax_mocking::{CodecFrom, ScCallStep, TopEncodeMulti, TxResponse, TypedResponse, TypedScDeploy};
use crate::utils::decode_scr_data::decode_scr_data_or_panic;

mod utils;

const CALLER_PRIVATE_KEY: &str = "69417ce717e43d0d3a598f68b5e562d7d2a532a5a3ac1e8b3342515e0b2d950f"; // to anyone reading : this has been generated only for the tests below
const CALLER: &str = "erd12wf7tlsk2z895vwmndheaknkp3uaqa7xuq847numkwlmcvy60wxql2ndlk";
const NEW_CONTRACT: &str = "erd1qqqqqqqqqqqqqpgq74myhunu4sfdpmskm6s6ul8k4cetjvhhlfpsaa20la";
const RETURNING_MOCK_URL: &str = "returning";

struct MockInteractor {
    is_deploy_returning: bool
}

#[async_trait]
impl BlockchainInteractor for MockInteractor {
    async fn new(_gateway_url: &str) -> Self {
        let is_deploy_returning = _gateway_url == RETURNING_MOCK_URL;

        MockInteractor {
            is_deploy_returning,
        }
    }

    fn register_wallet(&mut self, _wallet: Wallet) -> Address {
        Address::from_bech32_string(CALLER).unwrap()
    }

    async fn sc_call<S>(&mut self, mut _sc_call_step: S) where S: AsMut<ScCallStep> + Send {
        todo!()
    }

    async fn sc_deploy_get_result<OriginalResult, RequestedResult, S>(&mut self, mut _step: S) -> (Address, TypedResponse<RequestedResult>)
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
            RequestedResult: CodecFrom<OriginalResult>,
            S: AsMut<TypedScDeploy<OriginalResult>> + Send
    {
        let mut response = if self.is_deploy_returning {
            TxResponse::from_raw_results(decode_scr_data_or_panic("@6f6b@05"))
        } else {
            TxResponse::from_raw_results(decode_scr_data_or_panic("@6f6b"))
        };

        response.new_deployed_address = Some(Address::from_bech32_string(NEW_CONTRACT).unwrap().into());
        let step = _step.as_mut();
        step.sc_deploy_step.response = Some(response.clone());

        (
            NEW_CONTRACT.into(),
            TypedResponse::from_raw(&response)
        )
    }
}


fn get_executor(is_returning_value: bool) -> BaseTransactionNetworkExecutor<MockInteractor> {
    let wallet = Wallet::from_private_key(CALLER_PRIVATE_KEY).unwrap();

    let url = if is_returning_value {
        RETURNING_MOCK_URL.to_string()
    } else {
        "".to_string()
    };

    BaseTransactionNetworkExecutor::new(
        &url,
        &wallet
    )
}

#[tokio::test]
async fn test_deploy_with_biguint_arg() -> Result<(), NovaXError> {
    let mut executor = get_executor(false);

    let deploy_data = DeployData {
        code: "../.novax/tester-contract.wasm",
        metadata: Default::default(),
    };

    let deploy_result: (Address, CallResult<()>) = TesterContract::deploy(
        deploy_data,
        &mut executor,
        600000000u64,
        &BigUint::from(5u8)
        )
        .await
        .unwrap();

    assert_eq!(deploy_result.0, Address::from(NEW_CONTRACT));

    Ok(())
}

#[tokio::test]
async fn test_deploy_with_return_value() -> Result<(), NovaXError> {
    let mut executor = get_executor(true);

    let deploy_data = DeployData {
        code: "../.novax/tester-contract.wasm",
        metadata: Default::default(),
    };

    let deploy_result: (Address, CallResult<BigUint>) = TesterWithReturningDeployContract::deploy(
        deploy_data,
        &mut executor,
        600000000u64,
        &BigUint::from(5u8)
    )
        .await
        .unwrap();

    assert_eq!(deploy_result.0, Address::from(NEW_CONTRACT));
    assert_eq!(deploy_result.1.result.unwrap(), BigUint::from(5u8));

    Ok(())
}

#[tokio::test]
async fn test_deploy_with_biguint_arg_with_metadatas() -> Result<(), NovaXError> {
    let mut executor = get_executor(false);

    let deploy_data = DeployData {
        code: "../.novax/tester-contract.wasm",
        metadata: CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE,
    };

    let deploy_result: (Address, CallResult<()>) = TesterContract::deploy(
        deploy_data,
        &mut executor,
        600000000u64,
        &BigUint::from(5u8)
    )
        .await
        .unwrap();

    assert_eq!(deploy_result.0, Address::from(NEW_CONTRACT));

    Ok(())
}