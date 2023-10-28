use crate::collection::collection::Collection;
use crate::data::{Creator, Membership, Trophy};
use crate::util::*;
use scrypto::prelude::*;

#[blueprint]
#[types(Trophy, Membership, Creator)]
mod repository {
    use crate::collection::CollectionArg;
    enable_package_royalties! {
        new => Free;
        merge_trophies => Free;
        merge_membership => Free;
        new_collection_component => Usd(5.into());
        new_collection_component_and_badge => Usd(5.into());
        mint_creator_badge => Free;
        redeem_thanks_token => Free;
        close_repository => Free;
    }

    enable_method_auth! {
        roles {
            admin => updatable_by: [OWNER];
        },
        methods {
            new_collection_component => PUBLIC;
            new_collection_component_and_badge => PUBLIC;
            mint_creator_badge => PUBLIC;
            merge_trophies => PUBLIC;
            merge_membership => PUBLIC;
            redeem_thanks_token => PUBLIC;
            close_repository => restrict_to: [admin];
        }
    }

    struct Repository {
        // NFT resource manager.
        trophy_resource_manager: ResourceManager,

        // Thanks token resource manager.
        thanks_token_resource_manager: ResourceManager,

        // Membership resource manager.
        membership_resource_manager: ResourceManager,

        // Collection owner badge resource manager.
        creator_resource_manager: ResourceManager,

        // Badge for being able to mint trophies.
        minter_badge_manager: ResourceManager,

        // The owner badge resource address used to set ownership of sub components.
        repository_owner_access_badge_address: ResourceAddress,

        // Dapp definition address
        dapp_definition_address: GlobalAddress,

        // Closed date for the collection
        closed: Option<UtcDateTime>,
    }

    impl Repository {
        pub fn new(
            base_path: String,
            repository_owner_access_badge_address: ResourceAddress,
            dapp_definition_address: GlobalAddress,
        ) -> Global<Repository> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Repository::blueprint_id());

