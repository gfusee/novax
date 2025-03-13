use base64::Engine;
use multiversx_sc::imports::ManagedBuffer;
use multiversx_sc::types::CodeMetadata;
use multiversx_sc_codec::TopDecodeMulti;
use multiversx_sc_scenario::api::StaticApi;
use crate::generated::multisig::multisig::MultisigContract;
use crate::utils::map_novax_error_to_executor_error::map_novax_error_to_executor_error;
use novax_data::{Address, NativeConvertible};
use novax_executor::{ExecutorError, NormalizationInOut, TokenTransfer, TransactionError, TransactionExecutor, TransactionOnNetwork};
use num_bigint::BigUint;
use novax_executor::results::decode_topic;
use crate::executor::errors::{ERROR_WHILE_PROPOSING_ACTION_TRANSACTION, NONE_PROPOSAL_ID_ERROR};

#[derive(Clone)]
pub(crate) enum Action {
    AsyncCall(AsyncCallAction),
    TransferAndExecute(TransferAndExecuteAction),
    DeploySCFromSource(DeploySCFromSourceAction)
}

#[derive(Clone)]
pub(crate) struct AsyncCallAction {
    pub receiver: Address,
    pub function_name: Option<String>,
    pub arguments: Vec<Vec<u8>>,
    pub egld_value: BigUint,
    pub esdt_transfers: Vec<TokenTransfer>,
}

#[derive(Clone)]
pub(crate) struct TransferAndExecuteAction {
    pub receiver: Address,
    pub function_name: Option<String>,
    pub arguments: Vec<Vec<u8>>,
    pub egld_value: BigUint,
    pub esdt_transfers: Vec<TokenTransfer>,
}

#[derive(Clone)]
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
            },
            Action::TransferAndExecute(transfer_and_execute_action) => {
                propose_transfer_and_execute_action(
                    transfer_and_execute_action,
                    transaction_executor,
                    multisig_address,
                    gas_for_proposal,
                )
                    .await            }
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

    pub(crate) fn get_nested_multisig_call_result<OutputManaged>(
        &self,
        multisig_address: &str,
        tx: TransactionOnNetwork
    ) -> Result<Option<OutputManaged::Native>, ExecutorError>
    where
        OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        match self {
            Action::AsyncCall(_) => {
                get_nested_multisig_call_result_for_async_call_action::<OutputManaged>(
                    multisig_address,
                    tx
                )
            }
            Action::TransferAndExecute(_) => {
                todo!()
            }
            Action::DeploySCFromSource(_) => {
                panic!("Can't get nested multisig call result for deploy from source action.")
            }
        }
    }
}

