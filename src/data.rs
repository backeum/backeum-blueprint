use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct Trophy {
    pub name: String,
    pub created: String,
    pub info_url: UncheckedUrl,
    pub collection_id: String,
    pub key_image_url: UncheckedUrl,

    #[mutable]
    pub donated: Decimal,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct CollectionOwnerBadge {
    pub name: String,
    pub description: String,
}
