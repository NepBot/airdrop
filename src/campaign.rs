
use near_contract_standards::fungible_token::core::ext_ft_core;

use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct CampaignInfo {
    hash: Base58CryptoHash,
    owner_id: AccountId,
    guild_id: String,
    role_ids: Vec<String>,
    deposit: Deposit,
    start_time: U64,
    end_time: U64,
    left_amount: U128,
    total_amount: U128,
    claim_amount: U128,
    claimed_count: u32
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Deposit {
    FT(FT),
    NFT(NFT)
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FT {
    pub contract_id: AccountId,
    pub total_amount: u128,
    pub amount: u128
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NFT {
    pub contract_id: AccountId,
    pub token_ids: HashMap<String, Option<u64>>,
    pub total_amount: u128
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum DepositInput {
    FT((AccountId, U128)),
    NFT((AccountId, U128))
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CampaignInput {
    pub guild_id: String,
    pub role_ids: Vec<String>,
    pub start_time: U64,
    pub end_time: U64,
    pub deposit: DepositInput,
    pub claim_amount: U128
}


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Campaign {
    hash: Base58CryptoHash,
    owner_id: AccountId,
    guild_id: String,
    role_ids: Vec<String>,
    start_time: u64,
    end_time: u64,
    deposit: Deposit,
    claim_amount: u128,
    claimed_accounts: UnorderedSet<String>,
}


impl Campaign {
    pub fn new(campaign: CampaignInput, hash: Base58CryptoHash, owner_id: AccountId) -> Self {
        let deposit = match campaign.deposit {
            DepositInput::FT(v) => Deposit::FT(FT {
                contract_id: v.0,
                amount: 0,
                total_amount: v.1.0
            }),
            DepositInput::NFT(v) => Deposit::NFT(NFT {
                contract_id: v.0,
                token_ids: HashMap::new(),
                total_amount: v.1.0
            }),
        };
        Self {
            hash,
            owner_id: owner_id,
            guild_id: campaign.guild_id,
            role_ids: campaign.role_ids,
            start_time: campaign.start_time.0,
            end_time: campaign.end_time.0,
            deposit,
            claim_amount: campaign.claim_amount.0,
            claimed_accounts: UnorderedSet::new(hash.try_to_vec().unwrap()),
        }
    }

    pub fn claim(&mut self, user_id: String) -> Result<(), String> {

        if self.claimed_accounts.contains(&user_id) {
            return Err("Already claimed".into())
        }

        if env::block_timestamp() < self.start_time {
            return Err("not start yet".into())
        }

        if env::block_timestamp() > self.end_time {
            return Err("closed".into())
        }
        
        let sender_id = env::predecessor_account_id();
        match &self.deposit {
            Deposit::FT(v) => {
                if v.amount / self.claim_amount <= self.claimed_accounts.len() as u128 {
                    return Err("closed".into())
                }
                ext_ft_core::ext(v.contract_id.clone()).with_attached_deposit(1).with_unused_gas_weight(6).ft_transfer(sender_id.clone(), self.claim_amount.into(), None).then(
                    Contract::ext(env::current_account_id()).on_claim(self.hash, user_id)
                );
            }
            Deposit::NFT(v) => {

            },
        };
        Ok(())
    }

    pub fn on_claim(&mut self, user_id: String) {
        self.claimed_accounts.insert(&user_id);
    }

    pub fn redeem(&mut self) -> Result<(), String> {

        // if self.end_time > env::block_timestamp() {
        //     return Err("still open".into())
        // }

        match &self.deposit {
            Deposit::FT(v) => {
                let amount = v.amount - self.claimed_accounts.len() as u128 * self.claim_amount;
                ext_ft_core::ext(v.contract_id.clone()).with_attached_deposit(1).with_unused_gas_weight(6).ft_transfer(self.owner_id.clone(), amount.into(), None).then(
                    Contract::ext(env::current_account_id()).on_redeem(self.hash)
                );
            }
            Deposit::NFT(v) => {
                unimplemented!()
            },
        }
        Ok(())
    }

    pub fn get_left_amount(&self) -> u128 {
        match &self.deposit {
            Deposit::FT(v) => v.amount,
            Deposit::NFT(v) => v.token_ids.len() as u128,
        }
    }

    pub fn get_mut_deposit(&mut self) -> &mut Deposit {
        &mut self.deposit
    }

    pub fn get_total_amount(&self) -> u128 {
        match &self.deposit {
            Deposit::FT(v) => v.total_amount,
            Deposit::NFT(v) => v.total_amount,
        }
    }

    pub fn get_campaign_info(&self) -> CampaignInfo {
        CampaignInfo {
            hash: self.hash,
            owner_id: self.owner_id.clone(),
            guild_id: self.guild_id.clone(),
            role_ids: self.role_ids.clone(),
            deposit: self.deposit.clone(),
            start_time: self.start_time.into(),
            end_time: self.end_time.into(),
            left_amount: self.get_left_amount().into(),
            total_amount: self.get_total_amount().into(),
            claim_amount: self.claim_amount.into(),
            claimed_count: self.claimed_accounts.len() as u32,
        }
    }

    pub fn check_claimed(&self, user_id: String) -> bool {
        self.claimed_accounts.contains(&user_id)
    }
}