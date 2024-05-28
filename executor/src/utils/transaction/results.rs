use crate::{ExecutorError, TransactionOnNetworkTransactionLogsEvents, TransactionOnNetworkTransactionSmartContractResult};
use crate::error::transaction::TransactionError;

pub(crate) fn find_sc_deploy_event(logs: &[TransactionOnNetworkTransactionLogsEvents]) -> Option<TransactionOnNetworkTransactionLogsEvents> {
    logs.iter()
        .find(|event| event.identifier == "SCDeploy")
        .cloned()
}

pub(crate) fn find_smart_contract_result(opt_sc_results: &Option<Vec<TransactionOnNetworkTransactionSmartContractResult>>) -> Result<Option<Vec<Vec<u8>>>, ExecutorError> {
    let Some(sc_results) = opt_sc_results else {
        return Ok(None)
    };

    let scr_found_result = sc_results.iter()
        .find(|sc_result| sc_result.nonce != 0 && sc_result.data.starts_with('@'))
        .cloned();

    let data = if let Some(scr) = scr_found_result {
        let mut split = scr.data.split('@');
        if split.next().is_none() {
            return Err(TransactionError::CannotDecodeSmartContractResult.into())
        }

        let Some(result_code) = split.next() else {
            return Err(TransactionError::CannotDecodeSmartContractResult.into())
        };

        if result_code != "6f6b" {
            return Err(TransactionError::CannotDecodeSmartContractResult.into())
        }

        let data = split
            .map(|encoded_arg| hex::decode(encoded_arg).expect("error hex-decoding result"))
            .collect();

        Some(data)
    } else {
        None
    };

    Ok(data)
}