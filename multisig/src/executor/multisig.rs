use crate::errors::NO_DEPLOYED_ADDRESS;
use crate::executor::action::{Action, AsyncCallAction, DeploySCFromSourceAction, TransferAndExecuteAction};
use crate::executor::errors::{DEPLOY_BYTES_SHOULD_BE_AN_ADDRESS, ERROR_WHILE_PERFORMING_ACTION_TRANSACTION, ERROR_WHILE_RETRIEVING_FULL_ACTION_FULL_INFO, ERROR_WHILE_RETRIEVING_QUORUM, NO_ACTION_FOUND_FOR_ID_ERROR};
use crate::generated::multisig::multisig::MultisigContract;
use crate::generated::multisigview::multisigview::MultisigViewContract;
use crate::utils::map_novax_error_to_executor_error::map_novax_error_to_executor_error;
use async_trait::async_trait;
use multiversx_sc::types::CodeMetadata;
use novax::data::NativeConvertible;
use novax::executor::call_result::CallResult;
use novax::executor::{ExecutorError, TokenTransfer, TopDecodeMulti, TransactionExecutor};
use novax::Address;
use novax_executor::{DeployExecutor, NetworkQueryError, QueryExecutor, TransactionError};
use num_bigint::BigUint;
use std::time::Duration;

#[derive(Clone)]
pub struct MultisigExecutor<TxExecutor, QExecutor>
where
    TxExecutor: TransactionExecutor + Clone,
    QExecutor: QueryExecutor + Clone,
{
    multisig_address: Address,
    multisig_view_address: Address,
    gas_for_proposal: u64,
    use_async_call_action: bool,
    transaction_executor: TxExecutor,
    query_executor: QExecutor,
}

impl<TxExecutor, QExecutor> MultisigExecutor<TxExecutor, QExecutor>
where
    TxExecutor: TransactionExecutor + Clone,
    QExecutor: QueryExecutor + Clone,
{
    pub fn new(
        multisig_address: Address,
        multisig_view_address: Address,
        gas_for_proposal: u64,
        use_async_call_action: bool,
        transaction_executor: TxExecutor,
        query_executor: QExecutor,
    ) -> Self {
        Self {
            multisig_address,
            multisig_view_address,
            gas_for_proposal,
            use_async_call_action,
            transaction_executor,
            query_executor
        }
    }

    async fn get_quorum(&self) -> Result<u32, ExecutorError> {
        MultisigContract::new(self.multisig_address.to_bech32_string()?)
            .query(self.query_executor.clone())
            .get_quorum()
            .await
            .map_err(|error| map_novax_error_to_executor_error(
                error,
                |error| NetworkQueryError::Other {
                    id: ERROR_WHILE_RETRIEVING_QUORUM.to_string(),
                    reason: format!("An error occurred while retrieving the quorum:\n\n{error:?}")
                }.into()
            ))
    }

    fn decode_result_for_action<OutputManaged>(
        &self,
        action: Action,
        perform_call_result: Result<CallResult<Option<Address>>, ExecutorError>,
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        let multisig_address_bech32 = self.multisig_address.to_bech32_string()?;

        let response = match perform_call_result {
            Ok(perform_call_result) => {
                perform_call_result.response
            }
            Err(ExecutorError::Transaction(TransactionError::CannotDecodeSmartContractResult{ response })) => {
                response // There is an error in the writeLog event that might cause CannotDecodeSmartContractResult when performing an async call action on the multisig
            }
            Err(error) => {
                return Err(error);
            }
        };

        let result = action.get_nested_multisig_call_result::<OutputManaged>(
            &multisig_address_bech32,
            response.clone()
        )?;

        Ok(
            CallResult {
                result,
                response
            }
        )
    }

    async fn wait_for_signers_and_perform_action(
        &self,
        multisig_quorum: u32,
        proposal_id: u32,
        gas_limit: u64,
    ) -> Result<CallResult<Option<Address>>, ExecutorError> {
        loop {
            // TODO: write a more real-time waiting mechanism such as done in the caching crate with EachBlock.
            tokio::time::sleep(Duration::from_secs(6)).await;

            println!("Pending action {proposal_id} execution on the multisig contract...");

            let actions_full_info = MultisigViewContract::new(self.multisig_view_address.clone())
                .query(self.query_executor.clone())
                .get_pending_action_full_info()
                .await
                .map_err(|error| map_novax_error_to_executor_error(
                    error,
                    |error| NetworkQueryError::Other {
                        id: ERROR_WHILE_RETRIEVING_FULL_ACTION_FULL_INFO.to_string(),
                        reason: format!("An error occurred while retrieving all the pending actions:\n\n{error:?}")
                    }.into()
                ))?;

            let opt_action = actions_full_info
                .into_iter()
                .find(|action| action.action_id == proposal_id);

            let Some(action) = opt_action else {
                return Err(
                    TransactionError::Other {
                        id: NO_ACTION_FOUND_FOR_ID_ERROR.to_string(),
                        reason: format!("no action found with id: {proposal_id}")
                    }.into()
                );
            };

            let signers_len = action.signers.len();

            if signers_len < multisig_quorum as usize {
                println!("Pending signatures. {signers_len} have signed, {multisig_quorum} required...");
                continue;
            }

            return MultisigContract::new(self.multisig_address.clone())
                .call(
                    self.transaction_executor.clone(),
                    gas_limit
                )
                .perform_action(&proposal_id)
                .await
                .map_err(|error| {
                    map_novax_error_to_executor_error(
                        error,
                        |error| TransactionError::Other {
                            id: ERROR_WHILE_PERFORMING_ACTION_TRANSACTION.to_string(),
                            reason: format!("Found an error when performing the action:\n\n{error:?}")
                        }.into()
                    )
                });
        }
    }
}

