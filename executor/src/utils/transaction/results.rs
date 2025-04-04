use base64::Engine;
use multiversx_sc_snippets::network_response::decode_scr_data_or_panic;
use multiversx_sdk::utils::base64_decode;

use crate::{ExecutorError, TransactionOnNetwork, TransactionOnNetworkTransactionLogs, TransactionOnNetworkTransactionLogsEvents, TransactionOnNetworkTransactionSmartContractResult};
use crate::error::transaction::TransactionError;

const ERROR_SIGNALLED_BY_SMART_CONTRACT: &str = "error signalled by smartcontract";

#[derive(Clone, Debug)]
pub(crate) struct SmartContractError {
    pub status: u64,
    pub message: String
}

pub(crate) fn find_sc_deploy_event(logs: &[TransactionOnNetworkTransactionLogsEvents]) -> Option<TransactionOnNetworkTransactionLogsEvents> {
    logs.iter()
        .find(|event| event.identifier == "SCDeploy")
        .cloned()
}

pub(crate) fn find_smart_contract_result(
    tx_on_network: &TransactionOnNetwork
) -> Result<Option<Vec<Vec<u8>>>, ExecutorError> {
    let mut result = if let Some(sc_results) = tx_on_network.transaction.smart_contract_results.as_ref() {
        find_smart_contract_result_from_regular_sc_results(
            tx_on_network,
            &sc_results
        )?
    } else {
        None
    };

    if result.is_none() {
        if let Some(logs) = tx_on_network.transaction.logs.as_ref() {
            result = find_smart_contract_result_from_logs(logs)?;
        }
    }

    Ok(result)
}

pub(crate) fn find_sc_error(logs: &TransactionOnNetworkTransactionLogs) -> Result<Option<SmartContractError>, ExecutorError> {
    let opt_signal_error_event = logs.events
        .iter()
        .find(|log| log.identifier == "signalError");

    if let Some(signal_error_event) = opt_signal_error_event {
        let topics = &signal_error_event.topics;

        if topics.len() != 2 {
            return Err(TransactionError::WrongTopicsCountForSignalErrorEvent.into())
        }

        let error = decode_topic(topics.get(1).unwrap())?;
        let status = if error.contains(ERROR_SIGNALLED_BY_SMART_CONTRACT) {
            10
        } else {
            4
        };

        let result = SmartContractError {
            status,
            message: error,
        };
        return Ok(Some(result));
    }

    Ok(None)
}

