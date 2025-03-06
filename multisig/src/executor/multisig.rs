use async_trait::async_trait;
use novax::data::NativeConvertible;
use novax::executor::call_result::CallResult;
use novax::executor::{ExecutorError, TokenTransfer, TopDecodeMulti, TransactionExecutor};
use novax::Address;
use novax_executor::{NormalizationInOut, QueryExecutor, TransactionOnNetwork};
use std::time::Duration;
use crate::generated::multisig::multisig::MultisigContract;
use crate::generated::multisigview::multisigview::MultisigViewContract;

pub struct MultisigExecutor<TxExecutor, QExecutor>
where
    TxExecutor: TransactionExecutor + Clone,
    QExecutor: QueryExecutor + Clone,
{
    multisig_address: Address,
    multisig_view_address: Address,
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
        transaction_executor: TxExecutor,
        query_executor: QExecutor,
    ) -> Self {
        Self {
            multisig_address,
            multisig_view_address,
            transaction_executor,
            query_executor,
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
        let multisig_quorum = MultisigContract::new(self.multisig_address.clone())
            .query(self.query_executor.clone())
            .get_quorum()
            .await;

        let Ok(multisig_quorum) = multisig_quorum else {
            todo!()
        };

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
                gas_limit // TODO: set a fixed and reasonable gas limit for proposals
            )
            .propose_async_call(
                &Address::from_bech32_string(&normalized.receiver)?,
                &normalized.egld_value,
                &function_call
            )
            .await;

        let Ok(propose_call_result) = propose_call_result else {
            todo!()
        };

        let Some(proposal_id) = propose_call_result.result else {
            todo!()
        };

        let multisig_address_bech32 = self.multisig_address.to_bech32_string()?;
        let (result, tx) = loop {
            // TODO: write a more real-time wait mechanism such as done in the caching crate with EachBlock.
            tokio::time::sleep(Duration::from_secs(6)).await;

            println!("Pending action {proposal_id} execution on the multisig contract...");

            let actions_full_info = MultisigViewContract::new(self.multisig_view_address.clone())
                .query(self.query_executor.clone())
                .get_pending_action_full_info()
                .await;

            let Ok(actions_full_info) = actions_full_info else {
                todo!()
            };

            let opt_action = actions_full_info
                .into_iter()
                .find(|action| action.action_id == proposal_id);

            let Some(action) = opt_action else {
                todo!()
            };

            let signers_len = action.signers.len();

            if signers_len  < multisig_quorum as usize {
                println!("Pending signatures. {signers_len} have signed, {multisig_quorum} required...");
                continue;
            }

            let perform_call_result = MultisigContract::new(self.multisig_address.clone())
                .call(
                    self.transaction_executor.clone(),
                    gas_limit
                )
                .perform_action(&proposal_id)
                .await;

            let Ok(perform_call_result) = perform_call_result else {
                todo!()
            };

            let result = get_nested_multisig_call_result::<OutputManaged>(
                &multisig_address_bech32,
                perform_call_result.response.clone()
            );

            break (result, perform_call_result.response);
        };

        Ok(
            CallResult {
                response: tx,
                result,
            }
        )
    }
}

fn get_nested_multisig_call_result<OutputManaged>(
    multisig_address: &str,
    tx: TransactionOnNetwork
) -> Option<OutputManaged::Native>
where
    OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
{
    let async_call_success_event_identifier = "callBack";
    let async_call_success_topic_identifier = "asyncCallSuccess";

    let async_call_success_event = tx
        .transaction
        .logs
        .into_iter()
        .find(|log| log.address == multisig_address)?
        .events
        .into_iter()
        .find(|event| {
            return event.address == multisig_address
                && event.identifier == async_call_success_event_identifier
                && event.topics.get(0) == Some(async_call_success_topic_identifier.to_string()).as_ref()
        })?;

    if async_call_success_event.topics.len() < 2 {
        return None;
    }

    let encoded_data = async_call_success_event.topics[1..].to_vec();
    let mut encoded_data_bytes: Vec<Vec<u8>> = encoded_data
        .into_iter()
        .map(|encoded_arg| hex::decode(encoded_arg).expect("error hex-decoding result"))
        .collect();

    let Ok(decoded) = OutputManaged::multi_decode(&mut encoded_data_bytes) else {
        return None;
    };

    Some(decoded.to_native())
}