use std::time::Duration;
use async_trait::async_trait;
use novax::data::NativeConvertible;
use novax::executor::call_result::CallResult;
use novax::executor::{ExecutorError, TokenTransfer, TopDecodeMulti, TransactionExecutor};
use novax::multisig::multisig::MultisigContract;
use novax::multisigview::multisigview::MultisigViewContract;
use novax::Address;
use novax::multisig::multisig::Action::SendTransferExecute;
use novax_executor::{NormalizationInOut, QueryExecutor};

struct MultisigExecutor<TxExecutor, QExecutor>
where
    TxExecutor: TransactionExecutor + Clone,
    QExecutor: QueryExecutor + Clone,
{
    multisig_address: Address,
    multisig_view_address: Address,
    transaction_executor: TxExecutor,
    query_executor: QExecutor,
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
        let to_bech32_string = to.to_bech32_string()?;

        let normalized = NormalizationInOut {
            sender: to_bech32_string.clone(),
            receiver: to_bech32_string,
            function_name: Some(function),
            arguments,
            egld_value,
            esdt_transfers,
        }.normalize()?;

        let mut function_call: Vec<String> = vec![
            normalized.function_name.unwrap_or_default()
        ];

        for arg in normalized.arguments {
            // The only operation done on the unsafe String is .as_bytes in .to_managed.
            // Therefore, we can accept this unsafe call.
            let arg_string_unsafe = unsafe {
                String::from_utf8_unchecked(arg)
            };

            function_call.push(
                arg_string_unsafe
            )
        }

        let propose_call_result = MultisigContract::new(self.multisig_address.clone())
            .call(
                self.transaction_executor.clone(),
                gas_limit
            )
            .propose_transfer_execute(
                &Address::from_bech32_string(&normalized.receiver)?,
                &normalized.egld_value,
                &function_call
            )
            .await;

        let Ok(propose_call_result) = propose_call_result else {
        };

        let Some(proposal_id) = propose_call_result.result else {
        };

        loop {
            println!("Pending action {proposal_id} execution on the multisig contract...");

            let actions_full_info = MultisigViewContract::new(self.multisig_view_address.clone())
                .query(self.query_executor.clone())
                .get_pending_action_full_info()
                .await;

            let Ok(actions_full_info) = actions_full_info else {

            };

            let opt_action = actions_full_info
                .into_iter()
                .find(|action| action.action_id == proposal_id);

            let Some(action) = opt_action else {

            };

            // TODO: write a more real-time wait mechanism such as done in the caching crate with EachBlock.
            tokio::time::sleep(Duration::from_secs(6)).await;
        }
    }
}

