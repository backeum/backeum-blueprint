use crate::collection::collection::Collection;
use crate::collection::generate_url;
use crate::data::TrophyData;
use scrypto::prelude::*;

#[blueprint]
mod repository {
    enable_package_royalties! {
        new => Free;
        new_collection_component => Xrd(20.into());
        update_base_path => Free;
        update_royalty_amount => Free;
    }

    enable_method_auth! {
        roles {
            trophy_minter => updatable_by: [OWNER];
        },
        methods {
            new_collection_component => PUBLIC;
            update_base_path => restrict_to: [OWNER];
            update_royalty_amount => restrict_to: [OWNER];
        }
    }

    struct Repository {
        // NFT resource address.
        trophy_resource_manager: ResourceManager,

        // Badge for being able to mint trophies.
        minter_badge_manager: ResourceManager,

        // The owner badge resource address used to set ownership of sub components.
        owner_badge_resource_address: ResourceAddress,

        // Set the royalty amount on new collections.
        royalty_amount: Decimal,
    }

    impl Repository {
        pub fn new(base_path: String, owner_badge: ResourceAddress) -> Global<Repository> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Repository::blueprint_id());

            // Setup owner badge access rule
            let owner_badge_access_rule: AccessRule = rule!(require(owner_badge));

            // Creating an admin badge for the admin role
            let minter_badge_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(
                owner_badge_access_rule.clone(),
            ))
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

            let trophy_resource_manager = ResourceBuilder::new_ruid_non_fungible::<TrophyData>(OwnerRole::Fixed(owner_badge_access_rule.clone()))
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(global_caller(component_address)));
                        metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        "name" => "Backeum Trophies", locked;
                        "description" => "Backeum trophies celebrates the patronage of its holder with donations to individual Backeum creators. A unique symbol of support for the community, it's a vibrant testament to financial encouragement.", locked;
                        "domain" => format!("{}", base_path), updatable;
                        "tags" => vec!["backeum", "trophy"], locked;
                        "info_url" => format!("{}", base_path), locked;
                    }
                ))
                .withdraw_roles(withdraw_roles!(
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => rule!(require(global_caller(component_address)));
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(minter_badge_manager.address()));
                    minter_updater => rule!(require(global_caller(component_address)));
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(minter_badge_manager.address()) || require(global_caller(component_address)));
                    non_fungible_data_updater_updater => owner_badge_access_rule.clone();
                ))
                .create_with_no_initial_supply();

            Self {
                trophy_resource_manager,
                minter_badge_manager,
                owner_badge_resource_address: owner_badge,
                royalty_amount: dec!(15),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(owner_badge_access_rule.clone()))
            .roles(roles! {
                trophy_minter => rule!(require(minter_badge_manager.address()));
            })
            .with_address(address_reservation)
            .globalize()
        }

        // new_collection_component sets up a new collection component for a user, and give that contract
        // a mint badge that allows for it to create and update trophies. By going through Repository
        // for instantiation we can ensure that the mint badge is only given to a contract that is
        // made by Backeum.
        pub fn new_collection_component(
            &mut self,
            user_identity: String,
            collection_id: String,
        ) -> (Global<Collection>, Bucket) {
            let mint_badge = self.minter_badge_manager.mint(1);
            Collection::new(
                self.trophy_resource_manager,
                self.owner_badge_resource_address,
                self.royalty_amount,
                mint_badge,
                user_identity,
                collection_id,
            )
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
                    data.collection_id,
                );

                // Update image url.
                self.trophy_resource_manager.update_non_fungible_data(
                    &nft_id,
                    "key_image_url",
                    data.key_image_url,
                );
            }
        }

        // update_royalty_amount updates the royalty amount for each new collection.
        pub fn update_royalty_amount(&mut self, new_royalty_amount: Decimal) {
            self.royalty_amount = new_royalty_amount;
        }
    }
}
