
use near_sdk::{Balance, StorageUsage};

use crate::*;

use crate::signature::ed25519::{PublicKey, Signature};

pub(crate) fn refund_extra_storage_deposit(storage_used: StorageUsage, used_balance: Balance) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit()
        .checked_sub(used_balance)
        .expect("not enough attached balance");

    assert!(
        required_cost <= attached_deposit,
        "not enough attached balance {}",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

pub(crate) fn verify(message: Vec<u8>, sign: Vec<u8>, pk: Vec<u8>) {
    let pk = PublicKey::from_slice(&pk).unwrap();
    let sign = Signature::from_slice(&sign).unwrap();
    match pk.verify(message, &sign) {
        Ok(_) => log!("verify ok"),
        Err(_) => panic!("verify error")
    }
}

pub(crate) fn get_hash(campaign: CampaignInput) -> Base58CryptoHash {
    let hash = json!(campaign).to_string();
    print!("{:?}", hash);
    let hash = hash.as_bytes();
    let hash: CryptoHash = env::sha256(hash)[..].try_into().unwrap();
    let hash  = Base58CryptoHash::try_from(hash).unwrap();
    hash
}



#[cfg(test)]
mod test {
    use std::{str::FromStr, convert::TryFrom};

    use near_sdk::{json_types::{U64, U128}, AccountId};

    use crate::campaign::{CampaignInput, DepositInput};

    use super::get_hash;


    #[test]
    fn test() {
        let hash = get_hash(CampaignInput {
            guild_id: "935095654924042240".to_string(),
            role_ids: vec![
                "935096492606251038".to_string()
            ],
            start_time: U64::from(1663162532391),
            end_time: U64::from(1663248753649),
            deposit: DepositInput::FT((AccountId::from_str("nepbottest.token3.bhc8521.testnet").unwrap(), U128::from(3000000000000000000000000))),
            claim_amount: U128::from(1000000000000000000000000)
        });
        print!("{:?}", String::from(&hash));
    }
}