use serde::Deserialize;

use crate::utils::transaction::results::find_sc_error;

pub(crate) const SUCCESS_TRANSACTION_STATUS: [&str; 2] = ["success", "successful"];
pub(crate) const FINAL_TRANSACTION_STATUS: [&str; 3] = ["success", "successful", "fail"];

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkResponse {
    pub data: Option<TransactionOnNetwork>,
    pub error: String
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetwork {
    pub transaction: TransactionOnNetworkTransaction
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkTransaction {
    pub gas_used: u64,
    pub smart_contract_results: Option<Vec<TransactionOnNetworkTransactionSmartContractResult>>,
    pub status: String,
    pub logs: Option<TransactionOnNetworkTransactionLogs>
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkTransactionSmartContractResult {
    pub hash: String,
    pub nonce: u64,
    pub data: String,
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkTransactionLogs {
    pub address: String,
    pub events: Vec<TransactionOnNetworkTransactionLogsEvents>
}

#[derive(Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOnNetworkTransactionLogsEvents {
    pub address: String,
    pub identifier: String,
    pub topics: Vec<String>,
    pub data: Option<String>
}

impl TransactionOnNetwork {
    pub fn is_success(&self) -> bool {
        if !SUCCESS_TRANSACTION_STATUS.contains(&self.transaction.status.as_ref())  {
            return false;
        }

        let Some(logs) = self.transaction.logs.as_ref() else {
            return true
        };

        matches!(find_sc_error(logs), Ok(None))
    }
}

#[cfg(test)]
mod tests {
    use crate::TransactionOnNetworkResponse;

    #[test]
    fn test_deserialize_successful_swap() {
        let data = r#"
{
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "BuiltInFunctionCall",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
      "nonce": 1255,
      "round": 3844407,
      "epoch": 1576,
      "value": "0",
      "receiver": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
      "sender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
      "gasPrice": 1000000000,
      "gasLimit": 30000000,
      "gasUsed": 22468098,
      "data": "RVNEVFRyYW5zZmVyQDU3NDU0NzRjNDQyZDYxMzIzODYzMzUzOUAwZGUwYjZiM2E3NjQwMDAwQDczNzc2MTcwNTQ2ZjZiNjU2ZTczNDY2OTc4NjU2NDQ5NmU3MDc1NzRANTU1MzQ0NDMyZDMzMzUzMDYzMzQ2NUAwMWEyNTc5OA==",
      "signature": "7183d3cb779ba376e6cb1f09cdf03325ee2b60e5274374b47e080981b802f41692c499f114fe352802bd7a7695aac1edf6c27cc6cc1a4f0fba54aba1f54e4c03",
      "sourceShard": 1,
      "destinationShard": 1,
      "blockNonce": 3778507,
      "blockHash": "f088d44a244c21913767b484719c430589cbe04419df33ea53360b911ce8ae3f",
      "notarizedAtSourceInMetaNonce": 3780438,
      "NotarizedAtSourceInMetaHash": "0cb49fcb2a943d8e155bd041032c93d953487e24ff14387c369b7cce5a224d2b",
      "notarizedAtDestinationInMetaNonce": 3780438,
      "notarizedAtDestinationInMetaHash": "0cb49fcb2a943d8e155bd041032c93d953487e24ff14387c369b7cce5a224d2b",
      "miniblockType": "TxBlock",
      "miniblockHash": "88fd59d5dba5305633c2992955f424f2aa97a008120c52ef9e10bfa9b51cd61a",
      "hyperblockNonce": 3780438,
      "hyperblockHash": "0cb49fcb2a943d8e155bd041032c93d953487e24ff14387c369b7cce5a224d2b",
      "timestamp": 1717066442,
      "smartContractResults": [
        {
          "hash": "75b44b7c9a3b0fd35c70162d006e576f77af592a5f2a6eaad22cdd24b57c64e9",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "ESDTTransfer@5745474c442d613238633539@0388f27d8d3000@737761704e6f466565416e64466f7277617264@4d45582d613635396430@0000000000000000000000000000000000000000000000000000000000000000",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "tokens": [
            "WEGLD-a28c59"
          ],
          "esdtValues": [
            "995000000000000"
          ],
          "operation": "ESDTTransfer",
          "function": "swapNoFeeAndForward"
        },
        {
          "hash": "0c8456d6acb7b835dd2ed5f851bcfde19c898086ed0c5cfd1edabd5360c28687",
          "nonce": 1256,
          "value": 75319020000000,
          "receiver": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "@6f6b@0000000b555344432d33353063346500000000000000000000000401a6868d",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "operation": "transfer",
          "isRefund": true
        },
        {
          "hash": "9054856f6e4b361801b69efa70e7265b017657f273452b5e2eaafdcbd11dad2e",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "sender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "data": "swapTokensFixedInput@555344432d333530633465@01a25798",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 29559500,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "operation": "transfer",
          "function": "swapTokensFixedInput"
        },
        {
          "hash": "96a0496387b4b4f790cf30012bd434732ac60d3b1eb0d3f04c39fa2705432397",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "ESDTTransfer@5745474c442d613238633539@048c27395000@6465706f7369745377617046656573",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "tokens": [
            "WEGLD-a28c59"
          ],
          "esdtValues": [
            "5000000000000"
          ],
          "operation": "ESDTTransfer",
          "function": "depositSwapFees"
        },
        {
          "hash": "fa78830f8c2629c840f4e558c198cc0986f4d1ea4d76e7a6ade86911620d8c1c",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "ESDTTransfer@555344432d333530633465@01a6868d",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "tokens": [
            "USDC-350c4e"
          ],
          "esdtValues": [
            "27690637"
          ],
          "operation": "ESDTTransfer"
        }
      ],
      "logs": {
        "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
        "events": [
          {
            "address": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
            "identifier": "ESDTTransfer",
            "topics": [
              "V0VHTEQtYTI4YzU5",
              "",
              "DeC2s6dkAAA=",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs="
            ],
            "data": null,
            "additionalData": [
              "",
              "RVNEVFRyYW5zZmVy",
              "V0VHTEQtYTI4YzU5",
              "DeC2s6dkAAA=",
              "c3dhcFRva2Vuc0ZpeGVkSW5wdXQ=",
              "VVNEQy0zNTBjNGU=",
              "AaJXmA=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "ESDTTransfer",
            "topics": [
              "V0VHTEQtYTI4YzU5",
              "",
              "BIwnOVAA",
              "AAAAAAAAAAAFAHHPwyv1rniWOErl2N5cD4oOE2KyfOs="
            ],
            "data": "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
            "additionalData": [
              "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
              "RVNEVFRyYW5zZmVy",
              "V0VHTEQtYTI4YzU5",
              "BIwnOVAA",
              "ZGVwb3NpdFN3YXBGZWVz"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz",
            "identifier": "depositSwapFees",
            "topics": [
              "ZGVwb3NpdF9zd2FwX2ZlZXNfZXZlbnQ=",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=",
              "1w==",
              "AAAADFdFR0xELWEyOGM1OQAAAAAAAAAAAAAABgSMJzlQAA=="
            ],
            "data": null,
            "additionalData": [
              ""
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "ESDTTransfer",
            "topics": [
              "V0VHTEQtYTI4YzU5",
              "",
              "A4jyfY0wAA==",
              "AAAAAAAAAAAFABOe165KoDeS5ryzMjlKQP50bu+kfOs="
            ],
            "data": "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
            "additionalData": [
              "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
              "RVNEVFRyYW5zZmVy",
              "V0VHTEQtYTI4YzU5",
              "A4jyfY0wAA==",
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
              "AWETfrHs3tgu/Q=="
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
              "Big="
            ],
            "data": "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAAMV0VHTEQtYTI4YzU5AAAABwOI8n2NMAAAAAAKTUVYLWE2NTlkMAAAAAoBYRN+seze2C79AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADmnywAAAAAAAAYoAAAAAGZYWso=",
            "additionalData": [
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAAMV0VHTEQtYTI4YzU5AAAABwOI8n2NMAAAAAAKTUVYLWE2NTlkMAAAAAoBYRN+seze2C79AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADmnywAAAAAAAAYoAAAAAGZYWso="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "ESDTTransfer",
            "topics": [
              "VVNEQy0zNTBjNGU=",
              "",
              "AaaGjQ==",
              "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOs="
            ],
            "data": "RGlyZWN0Q2FsbA==",
            "additionalData": [
              "RGlyZWN0Q2FsbA==",
              "RVNEVFRyYW5zZmVy",
              "VVNEQy0zNTBjNGU=",
              "AaaGjQ=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "swapTokensFixedInput",
            "topics": [
              "c3dhcA==",
              "V0VHTEQtYTI4YzU5",
              "VVNEQy0zNTBjNGU=",
              "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOs=",
              "Big="
            ],
            "data": "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOsAAAAMV0VHTEQtYTI4YzU5AAAACA3gtrOnZAAAAAAAC1VTREMtMzUwYzRlAAAABAGmho0AAAAHA41+pMaAAAAAAAoEzt4YZBpkcbUoAAAABZLSUp2BAAAAAAA5p8sAAAAAAAAGKAAAAABmWFrK",
            "additionalData": [
              "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOsAAAAMV0VHTEQtYTI4YzU5AAAACA3gtrOnZAAAAAAAC1VTREMtMzUwYzRlAAAABAGmho0AAAAHA41+pMaAAAAAAAoEzt4YZBpkcbUoAAAABZLSUp2BAAAAAAA5p8sAAAAAAAAGKAAAAABmWFrK"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "completedTxEvent",
            "topics": [
              "hKyvZSjmVb0ZMKwSsT7mivFavYtfxLpGRLULpPiUADI="
            ],
            "data": null,
            "additionalData": null
          }
        ]
      },
      "status": "success",
      "tokens": [
        "WEGLD-a28c59"
      ],
      "esdtValues": [
        "1000000000000000000"
      ],
      "operation": "ESDTTransfer",
      "function": "swapTokensFixedInput",
      "initiallyPaidFee": "538095000000000",
      "fee": "462775980000000",
      "chainID": "D",
      "version": 1,
      "options": 0
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        _ = serde_json::from_str::<TransactionOnNetworkResponse>(data).unwrap();
    }

    #[test]
    fn test_is_success_successful_swap() {
        let data = r#"
{
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "BuiltInFunctionCall",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
      "nonce": 1255,
      "round": 3844407,
      "epoch": 1576,
      "value": "0",
      "receiver": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
      "sender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
      "gasPrice": 1000000000,
      "gasLimit": 30000000,
      "gasUsed": 22468098,
      "data": "RVNEVFRyYW5zZmVyQDU3NDU0NzRjNDQyZDYxMzIzODYzMzUzOUAwZGUwYjZiM2E3NjQwMDAwQDczNzc2MTcwNTQ2ZjZiNjU2ZTczNDY2OTc4NjU2NDQ5NmU3MDc1NzRANTU1MzQ0NDMyZDMzMzUzMDYzMzQ2NUAwMWEyNTc5OA==",
      "signature": "7183d3cb779ba376e6cb1f09cdf03325ee2b60e5274374b47e080981b802f41692c499f114fe352802bd7a7695aac1edf6c27cc6cc1a4f0fba54aba1f54e4c03",
      "sourceShard": 1,
      "destinationShard": 1,
      "blockNonce": 3778507,
      "blockHash": "f088d44a244c21913767b484719c430589cbe04419df33ea53360b911ce8ae3f",
      "notarizedAtSourceInMetaNonce": 3780438,
      "NotarizedAtSourceInMetaHash": "0cb49fcb2a943d8e155bd041032c93d953487e24ff14387c369b7cce5a224d2b",
      "notarizedAtDestinationInMetaNonce": 3780438,
      "notarizedAtDestinationInMetaHash": "0cb49fcb2a943d8e155bd041032c93d953487e24ff14387c369b7cce5a224d2b",
      "miniblockType": "TxBlock",
      "miniblockHash": "88fd59d5dba5305633c2992955f424f2aa97a008120c52ef9e10bfa9b51cd61a",
      "hyperblockNonce": 3780438,
      "hyperblockHash": "0cb49fcb2a943d8e155bd041032c93d953487e24ff14387c369b7cce5a224d2b",
      "timestamp": 1717066442,
      "smartContractResults": [
        {
          "hash": "75b44b7c9a3b0fd35c70162d006e576f77af592a5f2a6eaad22cdd24b57c64e9",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgqzw0d0tj25qme9e4ukverjjjqle6xamay0n4s5r0v9g",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "ESDTTransfer@5745474c442d613238633539@0388f27d8d3000@737761704e6f466565416e64466f7277617264@4d45582d613635396430@0000000000000000000000000000000000000000000000000000000000000000",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "tokens": [
            "WEGLD-a28c59"
          ],
          "esdtValues": [
            "995000000000000"
          ],
          "operation": "ESDTTransfer",
          "function": "swapNoFeeAndForward"
        },
        {
          "hash": "0c8456d6acb7b835dd2ed5f851bcfde19c898086ed0c5cfd1edabd5360c28687",
          "nonce": 1256,
          "value": 75319020000000,
          "receiver": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "@6f6b@0000000b555344432d33353063346500000000000000000000000401a6868d",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "operation": "transfer",
          "isRefund": true
        },
        {
          "hash": "9054856f6e4b361801b69efa70e7265b017657f273452b5e2eaafdcbd11dad2e",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "sender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "data": "swapTokensFixedInput@555344432d333530633465@01a25798",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 29559500,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "operation": "transfer",
          "function": "swapTokensFixedInput"
        },
        {
          "hash": "96a0496387b4b4f790cf30012bd434732ac60d3b1eb0d3f04c39fa2705432397",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "ESDTTransfer@5745474c442d613238633539@048c27395000@6465706f7369745377617046656573",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "tokens": [
            "WEGLD-a28c59"
          ],
          "esdtValues": [
            "5000000000000"
          ],
          "operation": "ESDTTransfer",
          "function": "depositSwapFees"
        },
        {
          "hash": "fa78830f8c2629c840f4e558c198cc0986f4d1ea4d76e7a6ade86911620d8c1c",
          "nonce": 0,
          "value": 0,
          "receiver": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "sender": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
          "data": "ESDTTransfer@555344432d333530633465@01a6868d",
          "prevTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "originalTxHash": "84acaf6528e655bd1930ac12b13ee68af15abd8b5fc4ba4644b50ba4f8940032",
          "gasLimit": 0,
          "gasPrice": 1000000000,
          "callType": 0,
          "originalSender": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
          "tokens": [
            "USDC-350c4e"
          ],
          "esdtValues": [
            "27690637"
          ],
          "operation": "ESDTTransfer"
        }
      ],
      "logs": {
        "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
        "events": [
          {
            "address": "erd1x39tc3q3nn72ecjnmcz7x0qp09kp97t080x99dgyhx7zh95j0n4szskhlv",
            "identifier": "ESDTTransfer",
            "topics": [
              "V0VHTEQtYTI4YzU5",
              "",
              "DeC2s6dkAAA=",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs="
            ],
            "data": null,
            "additionalData": [
              "",
              "RVNEVFRyYW5zZmVy",
              "V0VHTEQtYTI4YzU5",
              "DeC2s6dkAAA=",
              "c3dhcFRva2Vuc0ZpeGVkSW5wdXQ=",
              "VVNEQy0zNTBjNGU=",
              "AaJXmA=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "ESDTTransfer",
            "topics": [
              "V0VHTEQtYTI4YzU5",
              "",
              "BIwnOVAA",
              "AAAAAAAAAAAFAHHPwyv1rniWOErl2N5cD4oOE2KyfOs="
            ],
            "data": "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
            "additionalData": [
              "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
              "RVNEVFRyYW5zZmVy",
              "V0VHTEQtYTI4YzU5",
              "BIwnOVAA",
              "ZGVwb3NpdFN3YXBGZWVz"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqw88ux2l44eufvwz2uhvduhq03g8pxc4j0n4s0frzjz",
            "identifier": "depositSwapFees",
            "topics": [
              "ZGVwb3NpdF9zd2FwX2ZlZXNfZXZlbnQ=",
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOs=",
              "1w==",
              "AAAADFdFR0xELWEyOGM1OQAAAAAAAAAAAAAABgSMJzlQAA=="
            ],
            "data": null,
            "additionalData": [
              ""
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "ESDTTransfer",
            "topics": [
              "V0VHTEQtYTI4YzU5",
              "",
              "A4jyfY0wAA==",
              "AAAAAAAAAAAFABOe165KoDeS5ryzMjlKQP50bu+kfOs="
            ],
            "data": "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
            "additionalData": [
              "RXhlY3V0ZU9uRGVzdENvbnRleHQ=",
              "RVNEVFRyYW5zZmVy",
              "V0VHTEQtYTI4YzU5",
              "A4jyfY0wAA==",
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
              "AWETfrHs3tgu/Q=="
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
              "Big="
            ],
            "data": "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAAMV0VHTEQtYTI4YzU5AAAABwOI8n2NMAAAAAAKTUVYLWE2NTlkMAAAAAoBYRN+seze2C79AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADmnywAAAAAAAAYoAAAAAGZYWso=",
            "additionalData": [
              "AAAAAAAAAAAFAFgTchSw4UwpSGChbBEEKqcavBcgfOsAAAAMV0VHTEQtYTI4YzU5AAAABwOI8n2NMAAAAAAKTUVYLWE2NTlkMAAAAAoBYRN+seze2C79AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADmnywAAAAAAAAYoAAAAAGZYWso="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "ESDTTransfer",
            "topics": [
              "VVNEQy0zNTBjNGU=",
              "",
              "AaaGjQ==",
              "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOs="
            ],
            "data": "RGlyZWN0Q2FsbA==",
            "additionalData": [
              "RGlyZWN0Q2FsbA==",
              "RVNEVFRyYW5zZmVy",
              "VVNEQy0zNTBjNGU=",
              "AaaGjQ=="
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "swapTokensFixedInput",
            "topics": [
              "c3dhcA==",
              "V0VHTEQtYTI4YzU5",
              "VVNEQy0zNTBjNGU=",
              "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOs=",
              "Big="
            ],
            "data": "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOsAAAAMV0VHTEQtYTI4YzU5AAAACA3gtrOnZAAAAAAAC1VTREMtMzUwYzRlAAAABAGmho0AAAAHA41+pMaAAAAAAAoEzt4YZBpkcbUoAAAABZLSUp2BAAAAAAA5p8sAAAAAAAAGKAAAAABmWFrK",
            "additionalData": [
              "NEq8RBGc/KziU94F4zwBeWwS+W87zFK1BLm8K5aSfOsAAAAMV0VHTEQtYTI4YzU5AAAACA3gtrOnZAAAAAAAC1VTREMtMzUwYzRlAAAABAGmho0AAAAHA41+pMaAAAAAAAoEzt4YZBpkcbUoAAAABZLSUp2BAAAAAAA5p8sAAAAAAAAGKAAAAABmWFrK"
            ]
          },
          {
            "address": "erd1qqqqqqqqqqqqqpgqtqfhy99su9xzjjrq59kpzpp25udtc9eq0n4sr90ax6",
            "identifier": "completedTxEvent",
            "topics": [
              "hKyvZSjmVb0ZMKwSsT7mivFavYtfxLpGRLULpPiUADI="
            ],
            "data": null,
            "additionalData": null
          }
        ]
      },
      "status": "success",
      "tokens": [
        "WEGLD-a28c59"
      ],
      "esdtValues": [
        "1000000000000000000"
      ],
      "operation": "ESDTTransfer",
      "function": "swapTokensFixedInput",
      "initiallyPaidFee": "538095000000000",
      "fee": "462775980000000",
      "chainID": "D",
      "version": 1,
      "options": 0
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        let tx_on_network = serde_json::from_str::<TransactionOnNetworkResponse>(data).unwrap();

        assert!(tx_on_network.data.unwrap().is_success());
    }

    #[test]
    fn test_deserialize_pending_swap_multiple_contracts_transaction() {
        let data = r#"
{
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "SCInvoking",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "4cba466f2506d80a5fd19f36079a8b82ff5683e77d02e6612d29892d1b95b945",
      "nonce": 29,
      "round": 0,
      "epoch": 0,
      "value": "0",
      "receiver": "erd1qqqqqqqqqqqqqpgqc7fc277m787qr54cgftsr9marwlca9ee37esyyw4f5",
      "sender": "erd17pl75t9wlvk6dm0u8hkhw2fw6yks6wv3ansnwzj0ch9m0xlk37es3y9x4c",
      "gasPrice": 1000000000,
      "gasLimit": 80000000,
      "gasUsed": 318500,
      "data": "eEV4Y2hhbmdlU3dhcEAwMDAwMDAwMDAwMDAwMDAwMDUwMDU4MTM3MjE0YjBlMTRjMjk0ODYwYTE2YzExMDQyYWE3MWFiYzE3MjA3Y2ViQDU1NTM0NDQzMmQzMzM1MzA2MzM0NjVAMDIxY2NmQDAwMDAwMDBjNTc0NTQ3NGM0NDJkNjEzMjM4NjMzNTM5MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDA3YjFhMmJjMmVjNTAwMDA=",
      "signature": "597919b9532a10ef0b3bafdb8f0bb0ab4190614028027fcd489ea16e666652ca5a7224662c959bc3713174f9caa102743cd25d5fce38d44e137e5922eaa2ea02",
      "sourceShard": 1,
      "destinationShard": 1,
      "status": "pending",
      "initiallyPaidFee": "80000000000000000",
      "fee": "318500000000000",
      "chainID": "D",
      "version": 1,
      "options": 0
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        _ = serde_json::from_str::<TransactionOnNetworkResponse>(data).unwrap();
    }

    #[test]
    fn test_is_success_for_pending_transaction() {
        let data = r#"
{
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "SCInvoking",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "4cba466f2506d80a5fd19f36079a8b82ff5683e77d02e6612d29892d1b95b945",
      "nonce": 29,
      "round": 0,
      "epoch": 0,
      "value": "0",
      "receiver": "erd1qqqqqqqqqqqqqpgqc7fc277m787qr54cgftsr9marwlca9ee37esyyw4f5",
      "sender": "erd17pl75t9wlvk6dm0u8hkhw2fw6yks6wv3ansnwzj0ch9m0xlk37es3y9x4c",
      "gasPrice": 1000000000,
      "gasLimit": 80000000,
      "gasUsed": 318500,
      "data": "eEV4Y2hhbmdlU3dhcEAwMDAwMDAwMDAwMDAwMDAwMDUwMDU4MTM3MjE0YjBlMTRjMjk0ODYwYTE2YzExMDQyYWE3MWFiYzE3MjA3Y2ViQDU1NTM0NDQzMmQzMzM1MzA2MzM0NjVAMDIxY2NmQDAwMDAwMDBjNTc0NTQ3NGM0NDJkNjEzMjM4NjMzNTM5MDAwMDAwMDAwMDAwMDAwMDAwMDAwMDA3YjFhMmJjMmVjNTAwMDA=",
      "signature": "597919b9532a10ef0b3bafdb8f0bb0ab4190614028027fcd489ea16e666652ca5a7224662c959bc3713174f9caa102743cd25d5fce38d44e137e5922eaa2ea02",
      "sourceShard": 1,
      "destinationShard": 1,
      "status": "pending",
      "initiallyPaidFee": "80000000000000000",
      "fee": "318500000000000",
      "chainID": "D",
      "version": 1,
      "options": 0
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        let tx_on_network = serde_json::from_str::<TransactionOnNetworkResponse>(data).unwrap();

        assert!(!tx_on_network.data.unwrap().is_success())
    }

    #[test]
    fn test_deserialize_execution_failed_user_error_transaction() {
        let data = r#"
{
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "BuiltInFunctionCall",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "36878321d56fca840ebb138f9473067f72dc120602854e550ea4a45b6c69ef43",
      "nonce": 358,
      "round": 1226784,
      "epoch": 510,
      "value": "0",
      "receiver": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
      "sender": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
      "gasPrice": 1000000000,
      "gasLimit": 500000000,
      "gasUsed": 500000000,
      "data": "TXVsdGlFU0RUTkZUVHJhbnNmZXJAMDAwMDAwMDAwMDAwMDAwMDA1MDBjNDM0OWU1ZTg0MzMzMTMwZTJiMzA5NTYyNGYyNDY1YjQxZDYzNDc4MDQ2M0AwMUA0MTU0NTMyZDM0NjMzMDM5MzIzMEBAOGFjNzIzMDQ4OWU4MDAwMEA2NTZlNzQ2NTcyNTM3NDYxNmI2NQ==",
      "signature": "57a5f6a2f48bdb739d2cbf80e9480694208c8d30beed84f1dd94c03123b7796d1834950adaf8cf77c27151437035a87613f62d453aa2667c04e42f949f9fb106",
      "sourceShard": 1,
      "destinationShard": 1,
      "blockNonce": 1225573,
      "blockHash": "9ce7c413116101432a4541e7587208acd7e139fcf326184eb41b08daaffaaf56",
      "notarizedAtSourceInMetaNonce": 1225798,
      "NotarizedAtSourceInMetaHash": "01d4e84604742ac1b159d46bf1b02f58189ed8de55216a289eded34f72b6afdb",
      "notarizedAtDestinationInMetaNonce": 1225798,
      "notarizedAtDestinationInMetaHash": "01d4e84604742ac1b159d46bf1b02f58189ed8de55216a289eded34f72b6afdb",
      "miniblockType": "TxBlock",
      "miniblockHash": "a1f2fc40b6fc0beb17d8dacab8a726a07cf482cd43e4091b4a02f43f5c3fe592",
      "hyperblockNonce": 1225798,
      "hyperblockHash": "01d4e84604742ac1b159d46bf1b02f58189ed8de55216a289eded34f72b6afdb",
      "timestamp": 1701360704,
      "logs": {
        "address": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
        "events": [
          {
            "address": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
            "identifier": "MultiESDTNFTTransfer",
            "topics": [
              "QVRTLTRjMDkyMA==",
              "",
              "iscjBInoAAA=",
              "AAAAAAAAAAAFAMQ0nl6EMzEw4rMJViTyRltB1jR4BGM="
            ],
            "data": null,
            "additionalData": null
          },
          {
            "address": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
            "identifier": "signalError",
            "topics": [
              "lq3O+DrwJ9dElYcbuyqGhxlbHdDQXM6Y6WqYvfMRBGM=",
              "RXNkdExvY2FsUm9sZUJ1cm4gaXMgbWlzc2luZyBmb3IgYXNzZXQgdG9rZW4="
            ],
            "data": "QDc1NzM2NTcyMjA2NTcyNzI2Zjcy",
            "additionalData": null
          },
          {
            "address": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
            "identifier": "internalVMErrors",
            "topics": [
              "AAAAAAAAAAAFAMQ0nl6EMzEw4rMJViTyRltB1jR4BGM=",
              "ZW50ZXJTdGFrZQ=="
            ],
            "data": "CglydW50aW1lLmdvOjExNzUgW2Vycm9yIHNpZ25hbGxlZCBieSBzbWFydGNvbnRyYWN0XSBbZW50ZXJTdGFrZV0KCXJ1bnRpbWUuZ286MTE3NSBbZXJyb3Igc2lnbmFsbGVkIGJ5IHNtYXJ0Y29udHJhY3RdIFtlbnRlclN0YWtlXQoJcnVudGltZS5nbzoxMTcyIFtFc2R0TG9jYWxSb2xlQnVybiBpcyBtaXNzaW5nIGZvciBhc3NldCB0b2tlbl0=",
            "additionalData": null
          }
        ]
      },
      "status": "success",
      "tokens": [
        "ATS-4c0920"
      ],
      "esdtValues": [
        "10000000000000000000"
      ],
      "receivers": [
        "erd1qqqqqqqqqqqqqpgqcs6fuh5yxvcnpc4np9tzfujxtdqavdrcq33sfl9n6z"
      ],
      "receiversShardIDs": [1],
      "operation": "MultiESDTNFTTransfer",
      "function": "enterStake",
      "initiallyPaidFee": "5269280000000000",
      "fee": "5269280000000000",
      "chainID": "D",
      "version": 1,
      "options": 0
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        _ = serde_json::from_str::<TransactionOnNetworkResponse>(data).unwrap();
    }

    #[test]
    fn test_is_success_execution_failed_user_error_transaction() {
        let data = r#"
{
  "data": {
    "transaction": {
      "type": "normal",
      "processingTypeOnSource": "BuiltInFunctionCall",
      "processingTypeOnDestination": "SCInvoking",
      "hash": "36878321d56fca840ebb138f9473067f72dc120602854e550ea4a45b6c69ef43",
      "nonce": 358,
      "round": 1226784,
      "epoch": 510,
      "value": "0",
      "receiver": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
      "sender": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
      "gasPrice": 1000000000,
      "gasLimit": 500000000,
      "gasUsed": 500000000,
      "data": "TXVsdGlFU0RUTkZUVHJhbnNmZXJAMDAwMDAwMDAwMDAwMDAwMDA1MDBjNDM0OWU1ZTg0MzMzMTMwZTJiMzA5NTYyNGYyNDY1YjQxZDYzNDc4MDQ2M0AwMUA0MTU0NTMyZDM0NjMzMDM5MzIzMEBAOGFjNzIzMDQ4OWU4MDAwMEA2NTZlNzQ2NTcyNTM3NDYxNmI2NQ==",
      "signature": "57a5f6a2f48bdb739d2cbf80e9480694208c8d30beed84f1dd94c03123b7796d1834950adaf8cf77c27151437035a87613f62d453aa2667c04e42f949f9fb106",
      "sourceShard": 1,
      "destinationShard": 1,
      "blockNonce": 1225573,
      "blockHash": "9ce7c413116101432a4541e7587208acd7e139fcf326184eb41b08daaffaaf56",
      "notarizedAtSourceInMetaNonce": 1225798,
      "NotarizedAtSourceInMetaHash": "01d4e84604742ac1b159d46bf1b02f58189ed8de55216a289eded34f72b6afdb",
      "notarizedAtDestinationInMetaNonce": 1225798,
      "notarizedAtDestinationInMetaHash": "01d4e84604742ac1b159d46bf1b02f58189ed8de55216a289eded34f72b6afdb",
      "miniblockType": "TxBlock",
      "miniblockHash": "a1f2fc40b6fc0beb17d8dacab8a726a07cf482cd43e4091b4a02f43f5c3fe592",
      "hyperblockNonce": 1225798,
      "hyperblockHash": "01d4e84604742ac1b159d46bf1b02f58189ed8de55216a289eded34f72b6afdb",
      "timestamp": 1701360704,
      "logs": {
        "address": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
        "events": [
          {
            "address": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
            "identifier": "MultiESDTNFTTransfer",
            "topics": [
              "QVRTLTRjMDkyMA==",
              "",
              "iscjBInoAAA=",
              "AAAAAAAAAAAFAMQ0nl6EMzEw4rMJViTyRltB1jR4BGM="
            ],
            "data": null,
            "additionalData": null
          },
          {
            "address": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
            "identifier": "signalError",
            "topics": [
              "lq3O+DrwJ9dElYcbuyqGhxlbHdDQXM6Y6WqYvfMRBGM=",
              "RXNkdExvY2FsUm9sZUJ1cm4gaXMgbWlzc2luZyBmb3IgYXNzZXQgdG9rZW4="
            ],
            "data": "QDc1NzM2NTcyMjA2NTcyNzI2Zjcy",
            "additionalData": null
          },
          {
            "address": "erd1j6kua7p67qnaw3y4sudmk25xsuv4k8ws6pwvax8fd2vtmuc3q33s840l87",
            "identifier": "internalVMErrors",
            "topics": [
              "AAAAAAAAAAAFAMQ0nl6EMzEw4rMJViTyRltB1jR4BGM=",
              "ZW50ZXJTdGFrZQ=="
            ],
            "data": "CglydW50aW1lLmdvOjExNzUgW2Vycm9yIHNpZ25hbGxlZCBieSBzbWFydGNvbnRyYWN0XSBbZW50ZXJTdGFrZV0KCXJ1bnRpbWUuZ286MTE3NSBbZXJyb3Igc2lnbmFsbGVkIGJ5IHNtYXJ0Y29udHJhY3RdIFtlbnRlclN0YWtlXQoJcnVudGltZS5nbzoxMTcyIFtFc2R0TG9jYWxSb2xlQnVybiBpcyBtaXNzaW5nIGZvciBhc3NldCB0b2tlbl0=",
            "additionalData": null
          }
        ]
      },
      "status": "success",
      "tokens": [
        "ATS-4c0920"
      ],
      "esdtValues": [
        "10000000000000000000"
      ],
      "receivers": [
        "erd1qqqqqqqqqqqqqpgqcs6fuh5yxvcnpc4np9tzfujxtdqavdrcq33sfl9n6z"
      ],
      "receiversShardIDs": [1],
      "operation": "MultiESDTNFTTransfer",
      "function": "enterStake",
      "initiallyPaidFee": "5269280000000000",
      "fee": "5269280000000000",
      "chainID": "D",
      "version": 1,
      "options": 0
    }
  },
  "error": "",
  "code": "successful"
}
        "#;

        let tx_on_network = serde_json::from_str::<TransactionOnNetworkResponse>(data).unwrap();
        assert!(!tx_on_network.data.unwrap().is_success())
    }
}