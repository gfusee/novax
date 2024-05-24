use num_bigint::BigUint;
use novax_data::Address;
use crate::{ExecutorError, TokenTransfer};
use crate::error::transaction::TransactionError;

#[derive(Clone, PartialEq, Debug)]
pub struct NormalizationInOut {
    pub sender: String,
    pub receiver: String,
    pub function_name: Option<String>,
    pub arguments: Vec<Vec<u8>>,
    pub egld_value: BigUint,
    pub esdt_transfers: Vec<TokenTransfer>
}

impl NormalizationInOut {
    pub fn normalize(mut self) -> Result<NormalizationInOut, ExecutorError> {
        let esdt_transfers_len = self.esdt_transfers.len();

        if esdt_transfers_len > 0 && self.egld_value > BigUint::from(0u8) {
            return Err(TransactionError::EgldAndEsdtPaymentsDetected.into());
        }

        let result = if esdt_transfers_len == 0 {
            self
        } else if esdt_transfers_len == 1 {
            let transfer = self.esdt_transfers.remove(0);
            let is_fungible = transfer.nonce == 0;
            let (encoded_identifier, encoded_nonce, encoded_amount) = encode_transfer(transfer)?;


            let (receiver, function_name, mut built_in_args) = if is_fungible {
                let function_name = "ESDTTransfer".to_string();
                let built_in_args = vec![
                    encoded_identifier,
                    encoded_amount
                ];

                (self.receiver, Some(function_name), built_in_args)
            } else {
                let function_name = "ESDTNFTTransfer".to_string();
                let built_in_args = vec![
                    encoded_identifier,
                    encoded_nonce,
                    encoded_amount,
                    Address::from_bech32_string(&self.receiver)?.to_bytes().to_vec(),
                ];

                (self.sender.clone(), Some(function_name), built_in_args)
            };

            if let Some(function_name) = self.function_name {
                let encoded_function_name = encode_string(&function_name)?;

                built_in_args.push(encoded_function_name);
            }

            let mut args = built_in_args;

            args.append(&mut self.arguments);

            NormalizationInOut {
                sender: self.sender,
                receiver,
                function_name,
                arguments: args,
                egld_value: BigUint::from(0u8),
                esdt_transfers: vec![],
            }
        } else {
            let mut built_in_args: Vec<Vec<u8>> = vec![
                Address::from_bech32_string(&self.receiver)?.to_bytes().to_vec(),
                encode_u64(esdt_transfers_len as u64)
            ];

            for transfer in self.esdt_transfers {
                let (encoded_identifier, encoded_nonce, encoded_amount) = encode_transfer(transfer)?;

                built_in_args.push(encoded_identifier);
                built_in_args.push(encoded_nonce);
                built_in_args.push(encoded_amount);
            }

            if let Some(function_name) = self.function_name {
                let encoded_function_name = encode_string(&function_name)?;

                built_in_args.push(encoded_function_name);
            }

            let mut args = built_in_args;
            args.append(&mut self.arguments);

            NormalizationInOut {
                sender: self.sender.clone(),
                receiver: self.sender,
                function_name: Some("MultiESDTNFTTransfer".to_string()),
                arguments: args,
                egld_value: BigUint::from(0u8),
                esdt_transfers: vec![],
            }
        };

        Ok(result)
    }

    pub fn get_transaction_data(self) -> String {
        let mut args_string = vec![];

        if let Some(function_name) = self.function_name {
            args_string.push(function_name)
        }

        for arg in self.arguments {
            args_string.push(hex::encode(arg));
        }

        args_string.join("@")
    }
}

fn encode_string(string: &str) -> Result<Vec<u8>, ExecutorError> {
    hex::decode(hex::encode(string))
        .map_err(|_| TransactionError::CannotEncodeString { string: string.to_string()  }.into())
}

fn encode_u64(value: u64) -> Vec<u8> {
        let mut bytes = value.to_be_bytes().to_vec();

        while bytes.first() == Some(&0u8) {
            bytes = bytes[1..].to_vec();
        }

        bytes
}