            // Creating an minter badge for the minter role. This is used to mint trophies both in
            // this blueprint and in the collection blueprint. The minter badge is handed down to
            // the collection blueprint via the factory method new_collection_component.
            let minter_badge_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(
                rule!(require(repository_owner_access_badge_address)),
            ))
            .divisibility(DIVISIBILITY_NONE)
            .metadata(metadata!(
                init {
                    "name" => "Trophies Minter", locked;
                    "description" => "Grants authorization to mint NFs from repository", locked;
                    "tags" => vec!["backeum", "badge"], locked;
                    "icon_url" => UncheckedUrl::of(format!("{}{}", base_path.clone(), "/bucket/assets/wallet-assets/minter.png")), locked;
                    "info_url" => UncheckedUrl::of(base_path.clone()), locked;
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

            // Creating an collection owner badge for the trophy collections. This is used to set
            // ownership of the collection components. The collection owner badge is handed down to
            // the collection blueprint via the factory method new_collection_component.
            let creator_resource_manager = ResourceBuilder::new_ruid_non_fungible_with_registered_type::<Creator>(OwnerRole::Fixed(
                rule!(require(repository_owner_access_badge_address))
            ))
                .metadata(metadata!(
                init {
                    "name" => "Backeum Creator Badges", locked;
                    "description" => "Digital emblem celebrating a creator's crowdfunding journey. It evolves with cumulative donations, embodying progress and community support. It's a symbol of encouragement, encapsulating the artist-backer bond in the digital age.", locked;
                    "icon_url" => UncheckedUrl::of(format!("{}{}", base_path.clone(), "/bucket/assets/wallet-assets/badge.png")), updatable;
                    "tags" => vec!["backeum", "badge"], locked;
                    "info_url" => UncheckedUrl::of(base_path.clone()), locked;
                    "dapp_definition" => dapp_definition_address, locked;
                }
                ))
                .mint_roles(mint_roles! {
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(require(global_caller(component_address)));
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(minter_badge_manager.address()) || require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .create_with_no_initial_supply();

            // Manager for minting trophies for a central collection. This manager will be handed
            // down to collection components together with a minter badge. This allows all
            // collections to mint trophies from the same resource manager.
            let trophy_resource_manager = ResourceBuilder::new_ruid_non_fungible_with_registered_type::<Trophy>(OwnerRole::Fixed(
                rule!(require(repository_owner_access_badge_address))
            ))
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(repository_owner_access_badge_address) || require(global_caller(component_address)));
                        metadata_setter_updater => rule!(require(repository_owner_access_badge_address));
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        "name" => "Backeum Trophies", locked;
                        "description" => "Backeum trophies celebrates the patronage of its holder with donations to individual Backeum creators. A unique symbol of support for the community, it's a vibrant testament to financial encouragement.", locked;
                        "domain" => base_path.clone(), updatable;
                        "icon_url" => UncheckedUrl::of(format!("{}{}", base_path, "/bucket/assets/wallet-assets/trophy.png")), updatable;
                        "tags" => vec!["backeum", "trophy"], locked;
                        "info_url" => UncheckedUrl::of(base_path.clone()), locked;
                        "dapp_definition" => dapp_definition_address, locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(minter_badge_manager.address()) || require(global_caller(component_address)));
                    minter_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(minter_badge_manager.address()) || require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .create_with_no_initial_supply();

            // Thanks token is a fungible token that is used to thank backers. It is minted by
            // backing an NFT collection made by a creator. This manager will be handed down to
            // collection components together with a minter badge. This allows all collections to
            // mint and burn thanks tokens from the same resource manager.
            let thanks_token_resource_manager = ResourceBuilder::new_fungible(OwnerRole::Fixed(
                rule!(require(repository_owner_access_badge_address))
            ))
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(repository_owner_access_badge_address) || require(global_caller(component_address)));
                        metadata_setter_updater => rule!(require(repository_owner_access_badge_address));
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        "name" => "Backeum Thanks Token", locked;
                        "symbol" => "THANKS", locked;
                        "description" => "Earned by supporting creators on Backeum. This token symbolizes creator gratitude and is redeemable for exclusive rewards. Every $THANKS is a nod to your belief in artistry. Join, support, and reap unique benefits!", locked;
                        "icon_url" => UncheckedUrl::of(format!("{}{}", base_path, "/bucket/assets/wallet-assets/thanks-token.png")), updatable;
                        "tags" => vec!["backeum", "token", "redeemable"], locked;
                        "info_url" => UncheckedUrl::of(base_path.clone()), locked;
                        "dapp_definition" => dapp_definition_address, locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(minter_badge_manager.address()) || require(repository_owner_access_badge_address) || require(global_caller(component_address)));
                    minter_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(minter_badge_manager.address()) || require(global_caller(component_address)));
                    burner_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .create_with_no_initial_supply();

            // Manager for minting membership tokens for a central collection. This manager will be
            // handed down to collection components together with a minter badge. This allows all
            // collections to mint membership from the same resource manager.
            let membership_resource_manager = ResourceBuilder::new_ruid_non_fungible_with_registered_type::<Membership>(OwnerRole::Fixed(
                rule!(require(repository_owner_access_badge_address))
            ))
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(repository_owner_access_badge_address) || require(global_caller(component_address)));
                        metadata_setter_updater => rule!(require(repository_owner_access_badge_address));
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        "name" => "Backeum Memberships", locked;
                        "description" => "Backeum Supporter Badge NFT: Proof of backing creators on Backeum. Show your support with this digital emblem, uniting creators and backers in the digital realm.", locked;
                        "icon_url" => UncheckedUrl::of(format!("{}{}", base_path, "/bucket/assets/wallet-assets/membership.png")), updatable;
                        "tags" => vec!["backeum", "membership"], locked;
                        "info_url" => UncheckedUrl::of(base_path.clone()), locked;
                        "dapp_definition" => dapp_definition_address, locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(minter_badge_manager.address()) || require(global_caller(component_address)));
                    minter_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(minter_badge_manager.address()) || require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(require(repository_owner_access_badge_address));
                ))
                .create_with_no_initial_supply();

            Self {
                trophy_resource_manager,
                thanks_token_resource_manager,
                membership_resource_manager,
                creator_resource_manager,
                minter_badge_manager,
                repository_owner_access_badge_address,
                dapp_definition_address,
                closed: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(
                rule!(require(repository_owner_access_badge_address))
            ))
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(repository_owner_access_badge_address));
                    metadata_setter_updater => rule!(require(repository_owner_access_badge_address));
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "dapp_definition" => dapp_definition_address, locked;
                }
            ))
            .roles(roles! {
                admin => rule!(require(repository_owner_access_badge_address));
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
            creator_badge_proof: Proof,
        ) -> Global<Collection> {
            if self.closed.is_some() {
                panic!("This repository is permanently closed.");
            }

            let checked_creator_badge_proof =
                creator_badge_proof.check(self.creator_resource_manager.address());

            let data: Creator = self.creator_resource_manager.get_non_fungible_data(
                &checked_creator_badge_proof
                    .as_non_fungible()
                    .non_fungible_local_id(),
            );

            let minter_badge = self.minter_badge_manager.mint(1);

            Collection::new(CollectionArg {
                trophy_resource_manager: self.trophy_resource_manager,
                thanks_token_resource_manager: self.thanks_token_resource_manager,
                membership_resource_manager: self.membership_resource_manager,
                repository_owner_access_badge_address: self.repository_owner_access_badge_address,
                creator_resource_manager: self.creator_resource_manager,
                creator_badge_proof: checked_creator_badge_proof.clone(),
                minter_badge,
                user_name: data.user_name,
                user_slug: data.user_slug,
                dapp_definition_address: self.dapp_definition_address,
            })
        }

        // new_collection_component_and_badge sets up a new collection component for a user, and
        // give that contract a mint badge that allows for it to create and update trophies. By
        // going through Repository for instantiation we can ensure that the mint badge is only
        // given to a contract that is made by Backeum. This method also returns the collection
        // owner badge that the user can use to gain ownership of the collection.
        pub fn new_collection_component_and_badge(
            &mut self,
            user_name: String,
            user_slug: String,
        ) -> (Global<Collection>, Bucket) {
            if self.closed.is_some() {
                panic!("This repository is permanently closed.");
            }

            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let created = generate_created_string(
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap(),
            );

            let creator_badge = self
                .creator_resource_manager
                .mint_ruid_non_fungible::<Creator>(Creator {
                    name: format!("Backeum Owner Badge: {}", user_name.clone(),),
                    description:
                        "Grants ownership of Backeum collection components and membership badges"
                            .to_string(),
                    funded: dec!(0),
                    user_name: user_name.clone(),
                    user_slug: user_slug.clone(),
                    created: created.clone(),
                    key_image_url: UncheckedUrl::of(generate_creator_url(
                        domain.to_string(),
                        dec!(0),
                        created,
                    )),
                });

            let minter_badge = self.minter_badge_manager.mint(1);

            (
                Collection::new(CollectionArg {
                    trophy_resource_manager: self.trophy_resource_manager,
                    thanks_token_resource_manager: self.thanks_token_resource_manager,
                    membership_resource_manager: self.membership_resource_manager,
                    creator_resource_manager: self.creator_resource_manager,
                    repository_owner_access_badge_address: self
                        .repository_owner_access_badge_address,
                    creator_badge_proof: creator_badge
                        .create_proof_of_all()
                        .check(self.creator_resource_manager.address()),
                    minter_badge,
                    user_name,
                    user_slug,
                    dapp_definition_address: self.dapp_definition_address,
                }),
                creator_badge,
            )
        }

        // Mints a new collection owner badge that the user can use to gain ownership of a
        // collection. Ownership badges are free to mint and burn.
        pub fn mint_creator_badge(&mut self, user_name: String, user_slug: String) -> Bucket {
            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let created = generate_created_string(
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap(),
            );

            self.creator_resource_manager
                .mint_ruid_non_fungible::<Creator>(Creator {
                    name: format!("Backeum Owner Badge: {}", user_name.clone(),),
                    description:
                        "Grants ownership of Backeum collection components and membership badges"
                            .to_string(),
                    funded: dec!(0),
                    user_name: user_name.clone(),
                    user_slug: user_slug.clone(),
                    created: created.clone(),
                    key_image_url: UncheckedUrl::of(generate_creator_url(
                        domain.to_string(),
                        dec!(0),
                        created,
                    )),
                })
        }

        // merge_trophies will take multiple trophies of the same collection id and merge them into
        // one.
        pub fn merge_trophies(&mut self, trophies: Bucket) -> Bucket {
            assert_eq!(
                trophies.resource_address(),
                self.trophy_resource_manager.address(),
                "The given trophies is not the of the same resource type as managed by the repository."
            );

            let non_fungible_bucket = trophies.as_non_fungible();
            let trophies_list = non_fungible_bucket.non_fungibles::<Trophy>();
            let template = trophies_list.first().unwrap().data();
            let mut earliest_created: UtcDateTime =
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap();

            let mut donated = dec!(0);
            for trophy_data in trophies_list.iter() {
                assert_eq!(
                    trophy_data.data().collection_id,
                    template.collection_id,
                    "The given trophies is not the of the same collection id."
                );

                assert_eq!(
                    trophy_data.data().info_url,
                    template.info_url,
                    "The given trophies is not the of the same created date."
                );

                assert_eq!(
                    trophy_data.data().name,
                    template.name,
                    "The given trophies is not the of the same created date."
                );

                println!(
                    "Trophy created: {}",
                    parse_created_string(trophy_data.data().created)
                );

                let trophy_date = parse_created_string(trophy_data.data().created);

                if trophy_date
                    .to_instant()
                    .compare(earliest_created.to_instant(), TimeComparisonOperator::Lt)
                {
                    earliest_created = trophy_date;
                }

                donated += trophy_data.data().donated;
            }

            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let created = generate_created_string(earliest_created);
            let new_trophy_data = Trophy {
                name: template.name,
                info_url: template.info_url,
                collection_id: template.collection_id.clone(),
                created: created.clone(),
                donated,
                key_image_url: UncheckedUrl::of(generate_trophy_url(
                    domain.to_string(),
                    donated,
                    created.clone(),
                    template.collection_id.clone(),
                )),
            };

            // Burn the previous trophies.
            trophies.burn();

            self.trophy_resource_manager
                .mint_ruid_non_fungible(new_trophy_data.clone())
        }

        // merge_membership will take multiple memberships of the same creator and merge them into
        // one.
        pub fn merge_membership(&mut self, memberships: Bucket) -> Bucket {
            assert_eq!(
                memberships.resource_address(),
                self.membership_resource_manager.address(),
                "The given memberships is not the of the same resource type as managed by the repository."
            );

            let non_fungible_bucket = memberships.as_non_fungible();
            let membership_list = non_fungible_bucket.non_fungibles::<Membership>();
            let template = membership_list.first().unwrap().data();
            let mut earliest_created: UtcDateTime =
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap();

            let mut donated = dec!(0);
            for membership_data in membership_list.iter() {
                assert_eq!(
                    membership_data.data().creator,
                    template.creator,
                    "The given trophies is not the of the same collection id."
                );

                assert_eq!(
                    membership_data.data().info_url,
                    template.info_url,
                    "The given trophies is not the of the same created date."
                );

                assert_eq!(
                    membership_data.data().name,
                    template.name,
                    "The given trophies is not the of the same created date."
                );

                let membership_date = parse_created_string(membership_data.data().created);

                if membership_date
                    .to_instant()
                    .compare(earliest_created.to_instant(), TimeComparisonOperator::Lt)
                {
                    earliest_created = membership_date;
                }

                donated += membership_data.data().donated;
            }

            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let created = generate_created_string(earliest_created);
            let new_membership_data = Membership {
                name: template.name,
                description: template.description,
                info_url: template.info_url,
                user_name: template.user_name.clone(),
                user_slug: template.user_slug.clone(),
                creator: template.creator.clone(),
                donated,
                created: created.clone(),
                key_image_url: UncheckedUrl::of(generate_membership_url(
                    domain.to_string(),
                    donated,
                    created.clone(),
                    template.user_slug.clone(),
                )),
            };

            // Burn the previous trophies.
            memberships.burn();

            self.membership_resource_manager
                .mint_ruid_non_fungible(new_membership_data.clone())
        }

        // redeem_thanks_token is a method for the backers to redeem thanks tokens.
        pub fn redeem_thanks_token(&mut self, thanks_token: Bucket) {
            self.thanks_token_resource_manager.burn(thanks_token);
        }

        // close_repository is a method for the repository admin to close the repository
        // permanently. This will prevent any further collections to be made from the repository,
        // and will prevent any further usage of this repository.
        pub fn close_repository(&mut self) {
            if self.closed.is_some() {
                panic!("This repository is permanently closed.");
            }

            self.closed =
                Some(UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap());
        }
    }
}
