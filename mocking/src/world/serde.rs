use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use novax::account::AccountInfos;
use novax::Address;
use crate::gateway::keys::AddressKeys;
use crate::world::infos::{ScenarioWorldInfosEsdtTokenAmount, ScenarioWorldInfos};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub(crate) struct ScenarioWorldInfosJson {
    pub(crate) address_keys: HashMap<String, AddressKeys>,
    pub(crate) address_balances: HashMap<String, Vec<ScenarioWorldInfosEsdtTokenAmount>>,
    pub(crate) address_infos: Vec<AccountInfos>
}

impl From<ScenarioWorldInfos> for ScenarioWorldInfosJson {
    fn from(value: ScenarioWorldInfos) -> Self {
        ScenarioWorldInfosJson {
            address_keys: convert_hashmap_address_keys_to_bech32(value.address_keys),
            address_balances: convert_hashmap_address_keys_to_bech32(value.address_balances),
            address_infos: value.address_infos,
        }
    }
}

fn convert_hashmap_address_keys_to_bech32<T>(hashmap: HashMap<[u8; 32], T>) -> HashMap<String, T> {
    let mut result = HashMap::new();
    for (key, value) in hashmap.into_iter() {
        result.insert(Address::from_bytes(key).to_bech32_string().unwrap(), value);
    }

    result
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use num_bigint::BigUint;
    use novax::account::{AccountInfos, AccountInfosAccountData, AccountInfosData};
    use novax::Address;
    use crate::gateway::keys::{AddressKeys, AddressKeysData};
    use crate::world::serde::{convert_hashmap_address_keys_to_bech32, ScenarioWorldInfosJson};
    use crate::world::infos::{ScenarioWorldInfosEsdtTokenAmount, ScenarioWorldInfos};

    #[test]
    fn test_from_scenario_world_infos_impl() {
        let mut address_keys = HashMap::new();
        let first_address = "erd1devnet6uy8xjusvusfy3q83qadfhwrtty5fwa8ceh9cl60q2p6ysra7aaa";
        let first_address_bytes = Address::from_bech32_string(first_address).unwrap().to_bytes();
        let mut first_address_keys_pairs: HashMap<String, String> = HashMap::new();
        first_address_keys_pairs.insert("key1".to_string(), "value1".to_string());
        first_address_keys_pairs.insert("key2".to_string(), "value2".to_string());
        let first_address_key = AddressKeys {
            data: AddressKeysData {
                pairs: first_address_keys_pairs,
            },
            error: "".to_string(),
            code: "11111".to_string(),
        };
        address_keys.insert(
            first_address_bytes,
            first_address_key
        );

        let mut address_balances = HashMap::new();
        address_balances.insert(
            first_address_bytes,
            vec![
                ScenarioWorldInfosEsdtTokenAmount {
                    token_identifier: "WEGLD-abcdef".to_string(),
                    nonce: 0,
                    amount: BigUint::from(10u8),
                    opt_attributes: Some(vec![0, 1]),
                },
                ScenarioWorldInfosEsdtTokenAmount {
                    token_identifier: "NFT-abcdef-9a".to_string(),
                    nonce: 0,
                    amount: BigUint::from(10u8),
                    opt_attributes: Some(vec![0, 1]),
                }
            ]
        );

        let address_infos = vec![
            AccountInfos {
                data: AccountInfosData {
                    account: AccountInfosAccountData {
                        address: first_address.to_string(),
                        nonce: 10,
                        balance: "1".to_string(),
                        code: Some("code".to_string()),
                        owner_address: Some(first_address.to_string()),
                    },
                },
            }
        ];

        let scenario_world = ScenarioWorldInfos {
            address_keys: address_keys.clone(),
            address_balances: address_balances.clone(),
            address_infos: address_infos.clone()
        };

        let result = ScenarioWorldInfosJson::from(scenario_world);

        let expected_address_keys = convert_hashmap_address_keys_to_bech32(address_keys);
        let expected_address_balances = convert_hashmap_address_keys_to_bech32(address_balances);
        let expected_address_infos = address_infos;

        let expected_result = ScenarioWorldInfosJson {
            address_keys: expected_address_keys,
            address_balances: expected_address_balances,
            address_infos: expected_address_infos,
        };

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_convert_hashmap_empty() {
        let input: HashMap<[u8; 32], u64> = HashMap::new();
        let result = convert_hashmap_address_keys_to_bech32(input);

        assert!(result.is_empty())
    }

    #[test]
    fn test_convert_hashmap_one_element() {
        let address = "erd1devnet6uy8xjusvusfy3q83qadfhwrtty5fwa8ceh9cl60q2p6ysra7aaa";
        let mut input: HashMap<[u8; 32], u64> = HashMap::new();
        input.insert(
            Address::from_bech32_string(address).unwrap().to_bytes(),
            10
        );
        let result = convert_hashmap_address_keys_to_bech32(input);

        let mut expected: HashMap<String, u64> = HashMap::new();
        expected.insert(address.to_string(), 10);

        assert_eq!(result, expected)
    }

    #[test]
    fn test_convert_hashmap_multiple_elements() {
        let addresses = vec![
            "erd1devnet6uy8xjusvusfy3q83qadfhwrtty5fwa8ceh9cl60q2p6ysra7aaa",
            "erd1qqqqqqqqqqqqqpgqp699jngundfqw07d8jzkepucvpzush6k3wvqyc44rx"
        ];
        let mut input: HashMap<[u8; 32], u64> = HashMap::new();
        for (index, address) in addresses.clone().into_iter().enumerate() {
            input.insert(
                Address::from_bech32_string(address).unwrap().to_bytes(),
                index as u64
            );
        }
        let result = convert_hashmap_address_keys_to_bech32(input);

        let mut expected: HashMap<String, u64> = HashMap::new();
        expected.insert(addresses[0].to_string(), 0);
        expected.insert(addresses[1].to_string(), 1);

        assert_eq!(result, expected)
    }
}