#[async_trait]
impl<TxExecutor, QExecutor> TransactionExecutor for MultisigExecutor<TxExecutor, QExecutor>
where
    TxExecutor: TransactionExecutor + Clone,
    QExecutor: QueryExecutor + Clone,
{
    async fn sc_call<OutputManaged>(
        &mut self,
        to: &Address,
        function: String,
        arguments: Vec<Vec<u8>>,
        gas_limit: u64,
        egld_value: num_bigint::BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        let multisig_quorum = self.get_quorum().await?;

        let action = if self.use_async_call_action {
            Action::AsyncCall(
                AsyncCallAction {
                    receiver: to.clone(),
                    function_name: Some(function),
                    arguments,
                    egld_value,
                    esdt_transfers,
                }
            )
        } else {
            Action::TransferAndExecute(
                TransferAndExecuteAction {
                    receiver: to.clone(),
                    function_name: Some(function),
                    arguments,
                    egld_value,
                    esdt_transfers,
                }
            )
        };

        let proposal_id = action
            .clone()
            .propose_action(
                self.transaction_executor.clone(),
                self.multisig_address.clone(),
                self.gas_for_proposal
            )
            .await?;

        let perform_call_result = self.wait_for_signers_and_perform_action(
            multisig_quorum,
            proposal_id,
            gas_limit
        )
            .await;

        self.decode_result_for_action::<OutputManaged>(
            action,
            perform_call_result
        )
    }
}

#[async_trait]
impl<TxExecutor, QExecutor> DeployExecutor for MultisigExecutor<TxExecutor, QExecutor>
where
    TxExecutor: TransactionExecutor + Clone,
    QExecutor: QueryExecutor + Clone,
{
    async fn sc_deploy<OutputManaged>(&mut self, bytes: Vec<u8>, code_metadata: CodeMetadata, egld_value: BigUint, arguments: Vec<Vec<u8>>, gas_limit: u64) -> Result<(Address, CallResult<OutputManaged::Native>), ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        let multisig_quorum = self.get_quorum().await?;

        let source_address_bytes = vec_of_bytes_to_address(
            bytes,
            |_| {
                TransactionError::Other {
                    id: DEPLOY_BYTES_SHOULD_BE_AN_ADDRESS.to_string(),
                    reason: r#"
                    When using MultisigExecutor, contract bytes should be the address of a deployed contract having the wanted code.
                    Please pass bytes using the to_bech32_string() method on an Address.
                    "#.to_string()
                }.into()
            }
        )?;

        let action = Action::DeploySCFromSource(DeploySCFromSourceAction {
            source_address: source_address_bytes,
            arguments,
            code_metadata,
            egld_value,
        });

        let proposal_id = action.propose_action(
            self.transaction_executor.clone(),
            self.multisig_address.clone(),
            self.gas_for_proposal
        )
            .await?;

        let deployed_address_call_result = self.wait_for_signers_and_perform_action(
            multisig_quorum,
            proposal_id,
            gas_limit
        )
            .await?;

        let Some(Some(deployed_address)) = deployed_address_call_result.result else {
            return Err(
                TransactionError::Other {
                    id: NO_DEPLOYED_ADDRESS.to_string(),
                    reason: "No deployed found after performing the proposal.".to_string()
                }.into()
            )
        };

        Ok(
            (
                deployed_address,
                CallResult {
                    response: deployed_address_call_result.response,
                    result: None,
                }
            )
        )
    }
}

fn vec_of_bytes_to_address<F>(vec: Vec<u8>, error_fn: F) -> Result<Address, ExecutorError>
where
    F: FnOnce(Vec<u8>) -> ExecutorError,
{
    if vec.len() != 32 {
        return Err(error_fn(vec));
    }

    let boxed_slice: Box<[u8]> = vec.into_boxed_slice();

    // Explicitly annotate the type for `try_into`
    let boxed_array: Box<[u8; 32]> = match boxed_slice.try_into() as Result<Box<[u8; 32]>, Box<[u8]>> {
        Ok(arr) => arr,
        Err(boxed_slice) => return Err(error_fn(boxed_slice.into())), // Convert Box<[u8]> back into Vec<u8>
    };

    let address = Address::from_bytes(*boxed_array);

    if address.to_bech32_string().is_err() { // Just to check if the bytes are a valid address.
        return Err(error_fn(boxed_array.to_vec()));
    }

    Ok(address)
}