pub(crate) async fn propose_async_call_action<TxExecutor>(
    async_call_action: AsyncCallAction,
    transaction_executor: TxExecutor,
    multisig_address: Address,
    gas_for_proposal: u64,
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

pub(crate) async fn propose_transfer_and_execute_action<TxExecutor>(
    transfer_and_execute_action: TransferAndExecuteAction,
    transaction_executor: TxExecutor,
    multisig_address: Address,
    gas_for_proposal: u64,
) -> Result<u32, ExecutorError>
where
    TxExecutor: TransactionExecutor + Clone,
{
    let to_bech32_string = transfer_and_execute_action.receiver.to_bech32_string()?;

    let normalized = NormalizationInOut {
        sender: to_bech32_string.clone(),
        receiver: to_bech32_string,
        function_name: transfer_and_execute_action.function_name,
        arguments: transfer_and_execute_action.arguments,
        egld_value: transfer_and_execute_action.egld_value,
        esdt_transfers: transfer_and_execute_action.esdt_transfers,
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
        .propose_transfer_execute(
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

fn get_nested_multisig_call_result_for_async_call_action<OutputManaged>(
    multisig_address: &str,
    tx: TransactionOnNetwork
) -> Result<Option<OutputManaged::Native>, ExecutorError>
where
    OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
{
    let async_call_event_identifier = "callBack";
    let async_call_success_topic_identifier = "asyncCallSuccess";
    let async_call_error_topic_identifier = "asyncCallError";

    let Some(log_of_interest) = tx
        .transaction
        .logs
        .clone()
        .into_iter()
        .find(|log| log.address == multisig_address)
    else {
        return Ok(None);
    };

    let Some(async_call_event) = log_of_interest
        .events
        .into_iter()
        .find(|event| {
            return event.address == multisig_address
                && event.identifier == async_call_event_identifier
        })
    else {
        return Ok(None);
    };

    let Some(async_call_event_first_topics_raw) = async_call_event.topics.get(0) else {
        return Ok(None);
    };

    let async_call_event_first_topics_decoded = decode_topic(async_call_event_first_topics_raw)?;

    if async_call_event_first_topics_decoded == async_call_success_topic_identifier {
        if async_call_event.topics.len() < 2 {
            return Ok(None);
        }

        let mut encoded_data = async_call_event
            .topics[1..]
            .to_vec()
            .into_iter()
            .map(|topic| base64::engine::general_purpose::STANDARD.decode(topic).expect("Decoding failed"))
            .collect::<Vec<Vec<u8>>>();

        let Ok(decoded) = OutputManaged::multi_decode(&mut encoded_data) else {
            return Ok(None);
        };

        Ok(Some(decoded.to_native()))
    } else if async_call_event_first_topics_decoded == async_call_error_topic_identifier {
        if async_call_event.topics.len() != 3 {
            return Ok(None);
        }

        let encoded_data = async_call_event
            .topics[1..]
            .to_vec()
            .into_iter()
            .map(|topic| base64::engine::general_purpose::STANDARD.decode(topic).expect("Decoding failed"))
            .collect::<Vec<Vec<u8>>>();

        let Ok(err_code) = u32::multi_decode(&mut encoded_data[0..1].to_vec()) else {
            return Err(TransactionError::CannotDecodeSmartContractResult { response: tx }.into())
        };

        let Ok(err_message) = ManagedBuffer::<StaticApi>::multi_decode(&mut encoded_data[1..2].to_vec()) else {
            return Err(TransactionError::CannotDecodeSmartContractResult { response: tx }.into())
        };

        let utf8_error_message_or_unknown = String::from_utf8(err_message.to_boxed_bytes().into_vec())
            .unwrap_or("unknown non-utf8 error".to_string());

        return Err(
            ExecutorError::Transaction(
                TransactionError::SmartContractExecutionError {
                    status: err_code as u64,
                    message: utf8_error_message_or_unknown
                }
            )
        )
    } else {
        Ok(None)
    }
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

#[cfg(test)]
mod tests {
    use multiversx_sc::types::{EsdtTokenPayment, TokenIdentifier};
    use multiversx_sc_scenario::api::StaticApi;
    use novax_data::Payment;
    use novax_executor::{ExecutorError, TransactionError, TransactionOnNetworkResponse};
    use num_bigint::BigUint;
    use std::str::FromStr;
    use crate::executor::action::get_nested_multisig_call_result_for_async_call_action;

    #[tokio::test]
    async fn test_get_nested_multisig_call_result_async_call_action_with_simple_xexchange_view() {
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

        let result = get_nested_multisig_call_result_for_async_call_action::<TokenIdentifier<StaticApi>>(
            "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            tx_on_network
        );

        let expected = Ok(Some("WEGLD-a28c59".to_string()));

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_nested_multisig_call_result_async_call_action_with_xexchange_swap() {
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

        let result = get_nested_multisig_call_result_for_async_call_action::<EsdtTokenPayment<StaticApi>>(
            "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            tx_on_network
        );

        let expected = Ok(
            Some(
                Payment {
                    token_identifier: "WEGLD-a28c59".to_string(),
                    token_nonce: 0,
                    amount: BigUint::from_str("4012558589653694").unwrap(),
                }
            )
        );

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_nested_multisig_call_result_async_call_action_with_nested_call_error() {
        let data = r#"
        {
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "SCInvoking",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "ffd2cc4d50a3630573f4cc006103d412a347ff273daa010233acef472238e498",
      "nonce": 41,
      "round": 7956942,
      "epoch": 3289,
      "value": "0",
      "receiver": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
      "sender": "erd1ceqgv3cj5unwvsednmsxp300kx6rvcgfz297wvdyaztea4yfme2s9ncxue",
      "gasPrice": 1000000000,
      "gasLimit": 600000000,
      "gasUsed": 600000000,
      "data": "cGVyZm9ybUFjdGlvbkAxMw==",
      "signature": "e6dab6d2e0d223d77dae2d8033935e28fca04ae5145a782344e13accb79b491eedbe0f80bb92c0aa744b1583803ece231ee4f546378e355a68e19a6f00b0b80a",
      "sourceShard": 1,
      "destinationShard": 1,
      "blockNonce": 7886356,
      "blockHash": "759f11e8f32fc9fb1682d3e11f4c6907c9f2685086a7371bda65f41ae2f56eba",
      "notarizedAtSourceInMetaNonce": 7889845,
      "NotarizedAtSourceInMetaHash": "86202a86a146ef7214584ad7c26ee920bb2039bc7968c0afc4461db2317fb9bd",
      "notarizedAtDestinationInMetaNonce": 7889845,
      "notarizedAtDestinationInMetaHash": "86202a86a146ef7214584ad7c26ee920bb2039bc7968c0afc4461db2317fb9bd",
      "miniblockType": "TxBlock",
      "miniblockHash": "5a7e3aac91bfb42002b5ef6dbb38035865537bb6aa6f7ec9683eb8d763e27f36",
      "hyperblockNonce": 7889845,
      "hyperblockHash": "86202a86a146ef7214584ad7c26ee920bb2039bc7968c0afc4461db2317fb9bd",
      "timestamp": 1741741652,
      "logs": {
        "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
        "events": [
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "performAction",
            "topics": [
              "c3RhcnRQZXJmb3JtQWN0aW9u"
            ],
            "data": "AAAAEwYAAAAAAAAAAAUAxcJjS0t7+CZsCTCWfKEYUye9U6/eVQAAAAexorwuxQAAAAAAIGRlcGxveUhhdG9tU2ltcGxlTGVuZGluZ1N0cmF0ZWd5AAAACwAAAAtVU0RDLTM1MGM0ZQAAAAAAAAAAAAAAFEF1dG9zY2FsZVVTRENMZW5kaW5nAAAABkFWVVNEQwAAAAgN4Lazp2QAAAAAACAAAAAAAAAAAAUAeLkXiNESxMjRgpXqj6b3BDGnvgFNHQAAAAFkAAAAAgu4AAAAjAAAAAtVU0RDLTM1MGM0ZQAAAAAAAAAKSFRNLTIzYTFkYQAAAAIAAAAAAAAAAAAAAAAFAJhq5J5wa8KRFp5OVTOgQbQJg2PFfOsAAAAMV0VHTEQtYTI4YzU5AAAAAQAAAAAAAAAABQBYE3IUsOFMKUhgoWwRBCqnGrwXIHzrAAAAC1VTREMtMzUwYzRlAAAAEwAAAAtVU0RDLTM1MGM0ZQAAAAAAAAABxkCGRxKnJuZDLZ7gYMXvsbQ2YQkSi+cxpOiXntSJ3lU=",
            "additionalData": [
              "AAAAEwYAAAAAAAAAAAUAxcJjS0t7+CZsCTCWfKEYUye9U6/eVQAAAAexorwuxQAAAAAAIGRlcGxveUhhdG9tU2ltcGxlTGVuZGluZ1N0cmF0ZWd5AAAACwAAAAtVU0RDLTM1MGM0ZQAAAAAAAAAAAAAAFEF1dG9zY2FsZVVTRENMZW5kaW5nAAAABkFWVVNEQwAAAAgN4Lazp2QAAAAAACAAAAAAAAAAAAUAeLkXiNESxMjRgpXqj6b3BDGnvgFNHQAAAAFkAAAAAgu4AAAAjAAAAAtVU0RDLTM1MGM0ZQAAAAAAAAAKSFRNLTIzYTFkYQAAAAIAAAAAAAAAAAAAAAAFAJhq5J5wa8KRFp5OVTOgQbQJg2PFfOsAAAAMV0VHTEQtYTI4YzU5AAAAAQAAAAAAAAAABQBYE3IUsOFMKUhgoWwRBCqnGrwXIHzrAAAAC1VTREMtMzUwYzRlAAAAEwAAAAtVU0RDLTM1MGM0ZQAAAAAAAAABxkCGRxKnJuZDLZ7gYMXvsbQ2YQkSi+cxpOiXntSJ3lU="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "performAction",
            "topics": [
              "cGVyZm9ybUFzeW5jQ2FsbA==",
              "Ew==",
              "AAAAAAAAAAAFAMXCY0tLe/gmbAkwlnyhGFMnvVOv3lU=",
              "saK8LsUAAA==",
              "I3ASdw==",
              "ZGVwbG95SGF0b21TaW1wbGVMZW5kaW5nU3RyYXRlZ3k=",
              "VVNEQy0zNTBjNGU=",
              "",
              "",
              "QXV0b3NjYWxlVVNEQ0xlbmRpbmc=",
              "QVZVU0RD",
              "DeC2s6dkAAA=",
              "AAAAAAAAAAAFAHi5F4jREsTI0YKV6o+m9wQxp74BTR0=",
              "ZA==",
              "C7g=",
              "AAAAC1VTREMtMzUwYzRlAAAAAAAAAApIVE0tMjNhMWRhAAAAAgAAAAAAAAAAAAAAAAUAmGrknnBrwpEWnk5VM6BBtAmDY8V86wAAAAxXRUdMRC1hMjhjNTkAAAABAAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAALVVNEQy0zNTBjNGU=",
              "AAAAC1VTREMtMzUwYzRlAAAAAA=="
            ],
            "data": null,
            "additionalData": [
              ""
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqchpxxj6t00uzvmqfxzt8eggc2vnm65a0me2s6zwsrr",
            "identifier": "transferValueOnly",
            "topics": [
              "",
              "AAAAAAAAAAAFACr0ieZrWRU5IrwFF6xYfI+djGnS3lU="
            ],
            "data": "QXN5bmNDYWxsYmFjaw==",
            "additionalData": [
              "QXN5bmNDYWxsYmFjaw==",
              "Y2FsbEJhY2s=",
              "Bw==",
              "ZmFpbGVkIHRyYW5zZmVyIChpbnN1ZmZpY2llbnQgZnVuZHMp"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "callBack",
            "topics": [
              "YXN5bmNDYWxsRXJyb3I=",
              "Bw==",
              "ZmFpbGVkIHRyYW5zZmVyIChpbnN1ZmZpY2llbnQgZnVuZHMp"
            ],
            "data": null,
            "additionalData": [
              ""
            ]
          },
          {
            "address": "erd1ceqgv3cj5unwvsednmsxp300kx6rvcgfz297wvdyaztea4yfme2s9ncxue",
            "identifier": "internalVMErrors",
            "topics": [
              "AAAAAAAAAAAFACr0ieZrWRU5IrwFF6xYfI+djGnS3lU=",
              "cGVyZm9ybUFjdGlvbg=="
            ],
            "data": "CglydW50aW1lLmdvOjg1NiBbZmFpbGVkIHRyYW5zZmVyIChpbnN1ZmZpY2llbnQgZnVuZHMpXSBbZGVwbG95SGF0b21TaW1wbGVMZW5kaW5nU3RyYXRlZ3ldCglydW50aW1lLmdvOjg1MyBbZmFpbGVkIHRyYW5zZmVyIChpbnN1ZmZpY2llbnQgZnVuZHMpXSBbZGVwbG95SGF0b21TaW1wbGVMZW5kaW5nU3RyYXRlZ3ld",
            "additionalData": [
              "CglydW50aW1lLmdvOjg1NiBbZmFpbGVkIHRyYW5zZmVyIChpbnN1ZmZpY2llbnQgZnVuZHMpXSBbZGVwbG95SGF0b21TaW1wbGVMZW5kaW5nU3RyYXRlZ3ldCglydW50aW1lLmdvOjg1MyBbZmFpbGVkIHRyYW5zZmVyIChpbnN1ZmZpY2llbnQgZnVuZHMpXSBbZGVwbG95SGF0b21TaW1wbGVMZW5kaW5nU3RyYXRlZ3ld"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "writeLog",
            "topics": [
              "xkCGRxKnJuZDLZ7gYMXvsbQ2YQkSi+cxpOiXntSJ3lU=",
              "QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk5OTI2MDAwLCBnYXMgdXNlZCA9IDk4OTE5OTE="
            ],
            "data": "QDZmNmI=",
            "additionalData": [
              "QDZmNmI="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            "identifier": "completedTxEvent",
            "topics": [
              "/9LMTVCjYwVz9MwAYQPUEqNH/yc9qgECM6zvRyI45Jg="
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

        let result = get_nested_multisig_call_result_for_async_call_action::<EsdtTokenPayment<StaticApi>>(
            "erd1qqqqqqqqqqqqqpgq9t6gnentty2njg4uq5t6ckru37wcc6wjme2shfz4k7",
            tx_on_network
        );

        let expected = Err(
            ExecutorError::Transaction(
                TransactionError::SmartContractExecutionError {
                    status: 7,
                    message: "failed transfer (insufficient funds)".to_string(),
                }
            )
        );

        assert_eq!(result, expected);
    }
}