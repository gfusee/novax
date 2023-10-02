use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct TokenProperties {
    pub identifier: String,
    pub name: String,
    pub r#type: String,
    pub owner: String,
    pub minted_value: BigUint,
    pub burnt_value: BigUint,
    pub decimals: u8,
    pub is_paused: bool,
    pub can_upgrade: bool,
    pub can_mint: bool,
    pub can_change_owner: bool,
    pub can_pause: bool,
    pub can_freeze: bool,
    pub can_wipe: bool,
    pub can_add_special_roles: bool,
    pub can_transfer_nft_creation_role: bool,
    pub nft_create_stopped: bool,
    pub wiped_amount: BigUint
}