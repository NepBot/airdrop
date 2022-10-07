/**
 *  write by Jimmy Bai
 *  Sep 7 2022
 * 
 * 
*/




use campaign::{Campaign, CampaignInput, DepositInput};
use near_contract_standards::fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::log;
use near_sdk::serde_json::{json, from_slice};
use near_sdk::{AccountId, Balance, PromiseOrValue, PublicKey, env, near_bindgen, setup_alloc, ext_contract, log, CryptoHash, bs58};
use near_sdk::collections::{LookupMap, UnorderedMap, Vector, UnorderedSet};
use utils::get_hash;
use std::collections::HashMap;
use std::convert::{TryInto, TryFrom};
use std::fmt::Debug;
use std::hash::Hash;
use std::panic::{panic_any, self};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::{BlockHeight, Gas, PanicOnDefault, Promise, PromiseResult};
use near_sdk::json_types::{Base58PublicKey, U128, U64, ValidAccountId, Base58CryptoHash};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

use crate::utils::{refund_extra_storage_deposit, verify};
use crate::campaign::Deposit;

pub mod view;
pub mod utils;
pub mod campaign;
pub mod signature;
pub mod resolver;


const CROSS_CONTRACT_CALL_GAS: u64 = 20_000_000_000_000;


#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner_id: AccountId,
    public_key: String,
    campaigns: UnorderedMap<Base58CryptoHash, Campaign>,
    campaigns_by_guild: LookupMap<String, Vec<Base58CryptoHash>>,
    account_storage: u128,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(public_key: String) -> Self {
        let this = Self {
            owner_id: env::predecessor_account_id(),
            public_key,
            campaigns: UnorderedMap::new(b's'),
            campaigns_by_guild: LookupMap::new(b'g'),
            account_storage: 128
        };
        this
    }

    #[payable]
    pub fn add_campaign(&mut self, campaign: CampaignInput) {
        let initial_storage_usage = env::storage_usage();
        let sender_id = env::predecessor_account_id();
        match &campaign.deposit {
            DepositInput::FT(v) => assert!(v.1.0 >= campaign.claim_amount.0, "not enough token"),
            DepositInput::NFT(v) => unimplemented!(),
        }
        let hash = get_hash(campaign.clone());
        self.campaigns.insert(&hash, &Campaign::new(campaign.clone(), hash, sender_id));
        let mut guild_campaigns = self.campaigns_by_guild.get(&campaign.guild_id).unwrap_or_default();
        guild_campaigns.push(hash);
        self.campaigns_by_guild.insert(&campaign.guild_id, &guild_campaigns);
        refund_extra_storage_deposit(env::storage_usage() - initial_storage_usage, 0);
    }

    pub fn redeem(&mut self, hash: Base58CryptoHash) {
        let mut campaign = self.campaigns.get(&hash).unwrap();
        match campaign.redeem() {
            Ok(_) => {},
            Err(e) => panic!("{:?}", e)
        };
        self.campaigns.insert(&hash, &campaign);
    }

    #[payable]
    pub fn claim(&mut self, hash: Base58CryptoHash, user_id: String, timestamp: U64, sign: String) {
        let initial_storage_usage = env::storage_usage();
        let timestamp = u64::from(timestamp);
        assert!(env::block_timestamp() - timestamp < 120_000_000_000, "signature expired");
        let sign: Vec<u8> = bs58::decode(sign).into_vec().unwrap();
        let pk: Vec<u8> = bs58::decode(self.public_key.clone()).into_vec().unwrap();
        let json = json!(String::from(&hash) + &user_id + &timestamp.to_string()).to_string();
        verify(json.into_bytes(), sign.into(), pk.into());

        refund_extra_storage_deposit(env::storage_usage() - initial_storage_usage, 0);
        let mut campaign = self.campaigns.get(&hash).unwrap();
        match campaign.claim(user_id) {
            Ok(_) => {},
            Err(e) => panic!("{:?}", e)
        }
        self.campaigns.insert(&hash, &campaign);
        
    }

}