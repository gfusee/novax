use std::collections::{BTreeMap, HashMap};
use std::fs::OpenOptions;
use std::future::join;
use std::hash::Hash;
use std::path::Path;
use num_bigint::BigUint;
use futures::future::join_all;
use multiversx_sc_snippets::hex;
use multiversx_sc_snippets::multiversx_sc_scenario::scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext};
use multiversx_sc_snippets::multiversx_sc_scenario::scenario_model::{BytesKey, BytesValue};
use novax::errors::NovaXError;
use serde::{Deserialize, Serialize};
use novax::account::AccountInfos;
use novax::Address;
use novax_token::account::balance::{FetchAllTokens, TokenInfos};
use novax_token::error::token::TokenError;
use crate::errors::mocking::NovaXMockingError;
use crate::gateway::keys::AddressKeys;
use crate::{Account, ScenarioWorld, SetStateStep};
use crate::world::serde::ScenarioWorldInfosJson;

type BalanceAsyncResults = Vec<Result<([u8;32], Vec<TokenInfos>), TokenError>>;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ScenarioWorldInfosEsdtTokenAmount {
    pub token_identifier: String,
    pub nonce: u64,
    pub amount: BigUint,
    pub opt_attributes_expr: Option<Vec<u8>>
}

#[derive(PartialEq, Clone, Debug)]
pub struct ScenarioWorldInfos {
    pub address_keys: HashMap<[u8; 32], AddressKeys>,
    pub address_balances: HashMap<[u8; 32], Vec<ScenarioWorldInfosEsdtTokenAmount>>,
    pub address_infos: Vec<AccountInfos>
}

impl From<ScenarioWorldInfosJson> for ScenarioWorldInfos {
    fn from(value: ScenarioWorldInfosJson) -> Self {
        ScenarioWorldInfos {
            address_keys: convert_hashmap_address_keys_to_bytes(value.address_keys),
            address_balances: convert_hashmap_address_keys_to_bytes(value.address_balances),
            address_infos: value.address_infos,
        }
    }
}

impl ScenarioWorldInfos {
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<ScenarioWorldInfos, NovaXMockingError> {
        let reader = OpenOptions::new()
            .read(true)
            .open(file_path)
            .unwrap();

        let Ok(result) = serde_json::from_reader::<_, ScenarioWorldInfosJson>(reader) else { return Err(NovaXMockingError::UnableToReadInfosFromFile) };

        Ok(result.into())
    }

    pub fn overwrite(&mut self, other: ScenarioWorldInfos) {
        for other_address_infos in other.address_infos {
            let mut found_index: Option<usize> = None;
            for (self_address_infos_index, _) in self.address_infos.iter().enumerate() {
                if self.address_infos[self_address_infos_index].data.account.address == other_address_infos.data.account.address {
                    found_index = Some(self_address_infos_index);
                    break
                }
            }

            if let Some(index) = found_index {
                self.address_infos[index] = other_address_infos
            } else {
                self.address_infos.push(other_address_infos)
            }
        }

        overwrite_hashmap(&mut self.address_keys, other.address_keys);
        overwrite_hashmap(&mut self.address_balances, other.address_balances);
    }

    pub async fn fetch(
        gateway_url: &str,
        addresses: &[Address]
    ) -> Result<ScenarioWorldInfos, NovaXMockingError> {
        let (
            accounts,
            keys,
            balances
        ) = join!(
            get_addresses_infos(gateway_url, addresses),
            get_addresses_keys(gateway_url, addresses),
            get_addresses_balances(gateway_url, addresses)
        ).await;

        let infos = ScenarioWorldInfos {
            address_keys: keys?,
            address_balances: balances?,
            address_infos: accounts?,
        };

        Ok(infos)
    }

    pub fn into_world<RegisterFunction>(self, register: RegisterFunction) -> ScenarioWorld
    where
        RegisterFunction: Fn([u8; 32], &str, &mut ScenarioWorld)
    {
        let mut world = ScenarioWorld::new();

        let all_addresses: Vec<[u8; 32]> = self.address_keys.keys().chain(self.address_balances.keys()).copied().collect();

        let mut set_state_accounts_step = SetStateStep::new();

        let mut registered_codes = vec![];

        for address_bytes in all_addresses {
            let encoded_contract_address_expr = format!("0x{}", hex::encode(address_bytes));

            let mut account = Account::new();

            let balances = self.address_balances
                .iter()
                .find(|e| e.0 == &address_bytes)
                .map(|e| e.1);

            if let Some(balances) = balances {
                for balance in balances {
                    let token_identifier = balance.token_identifier.as_bytes().to_vec();
                    if balance.nonce == 0 {
                        account = account.esdt_balance(token_identifier, &balance.amount)
                    } else {
                        account = account.esdt_nft_balance(token_identifier, balance.nonce, &balance.amount, balance.opt_attributes_expr.clone());
                    }
                }
            }

            let keys = self.address_keys
                .iter()
                .find(|e| e.0 == &address_bytes)
                .map(|e| e.1);

            if let Some(keys) = keys {
                let mut keys_map = BTreeMap::new();
                for key in &keys.data.pairs {
                    let key_expr = format!("0x{}", key.0);
                    let value_expr = format!("0x{}", key.1);
                    let context = InterpreterContext::default();
                    keys_map.insert(BytesKey::interpret_from(key_expr, &context), BytesValue::interpret_from(value_expr, &context));
                }

                account.storage = keys_map;
            }

            let account_infos = self.address_infos
                .iter()
                .find(|e| Address::from_bech32_string(&e.data.account.address).unwrap().to_bytes() == address_bytes);

            if let Some(account_infos) = account_infos {
                if let Some(code) = &account_infos.data.account.code {
                    if let Some(owner) = &account_infos.data.account.owner_address {
                        if !owner.is_empty() {
                            let code_expr = format!("0x{code}");

                            if !registered_codes.contains(code) {
                                register(address_bytes, &code_expr, &mut world);
                                registered_codes.push(code.clone());
                            }

                            let owner_address_expr = format!("address:{owner}");
                            account = account.code(&*code_expr.clone());
                            account = account.owner(&*owner_address_expr);
                        }
                    }
                }
            }

            set_state_accounts_step = set_state_accounts_step.put_account(&*encoded_contract_address_expr, account);
        }

        world.set_state_step(set_state_accounts_step);

        world
    }

