use std::sync::Arc;
use num_bigint::BigUint;
use tokio::sync::Mutex;
use novax::code::DeployData;
use novax::CodeMetadata;
use novax::errors::NovaXError;
use novax::tester::tester::TesterContract;
use novax::executor::StandardMockExecutor;
use novax_mocking::{Account, ScenarioWorld, SetStateStep};

const CALLER: &str = "address:caller";
const CONTRACT: &str = "sc:tester";

fn get_executor() -> StandardMockExecutor {
    let mut world = ScenarioWorld::new();
    world.register_contract("file:../.novax/tester-contract.wasm", tester_contract::ContractBuilder);

    world.set_state_step(
        SetStateStep::new()
            .put_account(CALLER, Account::new().nonce(0))
            .new_address(CALLER, 0, CONTRACT)
    );

    StandardMockExecutor::new(
        Arc::new(Mutex::new(world)),
        Some(CALLER.to_string())
    )
}

#[tokio::test]
async fn test_deploy_with_biguint_arg() -> Result<(), NovaXError> {
    let mut executor = get_executor();

    let deploy_data = DeployData {
        code: "../.novax/tester-contract.wasm",
        metadata: Default::default(),
    };

    let deploy_result = TesterContract::deploy(
        deploy_data,
        &mut executor,
        600000000u64,
        &BigUint::from(5u8)
        )
        .await
        .unwrap();

    let deployed_contract = TesterContract::new(
        deploy_result.0
    );

    let result = deployed_contract
        .query(executor)
        .get_sum()
        .await
        .unwrap();

    let expected = BigUint::from(5u8);

    assert_eq!(result, expected);

    Ok(())
}

#[tokio::test]
async fn test_deploy_with_metadatas() -> Result<(), NovaXError> {
    let mut executor = get_executor();

    let deploy_data = DeployData {
        code: "../.novax/tester-contract.wasm",
        metadata: CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE,
    };

    let deploy_result = TesterContract::deploy(
        deploy_data,
        &mut executor,
        600000000u64,
        &BigUint::from(5u8)
    )
        .await
        .unwrap();

    let deployed_contract = TesterContract::new(
        deploy_result.0
    );

    let result = deployed_contract
        .query(executor)
        .get_sum()
        .await
        .unwrap();

    let expected = BigUint::from(5u8);

    assert_eq!(result, expected);

    Ok(())
}