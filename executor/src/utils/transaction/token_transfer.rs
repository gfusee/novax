use num_bigint::BigUint;

/// A structure representing the data necessary for transferring tokens during a contract call.
///
/// `TokenTransfer` is used to specify the details of a token transfer that is to be carried out as part
/// of a contract call. It includes the identifier of the token, a nonce, and the amount of tokens to be transferred.
/// This struct is solely used for providing token transfer information as a parameter to a contract call. In case
/// there is a token payment return from the contract, a different struct (which would be generated from the ABI along
/// with the client) named `EsdtTokenPayment` would be used to represent that data.
#[derive(PartialEq, Clone, Debug)]
pub struct TokenTransfer {
    /// A string representing the identifier of the token to be transferred.
    ///
    /// This field contains the unique identifier of the ESDT (Elrond Standard Digital Token) that is to be transferred.
    pub identifier: String,

    /// A nonce value associated with the token transfer.
    ///
    /// In the context of token transfers, a nonce can be used to differentiate between distinct transfers and
    /// ensure the idempotency of operations.
    pub nonce: u64,

    /// The amount of tokens to be transferred, represented as a `BigUint`.
    ///
    /// This field specifies the quantity of tokens that are to be transferred as part of the contract call.
    pub amount: BigUint
}