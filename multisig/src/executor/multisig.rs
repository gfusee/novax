use async_trait::async_trait;
use novax::data::NativeConvertible;
use novax::executor::call_result::CallResult;
use novax::executor::{ExecutorError, TokenTransfer, TopDecodeMulti, TransactionExecutor};
use novax::Address;
use novax_executor::{NetworkQueryError, NormalizationInOut, QueryExecutor, TransactionError, TransactionOnNetwork};
use std::time::Duration;
use base64::Engine;
use novax::errors::NovaXError;
use novax_executor::results::decode_topic;
use crate::generated::multisig::multisig::MultisigContract;
use crate::generated::multisigview::multisigview::MultisigViewContract;

pub const NONE_PROPOSAL_ID_ERROR: &str = "NOVAX_MULTISIG_NONE_PROPOSAL_ID_ERROR";
pub const NO_ACTION_FOUND_FOR_ID_ERROR: &str = "NOVAX_MULTISIG_NO_ACTION_FOUND_FOR_ID_ERROR";
pub const ERROR_WHILE_RETRIEVING_QUORUM: &str = "NOVAX_MULTISIG_ERROR_WHILE_PERFORMING_ACTION_TRANSACTION";
pub const ERROR_WHILE_RETRIEVING_FULL_ACTION_FULL_INFO: &str = "NOVAX_MULTISIG_ERROR_WHILE_RETRIEVING_FULL_ACTION_FULL_INFO";
pub const ERROR_WHILE_PROPOSING_ACTION_TRANSACTION: &str = "NOVAX_MULTISIG_ERROR_WHILE_PROPOSING_ACTION_TRANSACTION";
pub const ERROR_WHILE_PERFORMING_ACTION_TRANSACTION: &str = "NOVAX_MULTISIG_ERROR_WHILE_PERFORMING_ACTION_TRANSACTION";

pub struct MultisigExecutor<TxExecutor, QExecutor>
where
    TxExecutor: TransactionExecutor + Clone,
    QExecutor: QueryExecutor + Clone,
{
    multisig_address: Address,
    multisig_view_address: Address,
    gas_for_proposal: u64,
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
        transaction_executor: TxExecutor,
        query_executor: QExecutor,
    ) -> Self {
        Self {
            multisig_address,
            multisig_view_address,
            gas_for_proposal,
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
            .await
            .map_err(|error| map_novax_error_to_executor_error(
                error,
                |error| NetworkQueryError::Other {
                    id: ERROR_WHILE_RETRIEVING_QUORUM.to_string(),
                    reason: format!("An error occurred while retrieving the quorum:\n\n{error:?}")
                }.into()
            ))?;

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
                self.gas_for_proposal
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

        let multisig_address_bech32 = self.multisig_address.to_bech32_string()?;
        let (result, tx) = loop {
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

            let response = match perform_call_result {
                Ok(perform_call_result) => {
                    perform_call_result.response
                }
                Err(NovaXError::Executor(ExecutorError::Transaction(TransactionError::CannotDecodeSmartContractResult{ response }))) => {
                    response // There is an error in the writeLog event that might cause CannotDecodeSmartContractResult when performing an async call action on the multisig
                }
                Err(error) => {
                    return Err(
                        map_novax_error_to_executor_error(
                            error,
                            |error| TransactionError::Other {
                                id: ERROR_WHILE_PERFORMING_ACTION_TRANSACTION.to_string(),
                                reason: format!("Found an error when performing the action:\n\n{error:?}")
                            }.into()
                        )
                    )
                }
            };

            let result = get_nested_multisig_call_result::<OutputManaged>(
                &multisig_address_bech32,
                response.clone()
            );

            break (result, response);
        };

        Ok(
            CallResult {
                response: tx,
                result,
            }
        )
    }
}

