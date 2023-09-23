use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct Trophy {
    pub name: String,
    pub created: String,
    pub info_url: UncheckedUrl,
    pub collection_id: String,
    #[mutable]
    pub donated: Decimal,
    #[mutable]
    pub key_image_url: UncheckedUrl,
}