fn find_smart_contract_result_from_regular_sc_results(
    tx_on_network: &TransactionOnNetwork,
    sc_results: &[TransactionOnNetworkTransactionSmartContractResult]
) -> Result<Option<Vec<Vec<u8>>>, ExecutorError> {
    let scr_found_result = sc_results.iter()
        .find(|sc_result| sc_result.nonce != 0 && sc_result.data.starts_with('@'))
        .cloned();

    let data = if let Some(scr) = scr_found_result {
        let mut split = scr.data.split('@');
        if split.next().is_none() {
            return Err(TransactionError::CannotDecodeSmartContractResult { response: tx_on_network.clone() }.into())
        }

        let Some(result_code) = split.next() else {
            return Err(TransactionError::CannotDecodeSmartContractResult { response: tx_on_network.clone() }.into())
        };

        if result_code != "6f6b" {
            return Err(TransactionError::CannotDecodeSmartContractResult { response: tx_on_network.clone() }.into())
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

pub fn find_smart_contract_result_from_logs(
    logs: &TransactionOnNetworkTransactionLogs
) -> Result<Option<Vec<Vec<u8>>>, ExecutorError> {
    let find_result = logs.events
        .iter()
        .rev()
        .find_map(|event| {
            if event.identifier == "writeLog" {
                if let Some(data) = &event.data {
                    let decoded_data = String::from_utf8(base64_decode(data)).unwrap();

                    if decoded_data.starts_with('@') {
                        let out = decode_scr_data_or_panic(decoded_data.as_str());
                        return Some(out);
                    }
                }
            }

            None
    });

    Ok(find_result)
}

pub fn decode_topic(topic: &str) -> Result<String, ExecutorError> {
    let decoded = base64::engine::general_purpose::STANDARD.decode(topic)
        .map_err(|_| TransactionError::CannotDecodeTopic)?;

    String::from_utf8(decoded)
        .map_err(|_| TransactionError::CannotDecodeTopic.into())
}

#[cfg(test)]
mod tests {
    use crate::TransactionOnNetworkResponse;
    use crate::utils::transaction::results::find_smart_contract_result;

    #[test]
    fn test_with_multi_contract_same_shard_tx_that_has_no_sc_result() {
        // transaction data from the devnet
        // context : user -> A --call--> B, B returns a MultiValue2<u64, u64>, A returns the B's returned value
        let data = r#"
            {
              "data": {
                "transaction": {
                  "type": "normal",
                  "processingTypeOnSource": "SCInvoking",
                  "processingTypeOnDestination": "SCInvoking",
                  "hash": "e914857f1bfd003ba411bae372266703e5f706fa412c378feb37faa5e18c3d73",
                  "nonce": 49,
                  "round": 7646960,
                  "epoch": 6339,
                  "value": "0",
                  "receiver": "erd1qqqqqqqqqqqqqpgqshqmekudxlxwp0d9j368etjamr5dw7k45u7qx40w6h",
                  "sender": "erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g",
                  "gasPrice": 1000000000,
                  "gasLimit": 600000000,
                  "gasUsed": 600000000,
                  "data": "Y2FsbEFub3RoZXJDb250cmFjdFJldHVyblR3b1U2NEAwMDAwMDAwMDAwMDAwMDAwMDUwMEFDRkY2QjdBNEVCODEwMUE4REU3RkY3RjVEMkMwQkYzRTRENjNGNDdBNzND",
                  "signature": "53cc6496647287d735bd7950f4ec79d7b51f884defda1d6d840d722b7d0d869900ccecc01602da7a7c717955e8d4ed0711b92acd980d64ed6eebd6eaed0c4608",
                  "sourceShard": 0,
                  "destinationShard": 0,
                  "blockNonce": 7600794,
                  "blockHash": "77eb0904e56d6dd596c0d72821cf33b326fde383e72903ca4df5c2f200b0ea75",
                  "notarizedAtSourceInMetaNonce": 7609344,
                  "NotarizedAtSourceInMetaHash": "12df3fe65cacde2c9742b9506ac2261d34f3c72d690301192fd8016430d51913",
                  "notarizedAtDestinationInMetaNonce": 7609344,
                  "notarizedAtDestinationInMetaHash": "12df3fe65cacde2c9742b9506ac2261d34f3c72d690301192fd8016430d51913",
                  "miniblockType": "TxBlock",
                  "miniblockHash": "03219ac7427f7511687f0768c722c759c1b1428b2664b44a0cbe2072154851ee",
                  "hyperblockNonce": 7609344,
                  "hyperblockHash": "12df3fe65cacde2c9742b9506ac2261d34f3c72d690301192fd8016430d51913",
                  "timestamp": 1694433360,
                  "logs": {
                    "address": "erd1qqqqqqqqqqqqqpgqshqmekudxlxwp0d9j368etjamr5dw7k45u7qx40w6h",
                    "events": [
                      {
                        "address": "erd1qqqqqqqqqqqqqpgqshqmekudxlxwp0d9j368etjamr5dw7k45u7qx40w6h",
                        "identifier": "writeLog",
                        "topics": [
                          "5fXsK/a5JVZf0e2Z6ViFglDOQP1zsS1XkuaLvaZ5pzw=",
                          "QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk5ODA2MDAwLCBnYXMgdXNlZCA9IDM4NDcyNDA="
                        ],
                        "data": "QDZmNmJAMGFAMDIxODcxMWEwMA=="
                      },
                      {
                        "address": "erd1qqqqqqqqqqqqqpgqshqmekudxlxwp0d9j368etjamr5dw7k45u7qx40w6h",
                        "identifier": "completedTxEvent",
                        "topics": [
                          "6RSFfxv9ADukEbrjciZnA+X3BvpBLDeP6zf6peGMPXM="
                        ],
                        "data": null
                      }
                    ]
                  },
                  "status": "success",
                  "operation": "transfer",
                  "function": "callAnotherContractReturnTwoU64",
                  "initiallyPaidFee": "6192060000000000",
                  "fee": "6192060000000000",
                  "chainID": "D",
                  "version": 2,
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

        let results = find_smart_contract_result(
            &tx_on_network
        )
            .unwrap()
            .unwrap();

        let expected: Vec<Vec<u8>> = vec![
            hex::decode("0a").unwrap(),
            hex::decode("0218711a00").unwrap(),
        ];

        assert_eq!(results, expected)
    }

    #[test]
    fn test_with_multi_contract_cross_shard_tx_that_has_no_callback() {
        // transaction data from the devnet
        // context : user -> A --async call--> B, no callback
        let data = r#"
            {
              "data": {
                "transaction": {
                  "type": "normal",
                  "processingTypeOnSource": "SCInvoking",
                  "processingTypeOnDestination": "SCInvoking",
                  "hash": "4d50a055663dfee2479851684d7fb83cf00695b6f03f4dbbdf0f9232477cafc4",
                  "nonce": 51,
                  "round": 7647523,
                  "epoch": 6340,
                  "value": "0",
                  "receiver": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                  "sender": "erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g",
                  "gasPrice": 1000000000,
                  "gasLimit": 600000000,
                  "gasUsed": 600000000,
                  "data": "YXN5bmNDYWxsQW5vdGhlckNvbnRyYWN0UmV0dXJuVHdvVTY0Tm9DYWxsYmFja0AwMDAwMDAwMDAwMDAwMDAwMDUwMEFDRkY2QjdBNEVCODEwMUE4REU3RkY3RjVEMkMwQkYzRTRENjNGNDdBNzND",
                  "signature": "0fc30cddaa8e5365662a14344e3434cbccf287f357be99b3ed4add182f64dded774ec0d095ab1589e7c6c07e00de3b7239efc96eb2e0e97b48c1ef87084cec01",
                  "sourceShard": 0,
                  "destinationShard": 1,
                  "blockNonce": 7593758,
                  "blockHash": "a828c0ca58ef1c8aff60e512ab59f18204f1915d4a6c8285cfceb1c5725b88e8",
                  "notarizedAtSourceInMetaNonce": 7609903,
                  "NotarizedAtSourceInMetaHash": "4e90fe45c2fdccd5cf6977c1422e5f4ffa41c4e9f31fb4a50c20455f87df1e99",
                  "notarizedAtDestinationInMetaNonce": 7609907,
                  "notarizedAtDestinationInMetaHash": "10b8666a44411c3babbe20af7154fb3d47efcb1cb10d955523ec68fece26e517",
                  "miniblockType": "TxBlock",
                  "miniblockHash": "4ff4bb1ac88911d617c9b0342aeb5158db78490c2fe414cad08adcc584a77be7",
                  "hyperblockNonce": 7609907,
                  "hyperblockHash": "10b8666a44411c3babbe20af7154fb3d47efcb1cb10d955523ec68fece26e517",
                  "timestamp": 1694436738,
                  "smartContractResults": [
                    {
                      "hash": "462b56a1530e6070dc7c15f755e51a97a6972c8cd7891f3be4635b93211890c5",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                      "sender": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                      "data": "@00@0a@0218711a00",
                      "prevTxHash": "41d56fdacf3e14de67e821427c732b62ebfa07c82d2e5db6de75fe3a1c828d9b",
                      "originalTxHash": "4d50a055663dfee2479851684d7fb83cf00695b6f03f4dbbdf0f9232477cafc4",
                      "gasLimit": 595637825,
                      "gasPrice": 1000000000,
                      "callType": 2,
                      "logs": {
                        "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                        "events": [
                          {
                            "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                            "identifier": "writeLog",
                            "topics": [
                              "AAAAAAAAAAAFAP/Aj4ZNGKlpx2+xeJLdoJbREzb20P0=",
                              "QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk1NjM3ODI1LCBnYXMgdXNlZCA9IDIxNjE3NzA="
                            ],
                            "data": "QDZmNmI="
                          },
                          {
                            "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                            "identifier": "completedTxEvent",
                            "topics": [
                              "QdVv2s8+FN5n6CFCfHMrYuv6B8gtLl223nX+OhyCjZs="
                            ],
                            "data": null
                          }
                        ]
                      },
                      "operation": "transfer"
                    },
                    {
                      "hash": "41d56fdacf3e14de67e821427c732b62ebfa07c82d2e5db6de75fe3a1c828d9b",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                      "sender": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                      "data": "returnTwoU64@4f3c60",
                      "prevTxHash": "4d50a055663dfee2479851684d7fb83cf00695b6f03f4dbbdf0f9232477cafc4",
                      "originalTxHash": "4d50a055663dfee2479851684d7fb83cf00695b6f03f4dbbdf0f9232477cafc4",
                      "gasLimit": 597479490,
                      "gasPrice": 1000000000,
                      "callType": 1,
                      "operation": "transfer",
                      "function": "returnTwoU64"
                    }
                  ],
                  "logs": {
                    "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                    "events": [
                      {
                        "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                        "identifier": "writeLog",
                        "topics": [
                          "5fXsK/a5JVZf0e2Z6ViFglDOQP1zsS1XkuaLvaZ5pzw="
                        ],
                        "data": "QDZmNmI="
                      }
                    ]
                  },
                  "status": "success",
                  "operation": "transfer",
                  "function": "asyncCallAnotherContractReturnTwoU64NoCallback",
                  "initiallyPaidFee": "6214335000000000",
                  "fee": "6214335000000000",
                  "chainID": "D",
                  "version": 2,
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

        let results = find_smart_contract_result(
            &tx_on_network
        )
            .unwrap()
            .unwrap();

        let expected: Vec<Vec<u8>> = vec![];

        assert_eq!(results, expected)
    }

    #[test]
    fn test_with_multi_contract_cross_shard_tx_that_has_non_returning_callback() {
        // transaction data from the devnet
        // context : user -> A --async call--> B --callback--> A, the callback returns ()
        let data = r#"
            {
              "data": {
                "transaction": {
                  "type": "normal",
                  "processingTypeOnSource": "SCInvoking",
                  "processingTypeOnDestination": "SCInvoking",
                  "hash": "4f7f19e448176e4d47a0f844cbd6bdb1b6c68035dafe927e8249ed60af1c3b17",
                  "nonce": 52,
                  "round": 7647560,
                  "epoch": 6340,
                  "value": "0",
                  "receiver": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                  "sender": "erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g",
                  "gasPrice": 1000000000,
                  "gasLimit": 600000000,
                  "gasUsed": 600000000,
                  "data": "YXN5bmNDYWxsQW5vdGhlckNvbnRyYWN0UmV0dXJuVHdvVTY0V2l0aE5vblJldHVybmluZ0NhbGxiYWNrQDAwMDAwMDAwMDAwMDAwMDAwNTAwQUNGRjZCN0E0RUI4MTAxQThERTdGRjdGNUQyQzBCRjNFNEQ2M0Y0N0E3M0M=",
                  "signature": "3918fce429b2059b2321b709011079755dbb835e12839278ee510e4741180540e80c6111eea1d3312b2c63446de08b20e01f6040358fa94d1633c355bb65bc0f",
                  "sourceShard": 0,
                  "destinationShard": 1,
                  "blockNonce": 7593795,
                  "blockHash": "c17e727f90025225670b7852ea9807c67753c9b3f21b6ec7cc40077e3849a8b7",
                  "notarizedAtSourceInMetaNonce": 7609940,
                  "NotarizedAtSourceInMetaHash": "c67b5c550986cfd6c94d00f4b90234eb38ee196ff0d79a00d916f3bd24be272c",
                  "notarizedAtDestinationInMetaNonce": 7609944,
                  "notarizedAtDestinationInMetaHash": "d59b7398d962ce3119679af59d5d74e10083e62c3ee2b15421cc0d607979ca18",
                  "miniblockType": "TxBlock",
                  "miniblockHash": "2977affeffeb6cf41117bed442662021cb713528cdb1d0dce4537b01caeb8e0b",
                  "hyperblockNonce": 7609944,
                  "hyperblockHash": "d59b7398d962ce3119679af59d5d74e10083e62c3ee2b15421cc0d607979ca18",
                  "timestamp": 1694436960,
                  "smartContractResults": [
                    {
                      "hash": "fe7474188d5ca4b84c7577f03fc778d22d53c070dfcb05a9cda840229d30e4d3",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                      "sender": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                      "data": "returnTwoU64@4f3c60",
                      "prevTxHash": "4f7f19e448176e4d47a0f844cbd6bdb1b6c68035dafe927e8249ed60af1c3b17",
                      "originalTxHash": "4f7f19e448176e4d47a0f844cbd6bdb1b6c68035dafe927e8249ed60af1c3b17",
                      "gasLimit": 596979545,
                      "gasPrice": 1000000000,
                      "callType": 1,
                      "operation": "transfer",
                      "function": "returnTwoU64"
                    },
                    {
                      "hash": "948dc6702b376d1e043db8de2f87ca12907c342f54cfad7dfebadf59145ca3ac",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                      "sender": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                      "data": "@00@0a@0218711a00",
                      "prevTxHash": "fe7474188d5ca4b84c7577f03fc778d22d53c070dfcb05a9cda840229d30e4d3",
                      "originalTxHash": "4f7f19e448176e4d47a0f844cbd6bdb1b6c68035dafe927e8249ed60af1c3b17",
                      "gasLimit": 595137880,
                      "gasPrice": 1000000000,
                      "callType": 2,
                      "logs": {
                        "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                        "events": [
                          {
                            "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                            "identifier": "writeLog",
                            "topics": [
                              "AAAAAAAAAAAFAP/Aj4ZNGKlpx2+xeJLdoJbREzb20P0=",
                              "QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk1MTM3ODgwLCBnYXMgdXNlZCA9IDIyODg1NTA="
                            ],
                            "data": "QDZmNmJAMGFAMDIxODcxMWEwMA=="
                          },
                          {
                            "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                            "identifier": "completedTxEvent",
                            "topics": [
                              "/nR0GI1cpLhMdXfwP8d40i1TwHDfywWpzahAIp0w5NM="
                            ],
                            "data": null
                          }
                        ]
                      },
                      "operation": "transfer"
                    }
                  ],
                  "logs": {
                    "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                    "events": [
                      {
                        "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                        "identifier": "writeLog",
                        "topics": [
                          "5fXsK/a5JVZf0e2Z6ViFglDOQP1zsS1XkuaLvaZ5pzw="
                        ],
                        "data": "QDZmNmI="
                      }
                    ]
                  },
                  "status": "success",
                  "operation": "transfer",
                  "function": "asyncCallAnotherContractReturnTwoU64WithNonReturningCallback",
                  "initiallyPaidFee": "6235125000000000",
                  "fee": "6235125000000000",
                  "chainID": "D",
                  "version": 2,
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

        let results = find_smart_contract_result(
            &tx_on_network
        )
            .unwrap()
            .unwrap();

        let expected: Vec<Vec<u8>> = vec![];

        assert_eq!(results, expected)
    }

    #[test]
    fn test_with_multi_contract_cross_shard_tx_that_has_returning_callback() {
        // transaction data from the devnet
        // context : user -> A --async call--> B --callback--> A, the callback returns a MultiValue2<u64, u64>
        let data = r#"
            {
              "data": {
                "transaction": {
                  "type": "normal",
                  "processingTypeOnSource": "SCInvoking",
                  "processingTypeOnDestination": "SCInvoking",
                  "hash": "f34e136ca81c0e32f6fb532b753612715675073f3718b5db009bb275d246fd7a",
                  "nonce": 53,
                  "round": 7647583,
                  "epoch": 6340,
                  "value": "0",
                  "receiver": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                  "sender": "erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g",
                  "gasPrice": 1000000000,
                  "gasLimit": 600000000,
                  "gasUsed": 600000000,
                  "data": "YXN5bmNDYWxsQW5vdGhlckNvbnRyYWN0UmV0dXJuVHdvVTY0V2l0aFJldHVybmluZ0NhbGxiYWNrQDAwMDAwMDAwMDAwMDAwMDAwNTAwQUNGRjZCN0E0RUI4MTAxQThERTdGRjdGNUQyQzBCRjNFNEQ2M0Y0N0E3M0M=",
                  "signature": "858958d4aaf9cb0220ab2933edad3f65e1cb4c58aa7940cb0f40b489d0bd9fdf5c4736a40d6e813743ee622bb91e9f86eacf01b9a31e0ff53f9c84f13c500304",
                  "sourceShard": 0,
                  "destinationShard": 1,
                  "blockNonce": 7593818,
                  "blockHash": "b19f97110ca38d3cb15f802a00ab403491b0e5804ebc701527ab50064dc06825",
                  "notarizedAtSourceInMetaNonce": 7609963,
                  "NotarizedAtSourceInMetaHash": "4d9db6de610ca778114d44fe91dd036fac7c375c373ae9e77130d3fb9efc8391",
                  "notarizedAtDestinationInMetaNonce": 7609967,
                  "notarizedAtDestinationInMetaHash": "a4573d388c31860f9bd6f9507b65d1b3130e445abcada538f10704feba4614e7",
                  "miniblockType": "TxBlock",
                  "miniblockHash": "530f5fa3c7af474a187caca8dcea02a7a155017414147871d083bed5c49ec8f5",
                  "hyperblockNonce": 7609967,
                  "hyperblockHash": "a4573d388c31860f9bd6f9507b65d1b3130e445abcada538f10704feba4614e7",
                  "timestamp": 1694437098,
                  "smartContractResults": [
                    {
                      "hash": "065291164a8acd27c26b5a8f09664810081fda18cd54fca635196cf9b200297a",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                      "sender": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                      "data": "returnTwoU64@4f3c60",
                      "prevTxHash": "f34e136ca81c0e32f6fb532b753612715675073f3718b5db009bb275d246fd7a",
                      "originalTxHash": "f34e136ca81c0e32f6fb532b753612715675073f3718b5db009bb275d246fd7a",
                      "gasLimit": 596994205,
                      "gasPrice": 1000000000,
                      "callType": 1,
                      "operation": "transfer",
                      "function": "returnTwoU64"
                    },
                    {
                      "hash": "bc31cb153ae615204625df84fe9ae3a159aa412b7342f3dca958dd5517a08197",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                      "sender": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                      "data": "@00@0a@0218711a00",
                      "prevTxHash": "065291164a8acd27c26b5a8f09664810081fda18cd54fca635196cf9b200297a",
                      "originalTxHash": "f34e136ca81c0e32f6fb532b753612715675073f3718b5db009bb275d246fd7a",
                      "gasLimit": 595152540,
                      "gasPrice": 1000000000,
                      "callType": 2,
                      "logs": {
                        "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                        "events": [
                          {
                            "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                            "identifier": "writeLog",
                            "topics": [
                              "AAAAAAAAAAAFAP/Aj4ZNGKlpx2+xeJLdoJbREzb20P0=",
                              "QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk1MTUyNTQwLCBnYXMgdXNlZCA9IDIyODgwMTU="
                            ],
                            "data": "QDZmNmJAMGFAMDIxODcxMWEwMA=="
                          },
                          {
                            "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                            "identifier": "completedTxEvent",
                            "topics": [
                              "BlKRFkqKzSfCa1qPCWZIEAgf2hjNVPymNRls+bIAKXo="
                            ],
                            "data": null
                          }
                        ]
                      },
                      "operation": "transfer"
                    }
                  ],
                  "logs": {
                    "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                    "events": [
                      {
                        "address": "erd1qqqqqqqqqqqqqpgqllqglpjdrz5kn3m0k9uf9hdqjmg3xdhk6r7se3wvlk",
                        "identifier": "writeLog",
                        "topics": [
                          "5fXsK/a5JVZf0e2Z6ViFglDOQP1zsS1XkuaLvaZ5pzw="
                        ],
                        "data": "QDZmNmI="
                      }
                    ]
                  },
                  "status": "success",
                  "operation": "transfer",
                  "function": "asyncCallAnotherContractReturnTwoU64WithReturningCallback",
                  "initiallyPaidFee": "6230670000000000",
                  "fee": "6230670000000000",
                  "chainID": "D",
                  "version": 2,
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

        let results = find_smart_contract_result(
            &tx_on_network
        )
            .unwrap()
            .unwrap();

        let expected: Vec<Vec<u8>> = vec![];

        assert_eq!(results, expected)
    }

    #[test]
    fn test_transaction_multiple_sc_results() {
        let data = r#"
        {
          "data": {
            "transaction": {
              "type": "normal",
              "processingTypeOnSource": "BuiltInFunctionCall",
              "processingTypeOnDestination": "SCInvoking",
              "hash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
              "nonce": 236,
              "round": 3353069,
              "epoch": 1371,
              "value": "0",
              "receiver": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
              "sender": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
              "gasPrice": 1000000000,
              "gasLimit": 100000000,
              "gasUsed": 12767998,
              "data": "RVNEVFRyYW5zZmVyQDU1NTQ0YjJkMzEzNDY0MzUzNzY0QDhhYzcyMzA0ODllODAwMDBANzM3NzYxNzA1NDZmNmI2NTZlNzM0NjY5Nzg2NTY0NDk2ZTcwNzU3NEA1NzQ1NDc0YzQ0MmQ2MTMyMzg2MzM1MzlAZThkNGE1MTAwMA==",
              "signature": "caed340339e3ae17a92783f5f08f96ac875885e44c25510cd11251ce23f22994985a6605c4d36f841b7110288a5e928f624f150a66a9de8ade36b68028a9af09",
              "sourceShard": 0,
              "destinationShard": 1,
              "blockNonce": 3288476,
              "blockHash": "0e70ea5fb26c58b1029c84e24eb9a661272b6253d30c668af91f167bfd67b2b0",
              "notarizedAtSourceInMetaNonce": 3290316,
              "NotarizedAtSourceInMetaHash": "8200662ca3ade8fa8e1dd3a4184b0a74d4c43de8f4153170a506f60c94ad3e8b",
              "notarizedAtDestinationInMetaNonce": 3290320,
              "notarizedAtDestinationInMetaHash": "e5f332a8f2070fd1c4ff90f5dc1ee691f36e4ecb9cb5c37e8e7c8595036c3792",
              "miniblockType": "TxBlock",
              "miniblockHash": "d271ad87c6cf8653cc950272f3ee5e976820ada80468518fa35fe45b6e33dca8",
              "hyperblockNonce": 3290320,
              "hyperblockHash": "e5f332a8f2070fd1c4ff90f5dc1ee691f36e4ecb9cb5c37e8e7c8595036c3792",
              "timestamp": 1714118414,
              "smartContractResults": [
                {
                  "hash": "c0e63f1018ece1036e3e6dc405553e5f6badfe0f5d2a104f4cd4457a872d02f9",
                  "nonce": 0,
                  "value": 0,
                  "receiver": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                  "sender": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                  "data": "swapTokensFixedInput@5745474c442d613238633539@e8d4a51000",
                  "prevTxHash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
                  "originalTxHash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
                  "gasLimit": 99559500,
                  "gasPrice": 1000000000,
                  "callType": 0,
                  "originalSender": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                  "operation": "transfer",
                  "function": "swapTokensFixedInput"
                },
                {
                  "hash": "40078cec63b6e0d0d9522ea5e6d2d0cb6f21ebae981f354de3dc3545ac2928ad",
                  "nonce": 0,
                  "value": 0,
                  "receiver": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                  "sender": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                  "data": "ESDTTransfer@5745474c442d613238633539@9b35e4dd3902b9",
                  "prevTxHash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
                  "originalTxHash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
                  "gasLimit": 0,
                  "gasPrice": 1000000000,
                  "callType": 0,
                  "originalSender": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                  "logs": {
                    "address": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                    "events": [
                      {
                        "address": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                        "identifier": "ESDTTransfer",
                        "topics": [
                          "V0VHTEQtYTI4YzU5",
                          "",
                          "mzXk3TkCuQ==",
                          "4yr+3JBP4ZOXRq2XO+s4NWPPY2QrpmmzBA+blCil7WA="
                        ],
                        "data": null,
                        "additionalData": [
                          "",
                          "RVNEVFRyYW5zZmVy",
                          "V0VHTEQtYTI4YzU5",
                          "mzXk3TkCuQ=="
                        ]
                      },
                      {
                        "address": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                        "identifier": "writeLog",
                        "topics": [
                          "AAAAAAAAAAAFAKi6nmhia7yZP9eV6LtvR5Q7L1/6fOs="
                        ],
                        "data": "QDZmNmI=",
                        "additionalData": [
                          "QDZmNmI="
                        ]
                      },
                      {
                        "address": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                        "identifier": "completedTxEvent",
                        "topics": [
                          "xtxxjFbIeVFW2Ef0+XaPKxl2pRbTkP3OD1uLrRrDzOU="
                        ],
                        "data": null,
                        "additionalData": null
                      }
                    ]
                  },
                  "tokens": [
                    "WEGLD-a28c59"
                  ],
                  "esdtValues": [
                    "43687878470468281"
                  ],
                  "operation": "ESDTTransfer"
                },
                {
                  "hash": "26487a550721b8282ceafe603bb4d53ee93929ffd9ded39b08e7422adb4d8795",
                  "nonce": 237,
                  "value": 872320020000000,
                  "receiver": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                  "sender": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                  "data": "@6f6b@0000000c5745474c442d6132386335390000000000000000000000079b35e4dd3902b9",
                  "prevTxHash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
                  "originalTxHash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
                  "gasLimit": 0,
                  "gasPrice": 1000000000,
                  "callType": 0,
                  "logs": {
                    "address": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                    "events": [
                      {
                        "address": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                        "identifier": "completedTxEvent",
                        "topics": [
                          "xtxxjFbIeVFW2Ef0+XaPKxl2pRbTkP3OD1uLrRrDzOU="
                        ],
                        "data": null,
                        "additionalData": null
                      }
                    ]
                  },
                  "operation": "transfer",
                  "isRefund": true
                },
                {
                  "hash": "798ba4333a7cedb62f811d942dedb8c0c09bf9945a0d2ccede2eaed967eba135",
                  "nonce": 0,
                  "value": 0,
                  "receiver": "erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz",
                  "sender": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                  "data": "ESDTTransfer@55544b2d313464353764@2d79883d2000@6465706f7369745377617046656573",
                  "prevTxHash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
                  "originalTxHash": "c6dc718c56c8795156d847f4f9768f2b1976a516d390fdce0f5b8bad1ac3cce5",
                  "gasLimit": 0,
                  "gasPrice": 1000000000,
                  "callType": 0,
                  "originalSender": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                  "tokens": [
                    "UTK-14d57d"
                  ],
                  "esdtValues": [
                    "50000000000000"
                  ],
                  "operation": "ESDTTransfer",
                  "function": "depositSwapFees"
                }
              ],
              "logs": {
                "address": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                "events": [
                  {
                    "address": "erd1uv40ahysflse896x4ktnh6ecx43u7cmy9wnxnvcyp7deg299a4sq6vaywa",
                    "identifier": "ESDTTransfer",
                    "topics": [
                      "VVRLLTE0ZDU3ZA==",
                      "",
                      "iscjBInoAAA=",
                      "AAAAAAAAAAAFAKi6nmhia7yZP9eV6LtvR5Q7L1/6fOs="
                    ],
                    "data": null,
                    "additionalData": [
                      "",
                      "RVNEVFRyYW5zZmVy",
                      "VVRLLTE0ZDU3ZA==",
                      "iscjBInoAAA=",
                      "c3dhcFRva2Vuc0ZpeGVkSW5wdXQ=",
                      "V0VHTEQtYTI4YzU5",
                      "6NSlEAA="
                    ]
                  },
                  {
                    "address": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                    "identifier": "ESDTTransfer",
                    "topics": [
                      "VVRLLTE0ZDU3ZA==",
                      "",
                      "LXmIPSAA",
                      "AAAAAAAAAAAFAHHPwyv1rniWOErl2N5cD4oOE2KyfOs="
                    ],
                    "data": "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
                    "additionalData": [
                      "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
                      "RVNEVFRyYW5zZmVy",
                      "VVRLLTE0ZDU3ZA==",
                      "LXmIPSAA",
                      "ZGVwb3NpdFN3YXBGZWVz"
                    ]
                  },
                  {
                    "address": "erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz",
                    "identifier": "depositSwapFees",
                    "topics": [
                      "ZGVwb3NpdF9zd2FwX2ZlZXNfZXZlbnQ=",
                      "AAAAAAAAAAAFAKi6nmhia7yZP9eV6LtvR5Q7L1/6fOs=",
                      "ug==",
                      "AAAAClVUSy0xNGQ1N2QAAAAAAAAAAAAAAAYteYg9IAA="
                    ],
                    "data": null,
                    "additionalData": [
                      ""
                    ]
                  },
                  {
                    "address": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                    "identifier": "ESDTTransfer",
                    "topics": [
                      "V0VHTEQtYTI4YzU5",
                      "",
                      "mzXk3TkCuQ==",
                      "4yr+3JBP4ZOXRq2XO+s4NWPPY2QrpmmzBA+blCil7WA="
                    ],
                    "data": "RGlyZWN0Q2FsbA==",
                    "additionalData": [
                      "RGlyZWN0Q2FsbA==",
                      "RVNEVFRyYW5zZmVy",
                      "V0VHTEQtYTI4YzU5",
                      "mzXk3TkCuQ=="
                    ]
                  },
                  {
                    "address": "erd1qqqqqqqqqqqqqpgq4zafu6rzdw7fj07hjh5tkm68jsaj7hl60n4s8py4ra",
                    "identifier": "swapTokensFixedInput",
                    "topics": [
                      "c3dhcA==",
                      "VVRLLTE0ZDU3ZA==",
                      "V0VHTEQtYTI4YzU5",
                      "4yr+3JBP4ZOXRq2XO+s4NWPPY2QrpmmzBA+blCil7WA=",
                      "BVs="
                    ],
                    "data": "4yr+3JBP4ZOXRq2XO+s4NWPPY2QrpmmzBA+blCil7WAAAAAKVVRLLTE0ZDU3ZAAAAAiKxyMEiegAAAAAAAxXRUdMRC1hMjhjNTkAAAAHmzXk3TkCuQAAAAcjhvJvwQAAAAAACwGBykedC25GCD5kAAAACgGwxHNBlOj27dQAAAAAADItnAAAAAAAAAVbAAAAAGYrXw4=",
                    "additionalData": [
                      "4yr+3JBP4ZOXRq2XO+s4NWPPY2QrpmmzBA+blCil7WAAAAAKVVRLLTE0ZDU3ZAAAAAiKxyMEiegAAAAAAAxXRUdMRC1hMjhjNTkAAAAHmzXk3TkCuQAAAAcjhvJvwQAAAAAACwGBykedC25GCD5kAAAACgGwxHNBlOj27dQAAAAAADItnAAAAAAAAAVbAAAAAGYrXw4="
                    ]
                  }
                ]
              },
              "status": "success",
              "tokens": [
                "UTK-14d57d"
              ],
              "esdtValues": [
                "10000000000000000000"
              ],
              "operation": "ESDTTransfer",
              "function": "swapTokensFixedInput",
              "initiallyPaidFee": "1238095000000000",
              "fee": "365774980000000",
              "chainID": "D",
              "version": 1,
              "options": 0
            }
          },
          "error": "",
          "code": "successful"
        }"#;

        let tx_on_network = serde_json::from_str::<TransactionOnNetworkResponse>(data)
            .unwrap()
            .data
            .unwrap();

        let results = find_smart_contract_result(
            &tx_on_network
        )
            .unwrap()
            .unwrap();

        let expected: Vec<Vec<u8>> = vec![
            hex::decode("0000000c5745474c442d6132386335390000000000000000000000079b35e4dd3902b9").unwrap()
        ];

        assert_eq!(results, expected)
    }

    #[test]
    fn test_with_tx_that_has_sc_result() {
        // transaction data from the devnet, an artificial "10" result has been appended on the original result
        let data = r#"
            {
              "data": {
                "transaction": {
                  "type": "normal",
                  "processingTypeOnSource": "BuiltInFunctionCall",
                  "processingTypeOnDestination": "SCInvoking",
                  "hash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                  "nonce": 30,
                  "round": 7639115,
                  "epoch": 6333,
                  "value": "0",
                  "receiver": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                  "sender": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                  "gasPrice": 1000000000,
                  "gasLimit": 25500000,
                  "gasUsed": 15297149,
                  "data": "RVNEVFRyYW5zZmVyQDQ4NTQ0ZDJkNjY2NTMxNjYzNjM5QDBkZTBiNmIzYTc2NDAwMDBANzM3NzYxNzA1NDZmNmI2NTZlNzM0NjY5Nzg2NTY0NDk2ZTcwNzU3NEA1NzQ1NDc0YzQ0MmQ2NDM3NjMzNjYyNjJAMDM3Yzc3OGZjY2U5YzU1Yg==",
                  "signature": "e912fae4b7a9e51ddf316a5e82a0f457d453a62e3c17477f5d6175e1b33c5e92ddb187d65f54cf3131a0603321290279a0456c20778039f2ab09b54e33c60f0d",
                  "sourceShard": 2,
                  "destinationShard": 1,
                  "blockNonce": 7585351,
                  "blockHash": "e456f38f11fec78ed26d5fda068e912739dceedb2e5ce559bf17614b8386c039",
                  "notarizedAtSourceInMetaNonce": 7601495,
                  "NotarizedAtSourceInMetaHash": "e28c6011d4b3f73f3945cae70ff251e675dfea331a70077c5ab3310e3101af17",
                  "notarizedAtDestinationInMetaNonce": 7601499,
                  "notarizedAtDestinationInMetaHash": "333d4266614e981cc1c5654f85ef496038a8cddac46dfc0ad0b7c44c37ab489d",
                  "miniblockType": "TxBlock",
                  "miniblockHash": "13e041f32fde79ebf1abdcfe692e99516f9ec6778dcb917251b440daa7f1210a",
                  "hyperblockNonce": 7601499,
                  "hyperblockHash": "333d4266614e981cc1c5654f85ef496038a8cddac46dfc0ad0b7c44c37ab489d",
                  "timestamp": 1694386290,
                  "smartContractResults": [
                    {
                      "hash": "a23faa3c80bae0b968f007ff0fad3afdec05b4e71d749c3d583dec10c6eb05a2",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                      "sender": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                      "data": "ESDTTransfer@5745474c442d643763366262@03856446ff9a304b",
                      "prevTxHash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                      "originalTxHash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                      "gasLimit": 0,
                      "gasPrice": 1000000000,
                      "callType": 0,
                      "logs": {
                        "address": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                        "events": [
                          {
                            "address": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                            "identifier": "ESDTTransfer",
                            "topics": [
                              "V0VHTEQtZDdjNmJi",
                              "",
                              "A4VkRv+aMEs=",
                              "qP29NHPKNFkQzDGmOEIaNRCFYABw8ku13ZyenAuvJOY="
                            ],
                            "data": null
                          },
                          {
                            "address": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                            "identifier": "writeLog",
                            "topics": [
                              "AAAAAAAAAAAFAKVe/p1dXpaw/INtyTPaxf3N3LaNfOs="
                            ],
                            "data": "QDZmNmI="
                          },
                          {
                            "address": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                            "identifier": "completedTxEvent",
                            "topics": [
                              "1AWL08E9sLFIMsfFj+Fj2y9Xn/ZUQ4BYa4on2ItKUHA="
                            ],
                            "data": null
                          }
                        ]
                      },
                      "tokens": [
                        "WEGLD-d7c6bb"
                      ],
                      "esdtValues": [
                        "253719210115084363"
                      ],
                      "operation": "ESDTTransfer"
                    },
                    {
                      "hash": "b7b4d15917fd215399d8e772c3c4e732008baaedc2b8172f71c91708ba7523f0",
                      "nonce": 31,
                      "value": 102028510000000,
                      "receiver": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                      "sender": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                      "data": "@6f6b@0000000c5745474c442d64376336626200000000000000000000000803856446ff9a304b@10",
                      "prevTxHash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                      "originalTxHash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                      "gasLimit": 0,
                      "gasPrice": 1000000000,
                      "callType": 0,
                      "logs": {
                        "address": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                        "events": [
                          {
                            "address": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                            "identifier": "completedTxEvent",
                            "topics": [
                              "1AWL08E9sLFIMsfFj+Fj2y9Xn/ZUQ4BYa4on2ItKUHA="
                            ],
                            "data": null
                          }
                        ]
                      },
                      "operation": "transfer",
                      "isRefund": true
                    },
                    {
                      "hash": "05a766ca05d2053d1c0fbeb1797116474a06c86402a3bfd6c132c9a24cfa1bb0",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                      "sender": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                      "data": "swapTokensFixedInput@5745474c442d643763366262@037c778fcce9c55b",
                      "prevTxHash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                      "originalTxHash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                      "gasLimit": 25050500,
                      "gasPrice": 1000000000,
                      "callType": 0,
                      "operation": "transfer",
                      "function": "swapTokensFixedInput"
                    },
                    {
                      "hash": "4e639c80822d5d7780c8326d683fa9cd6d59649d14122dfabc5a96dda36da527",
                      "nonce": 0,
                      "value": 0,
                      "receiver": "erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy",
                      "sender": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                      "data": "ESDTTransfer@5745474c442d643763366262@e7730d1ef1b0@737761704e6f466565416e64466f7277617264@4d45582d646332383963@0000000000000000000000000000000000000000000000000000000000000000",
                      "prevTxHash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                      "originalTxHash": "d4058bd3c13db0b14832c7c58fe163db2f579ff6544380586b8a27d88b4a5070",
                      "gasLimit": 0,
                      "gasPrice": 1000000000,
                      "callType": 0,
                      "tokens": [
                        "WEGLD-d7c6bb"
                      ],
                      "esdtValues": [
                        "254481327387056"
                      ],
                      "operation": "ESDTTransfer",
                      "function": "swapNoFeeAndForward"
                    }
                  ],
                  "logs": {
                    "address": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                    "events": [
                      {
                        "address": "erd14r7m6drneg69jyxvxxnrsss6x5gg2cqqwreyhdwanj0fcza0ynnq5jmy4g",
                        "identifier": "ESDTTransfer",
                        "topics": [
                          "SFRNLWZlMWY2OQ==",
                          "",
                          "DeC2s6dkAAA=",
                          "AAAAAAAAAAAFAKVe/p1dXpaw/INtyTPaxf3N3LaNfOs="
                        ],
                        "data": null
                      },
                      {
                        "address": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                        "identifier": "ESDTTransfer",
                        "topics": [
                          "V0VHTEQtZDdjNmJi",
                          "",
                          "53MNHvGw",
                          "AAAAAAAAAAAFAOcoOHa5zr9eiFpjeVvIJxVDpaz7fOs="
                        ],
                        "data": null
                      },
                      {
                        "address": "erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy",
                        "identifier": "ESDTLocalBurn",
                        "topics": [
                          "TUVYLWRjMjg5Yw==",
                          "",
                          "AuMDPq1jy03x"
                        ],
                        "data": null
                      },
                      {
                        "address": "erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy",
                        "identifier": "swapNoFeeAndForward",
                        "topics": [
                          "c3dhcF9ub19mZWVfYW5kX2ZvcndhcmQ=",
                          "TUVYLWRjMjg5Yw==",
                          "AAAAAAAAAAAFAKVe/p1dXpaw/INtyTPaxf3N3LaNfOs=",
                          "GL0="
                        ],
                        "data": "AAAAAAAAAAAFAKVe/p1dXpaw/INtyTPaxf3N3LaNfOsAAAAMV0VHTEQtZDdjNmJiAAAABudzDR7xsAAAAApNRVgtZGMyODljAAAACQLjAz6tY8tN8QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABzvkcAAAAAAAAYvQAAAABk/khy"
                      },
                      {
                        "address": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                        "identifier": "ESDTTransfer",
                        "topics": [
                          "V0VHTEQtZDdjNmJi",
                          "",
                          "A4VkRv+aMEs=",
                          "qP29NHPKNFkQzDGmOEIaNRCFYABw8ku13ZyenAuvJOY="
                        ],
                        "data": null
                      },
                      {
                        "address": "erd1qqqqqqqqqqqqqpgq5400a82at6ttplyrdhyn8kk9lhxaed5d0n4s9s77kz",
                        "identifier": "swapTokensFixedInput",
                        "topics": [
                          "c3dhcA==",
                          "SFRNLWZlMWY2OQ==",
                          "V0VHTEQtZDdjNmJi",
                          "qP29NHPKNFkQzDGmOEIaNRCFYABw8ku13ZyenAuvJOY=",
                          "GL0="
                        ],
                        "data": "qP29NHPKNFkQzDGmOEIaNRCFYABw8ku13ZyenAuvJOYAAAAKSFRNLWZlMWY2OQAAAAgN4Lazp2QAAAAAAAxXRUdMRC1kN2M2YmIAAAAIA4VkRv+aMEsAAAAHA41+pMaAAAAAAAoofxtJRPkr8X9kAAAACgpOPCsHUu261HUAAAAAAHO+RwAAAAAAABi9AAAAAGT+SHI="
                      }
                    ]
                  },
                  "status": "success",
                  "tokens": [
                    "HTM-fe1f69"
                  ],
                  "esdtValues": [
                    "1000000000000000000"
                  ],
                  "operation": "ESDTTransfer",
                  "function": "swapTokensFixedInput",
                  "initiallyPaidFee": "502005000000000",
                  "fee": "399976490000000",
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

        let results = find_smart_contract_result(
            &tx_on_network
        )
            .unwrap()
            .unwrap();

        let expected: Vec<Vec<u8>> = vec![
            hex::decode("0000000c5745474c442d64376336626200000000000000000000000803856446ff9a304b")
                .unwrap(),
            hex::decode("10").unwrap(),
        ];

        assert_eq!(results, expected)
    }

    #[test]
    fn test_with_tx_that_has_no_sc_result() {
        // transaction data from the devnet
        let data = r#"
            {
              "data": {
                "transaction": {
                  "type": "normal",
                  "processingTypeOnSource": "SCInvoking",
                  "processingTypeOnDestination": "SCInvoking",
                  "hash": "6afac3ec13c89cc56154d06efdb457a24f58361699eee00a48202a8f8adc8c8a",
                  "nonce": 17,
                  "round": 7548071,
                  "epoch": 6257,
                  "value": "0",
                  "receiver": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                  "sender": "erd1uh67c2lkhyj4vh73akv7jky9sfgvus8awwcj64uju69mmfne5u7q299t7g",
                  "gasPrice": 1000000000,
                  "gasLimit": 600000000,
                  "gasUsed": 600000000,
                  "data": "cmV0dXJuVHdvVTY0",
                  "signature": "f3a3ca96a78c90c9cf1b08541e1777010f0176a5e1e525e631155b2784932cbfd74c9168d03ba201fd5434d1a1b4789895ddade9883eca2ee9e0bce18468fb00",
                  "sourceShard": 0,
                  "destinationShard": 0,
                  "blockNonce": 7502091,
                  "blockHash": "5ec66c651cb1514cba200e7e80a4491880f0db678ce7631c397872e3842f0aa2",
                  "notarizedAtSourceInMetaNonce": 7510505,
                  "NotarizedAtSourceInMetaHash": "8410309ec5b988af79b4dcfb44fd4729d46874ebd796672c78e417e314409051",
                  "notarizedAtDestinationInMetaNonce": 7510505,
                  "notarizedAtDestinationInMetaHash": "8410309ec5b988af79b4dcfb44fd4729d46874ebd796672c78e417e314409051",
                  "miniblockType": "TxBlock",
                  "miniblockHash": "fb150e515449c9b658879ed06f256b429239cbe78ec2c2821deb4b283ff21554",
                  "hyperblockNonce": 7510505,
                  "hyperblockHash": "8410309ec5b988af79b4dcfb44fd4729d46874ebd796672c78e417e314409051",
                  "timestamp": 1693840026,
                  "logs": {
                    "address": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                    "events": [
                      {
                        "address": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                        "identifier": "writeLog",
                        "topics": [
                          "5fXsK/a5JVZf0e2Z6ViFglDOQP1zsS1XkuaLvaZ5pzw=",
                          "QHRvbyBtdWNoIGdhcyBwcm92aWRlZCBmb3IgcHJvY2Vzc2luZzogZ2FzIHByb3ZpZGVkID0gNTk5OTMyMDAwLCBnYXMgdXNlZCA9IDE4NDE2NjU="
                        ],
                        "data": "QDZmNmJAMGFAMDIxODcxMWEwMA=="
                      },
                      {
                        "address": "erd1qqqqqqqqqqqqqpgq4nlkk7jwhqgp4r08lal46tqt70jdv0685u7qrr3l2d",
                        "identifier": "completedTxEvent",
                        "topics": [
                          "avrD7BPInMVhVNBu/bRXok9YNhaZ7uAKSCAqj4rcjIo="
                        ],
                        "data": null
                      }
                    ]
                  },
                  "status": "success",
                  "operation": "transfer",
                  "function": "returnTwoU64",
                  "initiallyPaidFee": "6067320000000000",
                  "fee": "6067320000000000",
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

        let results = find_smart_contract_result(
            &tx_on_network
        )
            .unwrap()
            .unwrap();

        let expected: Vec<Vec<u8>> = vec![
            hex::decode("0a").unwrap(),
            hex::decode("0218711a00").unwrap(),
        ];

        assert_eq!(results, expected)
    }
}