fn map_novax_error_to_executor_error<F>(
    error: NovaXError,
    or_else: F
) -> ExecutorError
where
    F: FnOnce(NovaXError) -> ExecutorError
{
    match error {
        NovaXError::Executor(error) => error,
        _ => or_else(error).into()
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
            let Some(first_topic) = event.topics.get(0) else {
                return false;
            };

            return event.address == multisig_address
                && event.identifier == async_call_success_event_identifier
                && decode_topic(first_topic) == Ok(async_call_success_topic_identifier.to_string())
        })?;

    if async_call_success_event.topics.len() < 2 {
        return None;
    }

    let mut encoded_data = async_call_success_event
        .topics[1..]
        .to_vec()
        .into_iter()
        .map(|topic| base64::engine::general_purpose::STANDARD.decode(topic).expect("Decoding failed"))
        .collect::<Vec<Vec<u8>>>();

    let Ok(decoded) = OutputManaged::multi_decode(&mut encoded_data) else {
        return None;
    };

    Some(decoded.to_native())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use multiversx_sc::types::{EsdtTokenPayment, TokenIdentifier};
    use multiversx_sc_scenario::api::StaticApi;
    use num_bigint::BigUint;
    use novax_data::Payment;
    use novax_executor::TransactionOnNetworkResponse;
    use crate::executor::multisig::get_nested_multisig_call_result;

    #[tokio::test]
    async fn test_get_nested_multisig_call_result_with_simple_xexchange_view() {
        let data = r#"
        {
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "SCInvoking",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "2a86b5e18a92495cf05b1c64c88d5de8956d86388c74537af595953bc1a0b909",
      "nonce": 8,
      "round": 7879538,
      "epoch": 3257,
      "value": "0",
      "receiver": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
      "sender": "erd1ceqgv3cj5unwvsednmsxp300kx6rvcgfz297wvdyaztea4yfme2s9ncxue",
      "gasPrice": 1000000000,
      "gasLimit": 600000000,
      "gasUsed": 600000000,
      "data": "cGVyZm9ybUFjdGlvbkAwMg==",
      "signature": "fe5c1173c9e86336dae979e20e949403fb737726c9aa7c1527227de280808f00faf209cb838461e2ae4e0add45a13dfb6a80c1b7faa6fb266560530f3b7c670c",
      "sourceShard": 1,
      "destinationShard": 1,
      "blockNonce": 7808964,
      "blockHash": "067fbd753023a373061f86a30f6fd416aee9d0d8e7390f0ea094bd9ce82f49df",
      "notarizedAtSourceInMetaNonce": 7812482,
      "NotarizedAtSourceInMetaHash": "b3704a10532b4aa78ddff79492874161f6b1b562ea61e05e28864a4cf94cec58",
      "notarizedAtDestinationInMetaNonce": 7812482,
      "notarizedAtDestinationInMetaHash": "b3704a10532b4aa78ddff79492874161f6b1b562ea61e05e28864a4cf94cec58",
      "miniblockType": "TxBlock",
      "miniblockHash": "f99a5ba4a58cda52919dd8ab05d6375c087b14bdba3ae793f9008c012bc0c441",
      "hyperblockNonce": 7812482,
      "hyperblockHash": "b3704a10532b4aa78ddff79492874161f6b1b562ea61e05e28864a4cf94cec58",
      "timestamp": 1741277228,
      "logs": {
        "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
        "events": [
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "performAction",
            "topics": [
              "c3RhcnRQZXJmb3JtQWN0aW9u"
            ],
            "data": "AAAAAgYAAAAAAAAAAAUAWBNyFLDhTClIYKFsEQQqpxq8FyB86wAAAAAAAAAPZ2V0Rmlyc3RUb2tlbklkAAAAAAAAAAHGQIZHEqcm5kMtnuBgxe+xtDZhCRKL5zGk6Jee1IneVQ==",
            "additionalData": [
              "AAAAAgYAAAAAAAAAAAUAWBNyFLDhTClIYKFsEQQqpxq8FyB86wAAAAAAAAAPZ2V0Rmlyc3RUb2tlbklkAAAAAAAAAAHGQIZHEqcm5kMtnuBgxe+xtDZhCRKL5zGk6Jee1IneVQ=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "performAction",
            "topics": [
              "cGVyZm9ybUFzeW5jQ2FsbA==",
              "Ag==",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=",
              "",
              "I3t+qw==",
              "Z2V0Rmlyc3RUb2tlbklk"
            ],
            "data": null,
            "additionalData": [
              ""
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "transferValueOnly",
            "topics": [
              "",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs="
            ],
            "data": "QXN5bmNDYWxs",
            "additionalData": [
              "QXN5bmNDYWxs",
              "Z2V0Rmlyc3RUb2tlbklk"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "transferValueOnly",
            "topics": [
              "",
              "AAAAAAAAAAAFACr0ieZrWRU5IrwFF6xYfI+djGnS3lU="
            ],
            "data": "QXN5bmNDYWxsYmFjaw==",
            "additionalData": [
              "QXN5bmNDYWxsYmFjaw==",
              "Y2FsbEJhY2s=",
              "AA==",
              "V0VHTEQtYTI4YzU5"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "callBack",
            "topics": [
              "YXN5bmNDYWxsU3VjY2Vzcw==",
              "V0VHTEQtYTI4YzU5"
            ],
            "data": null,
            "additionalData": [
              ""
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "writeLog",
            "topics": [
              "xkCGRxKnJuZDLZ7gYMXvsbQ2YQkSi+cxpOiXntSJ3lU=",
              "QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk5OTI2MDAwLCBnYXMgdXNlZCA9IDEzMTg0OTgz"
            ],
            "data": "QDZmNmJANTc0NTQ3NGM0NDJkNjEzMjM4NjMzNTM5",
            "additionalData": [
              "QDZmNmJANTc0NTQ3NGM0NDJkNjEzMjM4NjMzNTM5"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "completedTxEvent",
            "topics": [
              "Koa14YqSSVzwWxxkyI1d6JVthjiMdFN69ZWVO8GguQk="
            ],
            "data": null,
            "additionalData": null
          }
        ]
      },
      "status": "success",
      "operation": "transfer",
      "function": "performAction",
      "initiallyPaidFee": "6073260000000000",
      "fee": "6073260000000000",
      "chainID": "D",
      "version": 1,
      "options": 0
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        let tx_on_network = serde_json::from_str::<TransactionOnNetworkResponse>(data)
            .unwrap()
            .data
            .unwrap();

        let result = get_nested_multisig_call_result::<TokenIdentifier<StaticApi>>(
            "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            tx_on_network
        );

        let expected = Some("WEGLD-a28c59".to_string());

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_nested_multisig_call_result_with_xexchange_swap() {
        let data = r#"
        {
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "SCInvoking",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "1397d0b339019189295e6bd3eb0f42e17d54cbcdf8fb442b045a920183bc733c",
      "nonce": 19,
      "round": 7880177,
      "epoch": 3257,
      "value": "0",
      "receiver": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
      "sender": "erd1ceqgv3cj5unwvsednmsxp300kx6rvcgfz297wvdyaztea4yfme2s9ncxue",
      "gasPrice": 1000000000,
      "gasLimit": 600000000,
      "gasUsed": 600000000,
      "data": "cGVyZm9ybUFjdGlvbkAwOA==",
      "signature": "fd7222e4ad44b03b24061eea9321e731925c4efdfc387c718b9ecb415a096e7cd73ec65f9bdde21363eae26eb16eeea385f567432611cf62a6caa389d1c56704",
      "sourceShard": 1,
      "destinationShard": 1,
      "blockNonce": 7809603,
      "blockHash": "b730422e1c71aca43f888b78ddc28ed141f3589324b3feb5b8d2825d212b9018",
      "notarizedAtSourceInMetaNonce": 7813121,
      "NotarizedAtSourceInMetaHash": "6fe407c9f69597d4812518f337d75c079d775639a638ab0f2d09212a7c5b3603",
      "notarizedAtDestinationInMetaNonce": 7813121,
      "notarizedAtDestinationInMetaHash": "6fe407c9f69597d4812518f337d75c079d775639a638ab0f2d09212a7c5b3603",
      "miniblockType": "TxBlock",
      "miniblockHash": "cd55ee5d166ac1d7bce0271d2d79deea46f8d0738e8549141bedcd81ddf2b90d",
      "hyperblockNonce": 7813121,
      "hyperblockHash": "6fe407c9f69597d4812518f337d75c079d775639a638ab0f2d09212a7c5b3603",
      "timestamp": 1741281062,
      "smartContractResults": [
        {
          "hash": "b40371da314284084a4965e430d04963e257206f817f9afec0d7709156825882",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "sender": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
          "data": "ESDTTransfer@555344432d333530633465@0186a0@73776170546f6b656e734669786564496e707574@5745474c442d613238633539@01@9b5cbeca2a41fef732f8774f06de4ad8308bbffed04a9b191d07c124137e7dc9@1397d0b339019189295e6bd3eb0f42e17d54cbcdf8fb442b045a920183bc733c",
          "prevTxHash": "1397d0b339019189295e6bd3eb0f42e17d54cbcdf8fb442b045a920183bc733c",
          "originalTxHash": "1397d0b339019189295e6bd3eb0f42e17d54cbcdf8fb442b045a920183bc733c",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 1,
          "originalSender": "erd1ceqgv3cj5unwvsednmsxp300kx6rvcgfz297wvdyaztea4yfme2s9ncxue",
          "tokens": [
            "USDC-350c4e"
          ],
          "esdtValues": [
            "100000"
          ],
          "operation": "ESDTTransfer",
          "function": "swapTokensFixedInput"
        },
        {
          "hash": "980e369903305b05bf78eb36a6ac0c1fa0487105ba43b13bcebb97562088d323",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "ESDTTransfer@5745474c442d613238633539@03a90ec444e3@737761704e6f466565416e64466f7277617264@4d45582d613635396430@0000000000000000000000000000000000000000000000000000000000000000",
          "prevTxHash": "1397d0b339019189295e6bd3eb0f42e17d54cbcdf8fb442b045a920183bc733c",
          "originalTxHash": "1397d0b339019189295e6bd3eb0f42e17d54cbcdf8fb442b045a920183bc733c",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1ceqgv3cj5unwvsednmsxp300kx6rvcgfz297wvdyaztea4yfme2s9ncxue",
          "tokens": [
            "WEGLD-a28c59"
          ],
          "esdtValues": [
            "4024632100067"
          ],
          "operation": "ESDTTransfer",
          "function": "swapNoFeeAndForward"
        },
        {
          "hash": "2879b3a1708013cad7cbd65fd39f28793aaaf7f54bd15d165f8e2cf0b73ef9b9",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "ESDTTransfer@5745474c442d613238633539@0e4166996072be",
          "prevTxHash": "1397d0b339019189295e6bd3eb0f42e17d54cbcdf8fb442b045a920183bc733c",
          "originalTxHash": "1397d0b339019189295e6bd3eb0f42e17d54cbcdf8fb442b045a920183bc733c",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1ceqgv3cj5unwvsednmsxp300kx6rvcgfz297wvdyaztea4yfme2s9ncxue",
          "tokens": [
            "WEGLD-a28c59"
          ],
          "esdtValues": [
            "4012558589653694"
          ],
          "operation": "ESDTTransfer"
        }
      ],
      "logs": {
        "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
        "events": [
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "performAction",
            "topics": [
              "c3RhcnRQZXJmb3JtQWN0aW9u"
            ],
            "data": "AAAACAYAAAAAAAAAAAUAWBNyFLDhTClIYKFsEQQqpxq8FyB86wAAAAAAAAAMRVNEVFRyYW5zZmVyAAAABQAAAAtVU0RDLTM1MGM0ZQAAAAMBhqAAAAAUc3dhcFRva2Vuc0ZpeGVkSW5wdXQAAAAMV0VHTEQtYTI4YzU5AAAAAQEAAAABxkCGRxKnJuZDLZ7gYMXvsbQ2YQkSi+cxpOiXntSJ3lU=",
            "additionalData": [
              "AAAACAYAAAAAAAAAAAUAWBNyFLDhTClIYKFsEQQqpxq8FyB86wAAAAAAAAAMRVNEVFRyYW5zZmVyAAAABQAAAAtVU0RDLTM1MGM0ZQAAAAMBhqAAAAAUc3dhcFRva2Vuc0ZpeGVkSW5wdXQAAAAMV0VHTEQtYTI4YzU5AAAAAQEAAAABxkCGRxKnJuZDLZ7gYMXvsbQ2YQkSi+cxpOiXntSJ3lU="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "performAction",
            "topics": [
              "cGVyZm9ybUFzeW5jQ2FsbA==",
              "CA==",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=",
              "",
              "I3f2aQ==",
              "RVNEVFRyYW5zZmVy",
              "VVNEQy0zNTBjNGU=",
              "AYag",
              "c3dhcFRva2Vuc0ZpeGVkSW5wdXQ=",
              "V0VHTEQtYTI4YzU5",
              "AQ=="
            ],
            "data": null,
            "additionalData": [
              ""
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "ESDTTransfer",
            "topics": [
              "VVNEQy0zNTBjNGU=",
              "",
              "AYag",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs="
            ],
            "data": "QXN5bmNDYWxs",
            "additionalData": [
              "QXN5bmNDYWxs",
              "RVNEVFRyYW5zZmVy",
              "VVNEQy0zNTBjNGU=",
              "AYag",
              "c3dhcFRva2Vuc0ZpeGVkSW5wdXQ=",
              "V0VHTEQtYTI4YzU5",
              "AQ=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "ESDTTransfer",
            "topics": [
              "V0VHTEQtYTI4YzU5",
              "",
              "A6kOxETj",
              "AAAAAAAAAAAFABOe165KoDeS5ryzMjlKQP50bu+kfOs="
            ],
            "data": "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
            "additionalData": [
              "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
              "RVNEVFRyYW5zZmVy",
              "V0VHTEQtYTI4YzU5",
              "A6kOxETj",
              "c3dhcE5vRmVlQW5kRm9yd2FyZA==",
              "TUVYLWE2NTlkMA==",
              "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g",
            "identifier": "ESDTLocalBurn",
            "topics": [
              "TUVYLWE2NTlkMA==",
              "",
              "O4GGs5oYGqc="
            ],
            "data": null,
            "additionalData": null
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g",
            "identifier": "swapNoFeeAndForward",
            "topics": [
              "c3dhcF9ub19mZWVfYW5kX2ZvcndhcmQ=",
              "TUVYLWE2NTlkMA==",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=",
              "DLk="
            ],
            "data": "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAAMV0VHTEQtYTI4YzU5AAAABgOpDsRE4wAAAApNRVgtYTY1OWQwAAAACDuBhrOaGBqnAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHcqQwAAAAAAAAy5AAAAAGfJ1yY=",
            "additionalData": [
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAAMV0VHTEQtYTI4YzU5AAAABgOpDsRE4wAAAApNRVgtYTY1OWQwAAAACDuBhrOaGBqnAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHcqQwAAAAAAAAy5AAAAAGfJ1yY="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "ESDTTransfer",
            "topics": [
              "V0VHTEQtYTI4YzU5",
              "",
              "DkFmmWByvg==",
              "AAAAAAAAAAAFACr0ieZrWRU5IrwFF6xYfI+djGnS3lU="
            ],
            "data": "QmFja1RyYW5zZmVy",
            "additionalData": [
              "QmFja1RyYW5zZmVy",
              "RVNEVFRyYW5zZmVy",
              "V0VHTEQtYTI4YzU5",
              "DkFmmWByvg=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "swapTokensFixedInput",
            "topics": [
              "c3dhcA==",
              "VVNEQy0zNTBjNGU=",
              "V0VHTEQtYTI4YzU5",
              "AAAAAAAAAAAFACr0ieZrWRU5IrwFF6xYfI+djGnS3lU=",
              "DLk="
            ],
            "data": "AAAAAAAAAAAFACr0ieZrWRU5IrwFF6xYfI+djGnS3lUAAAALVVNEQy0zNTBjNGUAAAADAYagAAAADFdFR0xELWEyOGM1OQAAAAcOQWaZYHK+AAAAAWQAAAAF8hqqobsAAAAKCNyn2yd+OFFf8QAAAAAAdypDAAAAAAAADLkAAAAAZ8nXJg==",
            "additionalData": [
              "AAAAAAAAAAAFACr0ieZrWRU5IrwFF6xYfI+djGnS3lUAAAALVVNEQy0zNTBjNGUAAAADAYagAAAADFdFR0xELWEyOGM1OQAAAAcOQWaZYHK+AAAAAWQAAAAF8hqqobsAAAAKCNyn2yd+OFFf8QAAAAAAdypDAAAAAAAADLkAAAAAZ8nXJg=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "transferValueOnly",
            "topics": [
              "",
              "AAAAAAAAAAAFACr0ieZrWRU5IrwFF6xYfI+djGnS3lU="
            ],
            "data": "QXN5bmNDYWxsYmFjaw==",
            "additionalData": [
              "QXN5bmNDYWxsYmFjaw==",
              "Y2FsbEJhY2s=",
              "AA==",
              "AAAADFdFR0xELWEyOGM1OQAAAAAAAAAAAAAABw5BZplgcr4="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "callBack",
            "topics": [
              "YXN5bmNDYWxsU3VjY2Vzcw==",
              "AAAADFdFR0xELWEyOGM1OQAAAAAAAAAAAAAABw5BZplgcr4="
            ],
            "data": null,
            "additionalData": [
              ""
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "writeLog",
            "topics": [
              "xkCGRxKnJuZDLZ7gYMXvsbQ2YQkSi+cxpOiXntSJ3lU=",
              "QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk5OTI2MDAwLCBnYXMgdXNlZCA9IDI4MTU5NDEw"
            ],
            "data": "QDZmNmJAMDAwMDAwMGM1NzQ1NDc0YzQ0MmQ2MTMyMzg2MzM1MzkwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDcwZTQxNjY5OTYwNzJiZQ==",
            "additionalData": [
              "QDZmNmJAMDAwMDAwMGM1NzQ1NDc0YzQ0MmQ2MTMyMzg2MzM1MzkwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDcwZTQxNjY5OTYwNzJiZQ=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "completedTxEvent",
            "topics": [
              "E5fQszkBkYkpXmvT6w9C4X1Uy834+0QrBFqSAYO8czw="
            ],
            "data": null,
            "additionalData": null
          }
        ]
      },
      "status": "success",
      "operation": "transfer",
      "function": "performAction",
      "initiallyPaidFee": "6073260000000000",
      "fee": "6073260000000000",
      "chainID": "D",
      "version": 1,
      "options": 0
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        let tx_on_network = serde_json::from_str::<TransactionOnNetworkResponse>(data)
            .unwrap()
            .data
            .unwrap();

        let result = get_nested_multisig_call_result::<EsdtTokenPayment<StaticApi>>(
            "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            tx_on_network
        );

        let expected = Some(
            Payment {
                token_identifier: "WEGLD-a28c59".to_string(),
                token_nonce: 0,
                amount: BigUint::from_str("4012558589653694").unwrap(),
            }
        );

        assert_eq!(result, expected);
    }
}