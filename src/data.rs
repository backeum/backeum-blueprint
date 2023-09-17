use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct TrophyData {
    pub name: String,
    pub created: String,
    pub info_url: String,
    pub collection_id: String,
    #[mutable]
    pub donated: Decimal,
    #[mutable]
    pub key_image_url: String,
}
