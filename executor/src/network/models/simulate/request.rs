use serde::Serialize;

/// A convenient struct in NovaX used to accumulate information necessary for creating a transaction simulation request.
/// This struct is internally converted into `SimulationGatewayRequestBody` when sending the request to the gateway.
pub struct SimulationGatewayRequest {
    /// The value being transferred in the transaction, represented as a stringified number.
    pub value: String,

    /// The blockchain address of the transaction's receiver.
    pub receiver: String,

    /// The blockchain address of the transaction's sender.
    pub sender: String,

    /// The maximum amount of gas that the transaction is allowed to consume.
    pub gas_limit: u64,

    /// The data payload of the transaction, typically encoded for the transaction's context.
    pub data: String,
}

/// Struct representing the body of a request for transaction simulation, sent to the MultiversX gateway.
/// This structure is used to simulate transactions via the `/transaction/cost` endpoint.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimulationGatewayRequestBody {
    /// The nonce of the transaction, ensuring transactions are processed in order and exactly once.
    pub nonce: u64,

    /// The value being transferred in the transaction, represented as a stringified number.
    pub value: String,

    /// The blockchain address of the transaction's receiver.
    pub receiver: String,

    /// The blockchain address of the transaction's sender.
    pub sender: String,

    /// The gas price for the transaction, influencing the priority and speed of processing.
    pub gas_price: u64,

    /// The maximum gas amount that can be consumed by the transaction, limiting computational work.
    pub gas_limit: u64,

    /// The data payload of the transaction, often encoded for the transaction's context.
    pub data: String,

    /// The identifier of the blockchain network for transaction processing.
    pub chain_id: String,

    /// The guardian address of the sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guardian: Option<String>,

    /// The guardian signature for the transaction. Can be anything such as '00'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guardian_signature: Option<String>,

    /// The version of the transaction structure/format for network compatibility.
    pub version: u8,

    /// The options of the transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<u8>
}
