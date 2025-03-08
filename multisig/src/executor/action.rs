use multiversx_sc::types::CodeMetadata;
use crate::executor::multisig::{ERROR_WHILE_PROPOSING_ACTION_TRANSACTION, NONE_PROPOSAL_ID_ERROR};
use crate::generated::multisig::multisig::MultisigContract;
use crate::utils::map_novax_error_to_executor_error::map_novax_error_to_executor_error;
use novax_data::Address;
use novax_executor::{ExecutorError, NormalizationInOut, TokenTransfer, TransactionError, TransactionExecutor};
use num_bigint::BigUint;

pub(crate) enum Action {
    AsyncCall(AsyncCallAction),
    DeploySCFromSource(DeploySCFromSourceAction)
}

pub(crate) struct AsyncCallAction {
    pub receiver: Address,
    pub function_name: Option<String>,
    pub arguments: Vec<Vec<u8>>,
    pub egld_value: BigUint,
    pub esdt_transfers: Vec<TokenTransfer>,
}

pub(crate) struct DeploySCFromSourceAction {
    pub source_address: Address,
    pub arguments: Vec<Vec<u8>>,
    pub code_metadata: CodeMetadata,
    pub egld_value: BigUint,
}

impl Action {
    pub(crate) async fn propose_action<TxExecutor>(
        self,
        transaction_executor: TxExecutor,
        multisig_address: Address,
        gas_for_proposal: u64
    ) -> Result<u32, ExecutorError>
    where
        TxExecutor: TransactionExecutor + Clone,
    {
        match self {
            Action::AsyncCall(async_call_action) => {
                propose_async_call_action(
                    async_call_action,
                    transaction_executor,
                    multisig_address,
                    gas_for_proposal,
                )
                    .await
            }
            Action::DeploySCFromSource(deploy_from_source_action) => {
                propose_deploy_from_source_action(
                    deploy_from_source_action,
                    transaction_executor,
                    multisig_address,
                    gas_for_proposal
                )
                    .await
            }
        }
    }
}

pub(crate) async fn propose_async_call_action<TxExecutor>(
    async_call_action: AsyncCallAction,
    transaction_executor: TxExecutor,
    multisig_address: Address,
    gas_for_proposal: u64
) -> Result<u32, ExecutorError>
where
    TxExecutor: TransactionExecutor + Clone,
{
    let to_bech32_string = async_call_action.receiver.to_bech32_string()?;

    let normalized = NormalizationInOut {
        sender: to_bech32_string.clone(),
        receiver: to_bech32_string,
        function_name: async_call_action.function_name,
        arguments: async_call_action.arguments,
        egld_value: async_call_action.egld_value,
        esdt_transfers: async_call_action.esdt_transfers,
    }.normalize()?;

    let mut function_call: Vec<String> = vec![
        normalized.function_name.unwrap_or_default()
    ];

    function_call.append(
        &mut convert_bytes_args_to_strings(normalized.arguments)
    );

    let propose_call_result = MultisigContract::new(multisig_address)
        .call(
            transaction_executor.clone(),
            gas_for_proposal
        )
        .propose_async_call(
            &Address::from_bech32_string(&normalized.receiver)?,
            &normalized.egld_value,
            &function_call
        )
        .await
        .map_err(|error| map_novax_error_to_executor_error(
            error,
            |error| TransactionError::Other {
                id: ERROR_WHILE_PROPOSING_ACTION_TRANSACTION.to_string(),
                reason: format!("An error occurred while proposing the action:\n\n{error:?}")
            }.into()
        ))?;

    let Some(proposal_id) = propose_call_result.result else {
        return Err(
            TransactionError::Other {
                id: NONE_PROPOSAL_ID_ERROR.to_string(),
                reason: "proposal_id is none".to_string()
            }.into()
        );
    };

    Ok(proposal_id)
}

pub(crate) async fn propose_deploy_from_source_action<TxExecutor>(
    deploy_from_source_action: DeploySCFromSourceAction,
    transaction_executor: TxExecutor,
    multisig_address: Address,
    gas_for_proposal: u64
) -> Result<u32, ExecutorError>
where
    TxExecutor: TransactionExecutor + Clone,
{
    let propose_call_result = MultisigContract::new(multisig_address)
        .call(
            transaction_executor.clone(),
            gas_for_proposal
        )
        .propose_sc_deploy_from_source(
            &deploy_from_source_action.egld_value,
            &deploy_from_source_action.source_address,
            &deploy_from_source_action.code_metadata.bits(),
            &convert_bytes_args_to_strings(deploy_from_source_action.arguments),
        )
        .await
        .map_err(|error| map_novax_error_to_executor_error(
            error,
            |error| TransactionError::Other {
                id: ERROR_WHILE_PROPOSING_ACTION_TRANSACTION.to_string(),
                reason: format!("An error occurred while proposing the action:\n\n{error:?}")
            }.into()
        ))?;

    let Some(proposal_id) = propose_call_result.result else {
        return Err(
            TransactionError::Other {
                id: NONE_PROPOSAL_ID_ERROR.to_string(),
                reason: "proposal_id is none".to_string()
            }.into()
        );
    };

    Ok(proposal_id)
}

fn convert_bytes_args_to_strings(
    arguments: Vec<Vec<u8>>
) -> Vec<String> {
    let mut strings = Vec::new();

    for arg in arguments {
        // The only operation done on the unsafe String is .as_bytes in .to_managed.
        // Therefore, we can accept this unsafe call.
        let arg_string_unsafe = unsafe {
            String::from_utf8_unchecked(arg)
        };

        strings.push(arg_string_unsafe);
    }

    strings
}
