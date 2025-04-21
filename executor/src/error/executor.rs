use crate::error::network::NetworkQueryError;
use serde::{Deserialize, Serialize};
use novax_data::DataError;
use crate::error::date::DateError;
use crate::error::dummy::DummyExecutorError;
use crate::error::gateway::GatewayError;
use crate::error::mock_deploy::MockDeployError;
use crate::error::mock_transaction::MockTransactionError;
use crate::error::network_query_events::NetworkQueryEventsError;
use crate::error::transaction::TransactionError;
use crate::error::wallet::WalletError;
use crate::SimulationError;

/// An enumeration representing the various types of errors that can be encountered within the executor context.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ExecutorError {
    /// This variant wraps errors encountered during network queries, which may include issues such as connection
    /// failures or malformed requests. The wrapped `NetworkQueryError` provides more detailed information about
    /// the nature of the network-related error that occurred.
    NetworkQuery(NetworkQueryError),
    NetworkQueryEvents(NetworkQueryEventsError),

    Dummy(DummyExecutorError),
    Date(DateError),

    /// Represents errors specifically related to interactions with the MultiversX gateway. This can include
    /// HTTP request issues, response parsing errors, and other anomalies encountered while communicating
    /// with the gateway. The contained `GatewayError` elaborates on the specifics of the encountered issue,
    /// aiding in diagnosis and resolution.
    Gateway(GatewayError),

    /// Represents errors that occur during the simulation of blockchain transactions. These simulations are run on real nodes using actual data,
    /// providing a realistic environment for transaction execution without committing the results to the blockchain.
    /// The wrapped `SimulationError` offers detailed information about issues encountered during this simulation process,
    /// enabling developers to understand and rectify potential problems before live deployment.
    Simulation(SimulationError),

    /// Wraps errors related to data operations, usually arising from the `novax-data` crate. These may include errors
    /// related to data parsing, validation, or any other data-related operation. The wrapped `DataError` provides
    /// more detailed information about the nature of the data-related error that occurred.
    DataError(DataError),

    /// This variant wraps errors encountered during mock deployments. This is particularly useful when using the
    /// `MockExecutor` for testing or simulation purposes. The wrapped `MockDeployError` provides more detailed
    /// information about the nature of the mock deployment-related error that occurred.
    MockDeploy(MockDeployError),
    MockTransaction(MockTransactionError),

    Transaction(TransactionError),
    Wallet(WalletError),
}

/// An implementation of the `From` trait to allow for easy conversions from `DataError` to `ExecutorError`.
impl From<DataError> for ExecutorError {
    fn from(value: DataError) -> Self {
        ExecutorError::DataError(value)
    }
}