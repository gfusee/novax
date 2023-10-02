use multiversx_sc_scenario::scenario_model::TxResponse;

/// A structure encapsulating the result of a contract call.
///
/// `CallResult` is designed to hold the outcome of a contract call operation. It provides
/// a clear separation between the transaction response and the actual result data, if any,
/// returned by the contract call.
pub struct CallResult<T> {
    /// The response of the transaction triggered by the contract call.
    ///
    /// This field holds all the general transaction-related information returned after a contract call,
    /// such as the transaction status, error messages (if any), and other relevant data pertaining to
    /// the transaction execution.
    pub response: TxResponse,

    /// The result data returned by the contract call, if any.
    ///
    /// This field holds the actual data returned by the contract call if the call was successful
    /// and the contract returned some data. The type `T` is a placeholder for any type that can
    /// be deserialized from the contract's response. It's an `Option<T>` since a contract call
    /// may not always return data.
    pub result: Option<T>
}