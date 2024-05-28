use crate::{TransactionOnNetworkTransactionLogsEvents, TransactionOnNetworkTransactionSmartContractResult};

pub(crate) fn find_sc_deploy_event(logs: &[TransactionOnNetworkTransactionLogsEvents]) -> Option<TransactionOnNetworkTransactionLogsEvents> {
    logs.iter()
        .find(|event| event.identifier == "SCDeploy")
        .cloned()
}

pub(crate) fn find_smart_contract_result(opt_sc_results: &Option<Vec<TransactionOnNetworkTransactionSmartContractResult>>) -> Option<Vec<Vec<u8>>> {
    let Some(sc_results) = opt_sc_results else {
        return None
    };

    sc_results.iter()
        .find(|sc_result| sc_result.nonce != 0 && sc_result.data.starts_with('@'))
        .cloned()
        .map(|sc_result| {
            let mut split = sc_result.data.split('@');
            let _ = split.next().expect("SCR data should start with '@'"); // TODO: no expect and assert_eq!
            let result_code = split.next().expect("missing result code");
            assert_eq!(result_code, "6f6b", "result code is not 'ok'");

            split
                .map(|encoded_arg| hex::decode(encoded_arg).expect("error hex-decoding result"))
                .collect()
        })
}