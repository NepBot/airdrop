use near_sdk::serde_json;
use near_contract_standards::non_fungible_token::{TokenId, core::NonFungibleTokenReceiver, approval::NonFungibleTokenApprovalReceiver};

use crate::*;


#[near_bindgen]
impl Contract {

    #[private]
    pub fn on_claim(&mut self, hash: Base58CryptoHash, user_id: String) {
        for i in 0..env::promise_results_count() {
            match env::promise_result(i) {
                PromiseResult::Successful(_) => {
                    continue
                },
                _ => panic!("failed to transfer token")
            }
        }
        let mut campaign = self.campaigns.get(&hash).unwrap();
        campaign.on_claim(user_id);
        self.campaigns.insert(&hash, &campaign);
    }

    #[private]
    pub fn on_redeem(&mut self, hash: Base58CryptoHash) {
        for i in 0..env::promise_results_count() {
            match env::promise_result(i) {
                PromiseResult::Successful(_) => {
                    continue
                },
                _ => panic!("failed to transfer token")
            }
        }
        let mut campaign = self.campaigns.get(&hash).unwrap();
        let deposit = campaign.get_mut_deposit();
        match deposit {
            Deposit::FT(v) => v.amount = 0,
            Deposit::NFT(v) => v.token_ids = HashMap::new()
        };
        self.campaigns.insert(&hash, &campaign);
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let hash = Base58CryptoHash::try_from(msg).unwrap();
        let mut campaign = self.campaigns.get(&hash).unwrap();
        let mut deposit = campaign.get_mut_deposit();
        match deposit {
            Deposit::FT(v) => v.amount += amount.0,
            Deposit::NFT(_) => unimplemented!()
        };
        self.campaigns.insert(&hash, &campaign);
        PromiseOrValue::Value(0.into())
    }
}

#[near_bindgen]
impl NonFungibleTokenApprovalReceiver for Contract {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg:String
    ) -> near_sdk::PromiseOrValue<String>  {
        unimplemented!()
    }

}