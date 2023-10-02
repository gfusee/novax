use multiversx_sc::types::CodeMetadata;
use crate::code::bytes::AsBytesValue;

/// A structure representing the data necessary for deploying a smart contract.
#[derive(Clone, Debug)]
pub struct DeployData<Bytes: AsBytesValue> {
    /// The bytecode of the contract to be deployed.
    ///
    /// This field contains the bytecode of the contract that is to be deployed on the blockchain.
    /// The bytecode should be valid WebAssembly (Wasm) code.
    pub code: Bytes,

    /// The metadata of the contract.
    ///
    /// This field contains metadata about the contract, indicating its capabilities such as whether
    /// it is upgradeable, readable, payable, or payable by a smart contract (SC). The `CodeMetadata`
    /// type is defined in the `multiversx-sdk` crate.
    pub metadata: CodeMetadata
}