fn encode_transfer(token_transfer: TokenTransfer) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), ExecutorError> {
    let encoded_identifier = encode_string(&token_transfer.identifier)
        .map_err(|_| TransactionError::CannotEncodeTransfer)?;

    let encoded_nonce = encode_u64(token_transfer.nonce);

    let encoded_amount = hex::decode(hex::encode(token_transfer.amount.to_bytes_be()))
        .map_err(|_| TransactionError::CannotEncodeTransfer)?;

    Ok((encoded_identifier, encoded_nonce, encoded_amount))
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use crate::error::transaction::TransactionError;
    use crate::{ExecutorError, TokenTransfer};
    use crate::utils::transaction::normalization::NormalizationInOut;

    const SENDER: &str = "erd1h4uhy73dev6qrfj7wxsguapzs8632mfwqjswjpsj6kzm2jfrnslqsuduqu";

    const RECEIVER: &str = "erd1qqqqqqqqqqqqqpgq9wmk04e90fkhcuzns0pgwm33sdtxze346vpsq0ka9p";
    const RECEIVER_HEX: &str = "000000000000000005002bb767d7257a6d7c705383c2876e318356616635d303";

    const ENDPOINT_NAME: &str = "myEndpoint";
    const ENDPOINT_NAME_HEX: &str = "6d79456e64706f696e74";

    const FUNGIBLE_NAME: &str = "WEGLD-abcdef";
    const FUNGIBLE_NAME_HEX: &str = "5745474c442d616263646566";

    const NON_FUNGIBLE_NAME: &str = "SFT-abcdef";
    const NON_FUNGIBLE_NAME_HEX: &str = "5346542d616263646566";

    #[test]
    fn test_normalize_no_payment() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![
                vec![1, 2],
                vec![3, 4]
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![],
        };

        let result = value.clone().normalize().unwrap();
        let result_data = result.clone().get_transaction_data();

        let expected = value;
        let expected_data = "myEndpoint@0102@0304";

        assert_eq!(result, expected);
        assert_eq!(result_data, expected_data);
    }

    #[test]
    fn test_normalize_egld_payment() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![
                vec![1, 2],
                vec![3, 4]
            ],
            egld_value: BigUint::from(10u8),
            esdt_transfers: vec![],
        };

        let result = value.clone().normalize().unwrap();
        let result_data = result.clone().get_transaction_data();

        let expected = value;
        let expected_data = "myEndpoint@0102@0304";

        assert_eq!(result, expected);
        assert_eq!(result_data, expected_data)
    }

    #[test]
    fn test_normalize_single_fungible_payment_no_arg() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![
                TokenTransfer {
                    identifier: FUNGIBLE_NAME.to_string(),
                    nonce: 0,
                    amount: BigUint::from(100u8),
                }
            ]
        };

        let result = value.normalize().unwrap();
        let result_data = result.clone().get_transaction_data();

        let expected = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: "ESDTTransfer".to_string(),
            arguments: vec![
                hex::decode(FUNGIBLE_NAME_HEX).unwrap(),
                vec![100],
                hex::decode(ENDPOINT_NAME_HEX).unwrap()
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![]
        };

        let expected_data = format!("ESDTTransfer@{FUNGIBLE_NAME_HEX}@64@{ENDPOINT_NAME_HEX}");

        assert_eq!(result, expected);
        assert_eq!(result_data, expected_data)
    }

    #[test]
    fn test_normalize_single_fungible_payment_with_args() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![
                vec![1, 2],
                vec![3, 4]
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![
                TokenTransfer {
                    identifier: FUNGIBLE_NAME.to_string(),
                    nonce: 0,
                    amount: BigUint::from(100u8),
                }
            ]
        };

        let result = value.normalize().unwrap();
        let result_data = result.clone().get_transaction_data();

        let expected = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: "ESDTTransfer".to_string(),
            arguments: vec![
                hex::decode(FUNGIBLE_NAME_HEX).unwrap(),
                vec![100],
                hex::decode(ENDPOINT_NAME_HEX).unwrap(),
                vec![1, 2],
                vec![3, 4]
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![]
        };

        let expected_data = format!("ESDTTransfer@{FUNGIBLE_NAME_HEX}@64@{ENDPOINT_NAME_HEX}@0102@0304");

        assert_eq!(result, expected);
        assert_eq!(result_data, expected_data)
    }

    #[test]
    fn test_normalize_single_non_fungible_payment_no_arg() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![
                TokenTransfer {
                    identifier: NON_FUNGIBLE_NAME.to_string(),
                    nonce: 1,
                    amount: BigUint::from(100u8),
                }
            ]
        };

        let result = value.normalize().unwrap();
        let result_data = result.clone().get_transaction_data();

        let expected = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: SENDER.to_string(),
            function_name: "ESDTNFTTransfer".to_string(),
            arguments: vec![
                hex::decode(NON_FUNGIBLE_NAME_HEX).unwrap(),
                vec![1],
                vec![100],
                hex::decode(RECEIVER_HEX).unwrap(),
                hex::decode(ENDPOINT_NAME_HEX).unwrap()
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![]
        };
        let expected_data = format!("ESDTNFTTransfer@{NON_FUNGIBLE_NAME_HEX}@01@64@{RECEIVER_HEX}@{ENDPOINT_NAME_HEX}");

        assert_eq!(result, expected);
        assert_eq!(result_data, expected_data)
    }

    #[test]
    fn test_normalize_single_non_fungible_payment_with_args() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![
                vec![1, 2],
                vec![3, 4]
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![
                TokenTransfer {
                    identifier: NON_FUNGIBLE_NAME.to_string(),
                    nonce: 1,
                    amount: BigUint::from(100u8),
                }
            ]
        };

        let result = value.normalize().unwrap();
        let result_data = result.clone().get_transaction_data();

        let expected = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: SENDER.to_string(),
            function_name: "ESDTNFTTransfer".to_string(),
            arguments: vec![
                hex::decode(NON_FUNGIBLE_NAME_HEX).unwrap(),
                vec![1],
                vec![100],
                hex::decode(RECEIVER_HEX).unwrap(),
                hex::decode(ENDPOINT_NAME_HEX).unwrap(),
                vec![1, 2],
                vec![3, 4]
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![]
        };

        let expected_data = format!("ESDTNFTTransfer@{NON_FUNGIBLE_NAME_HEX}@01@64@{RECEIVER_HEX}@{ENDPOINT_NAME_HEX}@0102@0304");

        assert_eq!(result, expected);
        assert_eq!(result_data, expected_data)
    }

    #[test]
    fn test_normalize_single_multi_payments_no_arg() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![
                TokenTransfer {
                    identifier: FUNGIBLE_NAME.to_string(),
                    nonce: 0,
                    amount: BigUint::from(10u16),
                },
                TokenTransfer {
                    identifier: NON_FUNGIBLE_NAME.to_string(),
                    nonce: 1,
                    amount: BigUint::from(100u8),
                }
            ]
        };

        let result = value.normalize().unwrap();

        let expected = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: SENDER.to_string(),
            function_name: "MultiESDTNFTTransfer".to_string(),
            arguments: vec![
                hex::decode(RECEIVER_HEX).unwrap(),
                vec![2],
                hex::decode(FUNGIBLE_NAME_HEX).unwrap(),
                vec![],
                vec![10],
                hex::decode(NON_FUNGIBLE_NAME_HEX).unwrap(),
                vec![1],
                vec![100],
                hex::decode(ENDPOINT_NAME_HEX).unwrap()
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![]
        };

        let expected_data = format!("MultiESDTNFTTransfer@{RECEIVER_HEX}@{FUNGIBLE_NAME_HEX}@@0a@{NON_FUNGIBLE_NAME_HEX}@01@64@@{ENDPOINT_NAME_HEX}");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_normalize_single_multi_payments_with_args() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![
                vec![1, 2],
                vec![3, 4]
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![
                TokenTransfer {
                    identifier: FUNGIBLE_NAME.to_string(),
                    nonce: 0,
                    amount: BigUint::from(10u16),
                },
                TokenTransfer {
                    identifier: NON_FUNGIBLE_NAME.to_string(),
                    nonce: 1,
                    amount: BigUint::from(100u8),
                }
            ]
        };

        let result = value.normalize().unwrap();
        let result_data = result.clone().get_transaction_data();

        let expected = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: SENDER.to_string(),
            function_name: "MultiESDTNFTTransfer".to_string(),
            arguments: vec![
                hex::decode(RECEIVER_HEX).unwrap(),
                vec![2],
                hex::decode(FUNGIBLE_NAME_HEX).unwrap(),
                vec![],
                vec![10],
                hex::decode(NON_FUNGIBLE_NAME_HEX).unwrap(),
                vec![1],
                vec![100],
                hex::decode(ENDPOINT_NAME_HEX).unwrap(),
                vec![1, 2],
                vec![3, 4]
            ],
            egld_value: BigUint::from(0u8),
            esdt_transfers: vec![]
        };

        let expected_data = format!("MultiESDTNFTTransfer@{RECEIVER_HEX}@{FUNGIBLE_NAME_HEX}@@0a@{NON_FUNGIBLE_NAME_HEX}@01@64@@{ENDPOINT_NAME_HEX}@0102@0304");

        assert_eq!(result, expected);    }

    #[test]
    fn test_normalize_esdt_and_egld_payment() {
        let value = NormalizationInOut {
            sender: SENDER.to_string(),
            receiver: RECEIVER.to_string(),
            function_name: ENDPOINT_NAME.to_string(),
            arguments: vec![],
            egld_value: BigUint::from(1u8),
            esdt_transfers: vec![
                TokenTransfer {
                    identifier: FUNGIBLE_NAME.to_string(),
                    nonce: 0,
                    amount: BigUint::from(100u8),
                }
            ]
        };

        let result = value.normalize();

        let expected: Result<NormalizationInOut, ExecutorError> = Err(TransactionError::EgldAndEsdtPaymentsDetected.into());

        assert_eq!(result, expected)
    }
}