    pub fn save_into_file<P: AsRef<Path>>(self, file_path: P) {
        let writer = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .unwrap();

        serde_json::to_writer(writer, &ScenarioWorldInfosJson::from(self)).unwrap();
    }
}

async fn get_addresses_infos(gateway_url: &str, addresses: &[Address]) -> Result<Vec<AccountInfos>, NovaXMockingError> {
    let mut accounts_futures = vec![];
    for address in addresses {
        accounts_futures.push(async {
            AccountInfos::from_address(gateway_url, address).await
        })
    };

    let accounts: Vec<Result<AccountInfos, NovaXError>> = join_all(accounts_futures).await;
    let mut results = vec![];

    for account in accounts {
        results.push(account?)
    }

    Ok(results)
}

async fn get_addresses_keys(gateway_url: &str, addresses: &[Address]) -> Result<HashMap<[u8; 32], AddressKeys>, NovaXMockingError> {
    let mut keys_futures = vec![];
    for address in addresses {
        keys_futures.push(async {
            Ok((address.to_bytes(), AddressKeys::from_gateway(gateway_url, address).await?))
        })
    };

    let keys: Vec<Result<([u8; 32], AddressKeys), NovaXMockingError>> = join_all(keys_futures).await;
    let mut results = HashMap::new();

    for key in keys {
        let key = key?;
        results.insert(key.0, key.1);
    }

    Ok(results)
}

async fn get_addresses_balances(gateway_url: &str, addresses: &[Address]) -> Result<HashMap<[u8; 32], Vec<ScenarioWorldInfosEsdtTokenAmount>>, NovaXMockingError> {
    let mut balances_futures = vec![];
    for address in addresses {
        balances_futures.push(async {
            let infos = address.fetch_all_tokens(gateway_url).await?;

            Ok::<([u8; 32], Vec<TokenInfos>), TokenError>((address.to_bytes(), infos))
        })
    };

    let balances_results: BalanceAsyncResults = join_all(balances_futures).await;
    let mut balances = vec![];
    for balance in balances_results {
        balances.push(balance?)
    }

    let result = balances.into_iter()
        .map(|e| {
            let mut amounts: Vec<ScenarioWorldInfosEsdtTokenAmount> = vec![];
            for infos in e.1 {
                amounts.push(ScenarioWorldInfosEsdtTokenAmount {
                    token_identifier: parse_token_identifier(&infos.token_identifier),
                    nonce: infos.nonce,
                    amount: infos.balance,
                    opt_attributes_expr: infos.attributes.map(|e| e.as_bytes().to_vec())
                })
            }

            (e.0, amounts)
        })
        .collect();

    Ok(result)
}

fn convert_hashmap_address_keys_to_bytes<T>(hashmap: HashMap<String, T>) -> HashMap<[u8; 32], T> {
    let mut result = HashMap::new();
    for (key, value) in hashmap.into_iter() {
        result.insert(Address::from_bech32_string(&key).unwrap().to_bytes(), value);
    }

    result
}

fn overwrite_hashmap<A, B>(original: &mut HashMap<A, B>, new: HashMap<A, B>)
where
    A: Eq + Hash
{
    for (new_key, new_value) in new {
        original.insert(new_key, new_value);
    }
}

fn parse_token_identifier(token_identifier: &str) -> String {
    let mut hyphen_count = 0;
    let mut end_index = None;

    for (index, char) in token_identifier.char_indices() {
        if char == '-' {
            hyphen_count += 1;
            if hyphen_count == 2 {
                end_index = Some(index);
                break;
            }
        }
    }

    match end_index {
        Some(index) => String::from(&token_identifier[..index]),
        None => String::from(token_identifier),
    }
}

#[cfg(test)]
mod tests {
    use crate::world::infos::parse_token_identifier;

    #[test]
    fn test_parse_token_identifier_empty_string() {
        assert_eq!(parse_token_identifier(""), "");
    }

    #[test]
    fn test_parse_token_identifier_no_hyphen() {
        assert_eq!(parse_token_identifier("TEST"), "TEST");
    }

    #[test]
    fn test_parse_token_identifier_one_hyphen() {
        assert_eq!(parse_token_identifier("TEST-abcdef"), "TEST-abcdef");
    }

    #[test]
    fn test_parse_token_identifier_trailing_hyphen() {
        assert_eq!(parse_token_identifier("TEST-abcdef-"), "TEST-abcdef");
    }

    #[test]
    fn test_parse_token_identifier_two_hyphens() {
        assert_eq!(parse_token_identifier("TEST-abcdef-123456"), "TEST-abcdef");
    }

    #[test]
    fn test_parse_token_identifier_more_than_two_hyphens() {
        assert_eq!(parse_token_identifier("TEST-abcdef-123456-abcdef"), "TEST-abcdef");
    }
}