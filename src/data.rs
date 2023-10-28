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

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct Membership {
    pub name: String,
    pub description: String,
    pub info_url: UncheckedUrl,
    pub user_name: String,
    pub user_slug: String,
    pub creator: NonFungibleGlobalId,
    pub created: String,

    #[mutable]
    pub donated: Decimal,

    #[mutable]
    pub key_image_url: UncheckedUrl,
}

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub struct Creator {
    pub name: String,
    pub description: String,
    pub user_name: String,
    pub user_slug: String,
    pub created: String,

    #[mutable]
    pub funded: Decimal,

    #[mutable]
    pub key_image_url: UncheckedUrl,
}
