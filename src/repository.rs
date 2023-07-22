use crate::donation::donation::Donation;
use crate::donation::generate_url;
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
            admin => updatable_by: [OWNER];
            trophy_minter => updatable_by: [OWNER];
        },
        methods {
            new_donation_component => PUBLIC;
            update_base_path => restrict_to: [admin];
        }
    }

    struct Repository {
        // NFT resource address.
        trophy_resource_manager: ResourceManager,

        // Owner badge is given to the owner of the component.
        owner_badge_manager: ResourceManager,

        // Badge for being able to mint trophies.
        minter_badge_manager: ResourceManager,
    }

    impl Repository {
        pub fn new(base_path: String) -> (Global<Repository>, Bucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Runtime::blueprint_id());

            // Creating an admin badge for the admin role
            let owner_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Owner Badge", locked;
                        "description" => "Repository owner badge", locked;
                    }
                ))
                .mint_initial_supply(1);

            // Creating an admin badge for the admin role
            let minter_badge_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Trophies Minter", locked;
                        "description" => "Grants authorization to mint NFs from repository", locked;
                    }
                ))
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                })
                .withdraw_roles(withdraw_roles! {
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            let trophy_resource_manager = ResourceBuilder::new_ruid_non_fungible::<TrophyData>(OwnerRole::None)
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(owner_badge.resource_address()));
                        metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        "name" => "Backeum Trophies", locked;
                        "description" => "Backeum trophies celebrates the patronage of its holder with donations to individual Backeum creators. A unique symbol of support for the community, it's a vibrant testament to financial encouragement.", locked;
                        "domain" => base_path, updatable;
                    }
                ))
                .withdraw_roles(withdraw_roles!(
                    withdrawer => rule!(require(minter_badge_manager.address()));
                    withdrawer_updater => rule!(deny_all);
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(minter_badge_manager.address()));
                    minter_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(minter_badge_manager.address()) || require(owner_badge.resource_address()));
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();

            let component = Self {
                trophy_resource_manager,
                owner_badge_manager: owner_badge.resource_manager(),
                minter_badge_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                owner_badge.resource_address()
            ))))
            .roles(roles! {
                trophy_minter => rule!(require(minter_badge_manager.address()));
                admin => rule!(require(owner_badge.resource_address()));
            })
            .with_address(address_reservation)
            .globalize();

            (component, owner_badge)
        }

        // new_donation_component sets up a new donation component for a user, and give that contract
        // a mint badge that allows for it to create and update trophies. By going through Repository
        // for instantiation we can ensure that the mint badge is only given to a contract that is
        // made by Backeum.
        pub fn new_donation_component(&mut self) -> (Global<Donation>, Bucket) {
            let mint_badge = self.minter_badge_manager.mint(1);
            Donation::new(self.trophy_resource_manager, mint_badge, "".to_string())
        }

        // update_base_path updates the base path for each trophy.
        pub fn update_base_path(
            &mut self,
            new_base_path: String,
            update_nft_ids: Vec<NonFungibleLocalId>,
        ) {
            self.trophy_resource_manager
                .set_metadata("domain", new_base_path.clone());

            for nft_id in update_nft_ids {
                // Get data from the Trophy data based on NF id.
                let mut data: TrophyData =
                    self.trophy_resource_manager.get_non_fungible_data(&nft_id);

                // Generate new image url.
                data.key_image_url = generate_url(
                    new_base_path.to_string(),
                    data.donated,
                    data.created,
                    nft_id.to_string(),
                    data.user_identity,
                );

                // Update image url.
                self.trophy_resource_manager.update_non_fungible_data(
                    &nft_id,
                    "key_image_url",
                    data.key_image_url,
                );
            }
        }
    }
}
