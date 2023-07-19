use crate::donation::donation::Donation;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone)]
pub(crate) struct TrophyData {
    pub created: String,
    pub user_identity: String,
    #[mutable]
    pub donated: Decimal,
    #[mutable]
    pub key_image_url: String,
}

#[blueprint]
mod repository {
    enable_method_auth! {
        roles {
            trophy_minter => updatable_by: [OWNER];
        },
        methods {
            new_donation_component => PUBLIC;
        }
    }

    struct Repository {
        // NFT resource address.
        trophy_resource_manager: ResourceManager,

        // Badge for being able to mint trophies.
        minter_badge_manager: ResourceManager,

        // Base path, e.g https://backeum.com/nft_image
        base_path: String,
    }

    impl Repository {
        pub fn new(base_path: String) -> (Global<Repository>, Bucket) {
            // Creating an admin badge for the admin role
            let owner_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Owner Badge", locked;
                        "description" => "Backeum trophies owner badge", locked;
                    }
                ))
                .mint_initial_supply(1);

            let package_address = Runtime::package_address();

            // Creating an admin badge for the admin role
            let minter_badge_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Trophies Minter", locked;
                        "description" => "Grants authorization to mint NFTs from Backeum repository", locked;
                    }
                ))
                .mint_roles(mint_roles!{
                    minter => rule!(require(package_of_direct_caller(package_address)));
                    minter_updater => rule!(deny_all);
                })
                .withdraw_roles(withdraw_roles!{
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            let trophy_resource_manager = ResourceBuilder::new_ruid_non_fungible::<TrophyData>(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Backeum Trophies", locked;
                        "description" => "Backeum trophies celebrates the patronage of its holder with donations to individual Backeum creators. A unique symbol of support for the community, it's a vibrant testament to encouragement.", locked;
                    }
                ))
                .withdraw_roles(withdraw_roles!(
                    withdrawer => rule!(require(package_of_direct_caller(package_address)));
                    withdrawer_updater => rule!(deny_all);
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(package_of_direct_caller(package_address)));
                    minter_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(package_of_direct_caller(package_address)));
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();

            let component = Self {
                trophy_resource_manager,
                minter_badge_manager,
                base_path,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles! {
                trophy_minter => rule!(require(minter_badge_manager.address()));
            })
            .globalize();

            (component, owner_badge)
        }

        // new_donation_component sets up a new donation contract for a user, and give that contract
        // a mint badge that allows for it to create and update trophies. By going through Repository
        // for instantiation we can ensure that the mint badge is only given to a contract that is
        // made by Backeum.
        pub fn new_donation_component(&mut self) -> (Global<Donation>, Bucket) {
            let mint_badge = self.minter_badge_manager.mint(1);
            Donation::new(self.trophy_resource_manager, mint_badge, "".to_string())
        }
    }
}
