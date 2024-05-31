use multiversx_sc::codec::TopEncode;
use multiversx_sc::imports::{CodeMetadata, ManagedBuffer};
use multiversx_sc_scenario::api::StaticApi;
use num_bigint::BigUint;
use novax_data::Address;
use crate::TokenTransfer;

pub struct DeployCallInput {
    pub to: Address,
    pub function: String,
    pub arguments: Vec<Vec<u8>>,
    pub gas_limit: u64,
    pub egld_value: BigUint,
    pub esdt_transfers: Vec<TokenTransfer>
}

pub fn get_deploy_call_input(
    bytes: Vec<u8>,
    code_metadata: CodeMetadata,
    egld_value: BigUint,
    mut arguments: Vec<Vec<u8>>,
    gas_limit: u64
) -> DeployCallInput {
    let mut encoded_metadata: ManagedBuffer<StaticApi> = ManagedBuffer::new();
    code_metadata.top_encode(&mut encoded_metadata).unwrap();

    let built_in_arguments: Vec<Vec<u8>> = vec![
        bytes,
        vec![5, 0], // VM type: WASM
        encoded_metadata.to_boxed_bytes().into_vec()
    ];

    let mut all_arguments = built_in_arguments;
    all_arguments.append(&mut arguments);

    DeployCallInput {
        to: Address::from_bytes([0u8; 32]),
        function: "".to_string(),
        arguments: all_arguments,
        gas_limit,
        egld_value,
        esdt_transfers: vec![],
    }
}