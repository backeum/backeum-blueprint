use crate::data::Trophy;
use crate::util::*;
use scrypto::prelude::*;

#[blueprint]
#[types(Trophy)]
mod collection {
    enable_package_royalties! {
        new => Free;
        donate_mint => Xrd(20.into());
        donate_update => Xrd(20.into());
        withdraw_donations => Free;
        close_collection => Free;
    }

    enable_method_auth! {
        roles {
            owner => updatable_by: [];
        },
        methods {
            donate_mint => PUBLIC;
            donate_update => PUBLIC;
            withdraw_donations => restrict_to: [owner];
            close_collection => restrict_to: [owner];
        }
    }

    struct Collection {
        // Mints a proof that is used as proof of donated value to the NFT repository.
        trophy_resource_manager: ResourceManager,

        // NFT minter badge
        minter_badge: Vault,

        // Collected donations
        donations: Vault,

        // Specific user name that owns this component
        user_name: String,

        // Specific user slug that owns this component
        user_slug: String,

        // Which collection this collection component is for
        collection_id: String,

        // Closed date for the collection
        closed: Option<UtcDateTime>,
    }

    impl Collection {
        pub fn new(
            trophy_resource_manager: ResourceManager,
            repository_owner_badge: ResourceAddress,
            collection_owner_badge: ResourceAddress,
            minter_badge: Bucket,
            user_name: String,
            user_slug: String,
            dapp_definition_address: GlobalAddress,
        ) -> Global<Collection> {
            let (reservation, address) =
                Runtime::allocate_component_address(Collection::blueprint_id());
            let collection_id = Runtime::bech32_encode_address(address);
            Self {
                minter_badge: Vault::with_bucket(minter_badge),
                donations: Vault::new(XRD),
                user_name,
                user_slug,
                collection_id,
                trophy_resource_manager,
                closed: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(repository_owner_badge))))
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(repository_owner_badge));
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "dapp_definition" => dapp_definition_address, locked;
                }
            ))
            .roles(roles!(
                owner => rule!(require(collection_owner_badge));
            ))
            .with_address(reservation)
            .globalize()
        }

        // donate_mint is a public method, callable by anyone who want to donate to the user. In
        // return they will get a trophy NFT that represents the donation.
        pub fn donate_mint(&mut self, tokens: Bucket) -> Bucket {
            if self.closed.is_some() {
                panic!("This collection is permanently closed.");
            }

            // Push a proof of minter badge to the local auth zone for minting a trophy.
            LocalAuthZone::push(self.minter_badge.as_fungible().create_proof_of_amount(1));

            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let created = generate_created_string(
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap(),
            );
            let donated = tokens.amount() + 20;

            // Create the trophy data.
            let data = Trophy {
                name: format!("Backer Trophy: {}", self.user_name),
                info_url: UncheckedUrl::of(format!("{}/p/{}", domain, self.user_slug)),
                collection_id: self.collection_id.clone(),
                created: created.clone(),
                donated,
                key_image_url: UncheckedUrl::of(generate_url(
                    domain.to_string(),
                    donated,
                    created,
                    self.collection_id.clone(),
                )),
            };

            // Mint the trophy NFT.
            let trophy = self
                .trophy_resource_manager
                .mint_ruid_non_fungible(data.clone());

            // Take all tokens, and return trophy.
            self.donations.put(tokens);
            trophy
        }

        // donate is a public method, callable by anyone who want to donate to the user.
        pub fn donate_update(&mut self, tokens: Bucket, proof: Proof) {
            if self.closed.is_some() {
                panic!("This collection is permanently closed.");
            }

            // Push a proof of minter badge to the local auth zone for minting a trophy.
            LocalAuthZone::push(self.minter_badge.as_fungible().create_proof_of_amount(1));

            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            // Check that the proof is of same resource address.
            let checked_proof = proof.check(self.trophy_resource_manager.address());

            // Retrieve the NF id from the proof, use it to update metadata on the NF.
            let nft_id = checked_proof.as_non_fungible().non_fungible_local_id();

            // Get data from the Trophy data based on NF id.
            let mut data: Trophy = self.trophy_resource_manager.get_non_fungible_data(&nft_id);

            // Check whether the NF user_identity is owned by this component.
            assert_eq!(
                data.collection_id, self.collection_id,
                "The given trophy does match the collection id of this component."
            );

            // Generate new data based on the updated donation value.
            data.donated += tokens.amount();
            data.donated += 20;
            data.key_image_url = UncheckedUrl::of(generate_url(
                domain.to_string(),
                data.donated,
                data.created,
                self.collection_id.clone(),
            ));

            // Update NF with new data
            self.trophy_resource_manager
                .update_non_fungible_data(&nft_id, "donated", data.donated);
            self.trophy_resource_manager.update_non_fungible_data(
                &nft_id,
                "key_image_url",
                data.key_image_url,
            );

            // Take all tokens, and return trophy.
            self.donations.put(tokens);
        }

        // withdraw_donations is a method for the admin to withdraw all donations.
        pub fn withdraw_donations(&mut self) -> Bucket {
            self.donations.take_all()
        }

        // close_collection is a method for the collection admin to close the collection
        // permanently. This will prevent any further donations to be made to the collection, and
        // will prevent any further minting or updating to the trophies.
        pub fn close_collection(&mut self) {
            self.closed =
                Some(UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap());
        }
    }
}
