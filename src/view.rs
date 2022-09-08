
use crate::*;
use crate::campaign::CampaignInfo;



#[near_bindgen]
impl Contract {

    pub fn get_campaign(&self, hash: Base58CryptoHash) -> Option<CampaignInfo> {
        match self.campaigns.get(&hash) {
            Some(v) => {
                Some(v.get_campaign_info())
            },
            None => None
        }
    }

    pub fn get_campaigns_by_guild(&self, guild_id: String) -> Vec<CampaignInfo> {
        let campaign_hashes = self.campaigns_by_guild.get(&guild_id).unwrap_or_default();
        let mut campaigns = Vec::new();
        for hash in campaign_hashes {
            let campaign = self.campaigns.get(&hash).unwrap();
            campaigns.push(campaign.get_campaign_info());
        }
        campaigns
    }

    pub fn check_is_claimed(&self, user_id: String, hash: Base58CryptoHash) -> bool {
        match self.campaigns.get(&hash) {
            Some(v) => v.check_claimed(user_id),
            None => false
        }
    }

}