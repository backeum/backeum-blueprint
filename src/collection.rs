use crate::data::{Creator, Membership, Transaction, Trophy};
use crate::util::*;
use scrypto::prelude::*;

// Arguments for initiating collection.
#[derive(ScryptoSbor)]
pub struct CollectionArg {
    pub trophy_resource_manager: ResourceManager,
    pub thanks_token_resource_manager: ResourceManager,
    pub membership_resource_manager: ResourceManager,
    pub creator_resource_manager: ResourceManager,
    pub repository_owner_access_badge_address: ResourceAddress,
    pub creator_badge_proof: CheckedProof,
    pub minter_badge: Bucket,
    pub creator_name: String,
    pub creator_slug: String,
    pub trophy_name: String,
    pub trophy_description: String,
    pub dapp_definition_address: GlobalAddress,
}

#[blueprint]
#[types(Trophy, Membership, Creator, Transaction)]
mod collection {
    enable_method_auth! {
        roles {
            repository_owner => updatable_by: [];
            owner => updatable_by: [];
        },
        methods {
            donate_mint => PUBLIC;
            donate_mint_with_membership => PUBLIC;
            donate_update => PUBLIC;
            donate_update_with_membership => PUBLIC;
            withdraw_donations => restrict_to: [owner];
            withdraw_fees => restrict_to: [repository_owner];
            close_collection => restrict_to: [owner];
        }
    }

    struct Collection {
        // Mints a proof that is used as proof of donated value to the NFT repository.
        trophy_resource_manager: ResourceManager,

        // Mints a proof that is used as proof of donated value to the NFT repository.
        membership_resource_manager: ResourceManager,

        // Creator badge address
        creator_resource_manager: ResourceManager,

        // Mints a proof that is used as proof of donated value to the NFT repository.
        thanks_token_resource_manager: ResourceManager,

        // NFT minter badge
        minter_badge: Vault,

        // Collected donations
        donations: Vault,

        // Fees for the donations
        fees: Vault,

        // Specific user name that owns this component
        creator_name: String,

        // Specific user slug that owns this component
        creator_slug: String,

        // Name of the trophy
        trophy_name: String,

        // Description of the trophy
        trophy_description: String,

        // Which collection this collection component is for
        collection_id: String,

        // Creator badge address
        creator_badge_global_id: NonFungibleGlobalId,

        // Closed date for the collection
        closed: Option<UtcDateTime>,
    }

    impl Collection {
        pub fn new(arg: CollectionArg) -> Global<Collection> {
            let (reservation, address) =
                Runtime::allocate_component_address(Collection::blueprint_id());
            let collection_id = Runtime::bech32_encode_address(address);

            let creator_badge_global_id = NonFungibleGlobalId::new(
                arg.creator_badge_proof.resource_address(),
                arg.creator_badge_proof
                    .as_non_fungible()
                    .non_fungible_local_id(),
            );

            Self {
                minter_badge: Vault::with_bucket(arg.minter_badge),
                donations: Vault::new(XRD),
                fees: Vault::new(XRD),
                collection_id,
                creator_badge_global_id: creator_badge_global_id.clone(),
                trophy_resource_manager: arg.trophy_resource_manager,
                thanks_token_resource_manager: arg.thanks_token_resource_manager,
                membership_resource_manager: arg.membership_resource_manager,
                creator_resource_manager: arg.creator_resource_manager,
                creator_name: arg.creator_name,
                creator_slug: arg.creator_slug,
                trophy_name: arg.trophy_name,
                trophy_description: arg.trophy_description,
                closed: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                arg.repository_owner_access_badge_address
            ))))
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(arg.repository_owner_access_badge_address));
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => rule!(deny_all);
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "dapp_definition" => arg.dapp_definition_address, locked;
                }
            ))
            .roles(roles!(
                repository_owner => rule!(require(arg.repository_owner_access_badge_address));
                owner => rule!(require(creator_badge_global_id.clone()));
            ))
            .with_address(reservation)
            .globalize()
        }

        // update_creator_metadata is a private method that updates the creator metadata based on
        // the amount donated.
        fn update_creator_metadata(&mut self, amount: Decimal) {
            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let creator_nft_id = self.creator_badge_global_id.local_id();

            let mut data: Creator = self
                .creator_resource_manager
                .get_non_fungible_data(creator_nft_id);

            data.funded += amount;
            data.key_image_url = UncheckedUrl::of(generate_creator_url(
                domain.to_string(),
                data.funded,
                data.created,
            ));

            self.creator_resource_manager.update_non_fungible_data(
                creator_nft_id,
                "funded",
                data.funded,
            );
            self.creator_resource_manager.update_non_fungible_data(
                creator_nft_id,
                "key_image_url",
                data.key_image_url,
            );
        }

        // mint_membership is a private method that mints a membership NFT based on the amount
        fn mint_membership(&mut self, donated: Decimal) -> Bucket {
            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let created = generate_created_string(
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap(),
            );

            let transaction = Transaction {
                amount: donated,
                created: created.clone(),
            };

            let data = Membership {
                name: format!("Membership: {}", self.creator_name),
                description: format!("Digital emblem celebrating {}'s crowdfunding journey. It evolves with cumulative donations. It's a symbol of encouragement, encapsulating the artist-backer bond in the digital age.", self.creator_name).to_string(),
                creator: self.creator_badge_global_id.clone(),
                creator_name: self.creator_name.clone(),
                creator_slug: self.creator_slug.clone(),
                info_url: UncheckedUrl::of(format!("{}/p/{}", domain, self.creator_slug)),
                created: created.clone(),
                transactions: vec![transaction],
                donated,
                key_image_url: UncheckedUrl::of(generate_membership_url(
                    domain.to_string(),
                    donated,
                    created,
                    self.creator_slug.to_string(),
                )),
            };

            self.membership_resource_manager
                .mint_ruid_non_fungible(data.clone())
        }

        // update_membership_metadata is a private method that updates the membership metadata based
        // on the amount donated.
        fn update_membership_metadata(&mut self, nft_id: NonFungibleLocalId, amount: Decimal) {
            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            // Get data from the Membership data based on NF id.
            let mut data: Membership = self
                .membership_resource_manager
                .get_non_fungible_data(&nft_id);

            assert_eq!(
                data.creator_slug, self.creator_slug,
                "The given membership does not match this component."
            );

            assert!(
                data.creator.eq(&self.creator_badge_global_id),
                "The given membership does not match this component."
            );

            let created = generate_created_string(
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap(),
            );

            let transaction = Transaction { amount, created };

            // Generate new data based on the updated donation value.
            data.transactions.push(transaction);
            data.donated += amount;
            data.key_image_url = UncheckedUrl::of(generate_membership_url(
                domain.to_string(),
                data.donated,
                data.created,
                self.creator_slug.clone(),
            ));

            // Update NF with new data
            self.membership_resource_manager.update_non_fungible_data(
                &nft_id,
                "transactions",
                data.transactions,
            );
            self.membership_resource_manager.update_non_fungible_data(
                &nft_id,
                "donated",
                data.donated,
            );
            self.membership_resource_manager.update_non_fungible_data(
                &nft_id,
                "key_image_url",
                data.key_image_url,
            );
        }

        // mint_trophy is a private method that mints a trophy NFT based on the amount donated.
        fn mint_trophy(&mut self, amount: Decimal) -> Bucket {
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let created = generate_created_string(
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap(),
            );

            let transaction = Transaction {
                amount,
                created: created.clone(),
            };

            // Create the trophy data.
            let data = Trophy {
                name: self.trophy_name.clone(),
                description: self.trophy_description.clone(),
                creator: self.creator_badge_global_id.clone(),
                creator_name: self.creator_name.clone(),
                creator_slug: self.creator_slug.clone(),
                info_url: UncheckedUrl::of(format!("{}/p/{}", domain, self.creator_slug)),
                collection_id: self.collection_id.clone(),
                created: created.clone(),
                transactions: vec![transaction],
                donated: amount,
                key_image_url: UncheckedUrl::of(generate_trophy_url(
                    domain.to_string(),
                    amount,
                    created.clone(),
                    self.collection_id.clone(),
                )),
            };

            // Mint the trophy NFT.
            self.trophy_resource_manager
                .mint_ruid_non_fungible(data.clone())
        }

        // update_trophy_metadata is a private method that updates the trophy metadata based on the
        // amount donated.
        fn update_trophy_metadata(&mut self, nft_id: NonFungibleLocalId, amount: Decimal) {
            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            // Get data from the Trophy data based on NF id.
            let mut data: Trophy = self.trophy_resource_manager.get_non_fungible_data(&nft_id);

            // Check whether the NF user_identity is owned by this component.
            assert_eq!(
                data.collection_id, self.collection_id,
                "The given trophy does match the collection id of this component."
            );

            assert!(
                data.creator.eq(&self.creator_badge_global_id),
                "The given membership does not match this component."
            );

            let created = generate_created_string(
                UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap(),
            );

            let transaction = Transaction { amount, created };

            // Generate new data based on the updated donation value.
            data.transactions.push(transaction);
            data.donated += amount;
            data.key_image_url = UncheckedUrl::of(generate_trophy_url(
                domain.to_string(),
                data.donated,
                data.created,
                self.collection_id.clone(),
            ));

            // Update NF with new data
            self.trophy_resource_manager.update_non_fungible_data(
                &nft_id,
                "transactions",
                data.transactions,
            );
            self.trophy_resource_manager
                .update_non_fungible_data(&nft_id, "donated", data.donated);
            self.trophy_resource_manager.update_non_fungible_data(
                &nft_id,
                "key_image_url",
                data.key_image_url,
            );
        }

        // donate_mint is a public method, callable by anyone who want to donate to the user. In
        // return they will get a trophy NFT that represents the donation.
        pub fn donate_mint(&mut self, mut tokens: Bucket) -> (Bucket, Bucket, Bucket) {
            if self.closed.is_some() {
                panic!("This collection is permanently closed.");
            }

            // Push a proof of minter badge to the local auth zone for minting a trophy.
            LocalAuthZone::push(self.minter_badge.as_fungible().create_proof_of_amount(1));

            // Update creator badge
            self.update_creator_metadata(tokens.amount());

            let trophy = self.mint_trophy(tokens.amount());

            let membership = self.mint_membership(tokens.amount());

            let thanks = self.thanks_token_resource_manager.mint(tokens.amount());

            self.fees.put(tokens.take(tokens.amount() * dec!(0.04)));

            self.donations.put(tokens);

            (trophy, thanks, membership)
        }

        // donate_mint_with_membership is a public method, callable by anyone who want to donate to
        // the user. In return they will get a trophy NFT that represents the donation. This method
        // requires a membership proof to be passed in.
        pub fn donate_mint_with_membership(
            &mut self,
            mut tokens: Bucket,
            membership_proof: Proof,
        ) -> (Bucket, Bucket) {
            if self.closed.is_some() {
                panic!("This collection is permanently closed.");
            }

            // Push a proof of minter badge to the local auth zone for minting a trophy.
            LocalAuthZone::push(self.minter_badge.as_fungible().create_proof_of_amount(1));

            // Update creator badge
            self.update_creator_metadata(tokens.amount());

            let checked_membership_proof =
                membership_proof.check(self.membership_resource_manager.address());

            // Update membership badge
            self.update_membership_metadata(
                checked_membership_proof
                    .as_non_fungible()
                    .non_fungible_local_id(),
                tokens.amount(),
            );

            let trophy = self.mint_trophy(tokens.amount());

            // Mint thanks tokens equal to the donated amount.
            let thanks = self.thanks_token_resource_manager.mint(tokens.amount());

            // Take fees from the donation.
            self.fees.put(tokens.take(tokens.amount() * dec!(0.04)));

            // Take all tokens, and return trophy.
            self.donations.put(tokens);
            (trophy, thanks)
        }

        // donate_update is a public method, callable by anyone who want to donate to the user.
        pub fn donate_update(
            &mut self,
            mut tokens: Bucket,
            trophy_proof: Proof,
        ) -> (Bucket, Bucket) {
            if self.closed.is_some() {
                panic!("This collection is permanently closed.");
            }

            // Push a proof of minter badge to the local auth zone for minting a trophy.
            LocalAuthZone::push(self.minter_badge.as_fungible().create_proof_of_amount(1));

            // Update creator badge
            self.update_creator_metadata(tokens.amount());

            // Check that the proof is of same resource address.
            let checked_proof = trophy_proof.check(self.trophy_resource_manager.address());

            // Update trophy NF metadata
            self.update_trophy_metadata(
                checked_proof.as_non_fungible().non_fungible_local_id(),
                tokens.amount(),
            );

            let membership = self.mint_membership(tokens.amount());

            // Mint thanks tokens equal to the donated amount.
            let thanks = self.thanks_token_resource_manager.mint(tokens.amount());

            // Take fees from the donation.
            self.fees.put(tokens.take(tokens.amount() * dec!(0.04)));

            // Take all tokens, and return trophy.
            self.donations.put(tokens);
            (thanks, membership)
        }

        // donate_update_with_membership is a public method, callable by anyone who want to donate to the user.
        // This method requires a membership proof, and trophy proof to be passed in.
        pub fn donate_update_with_membership(
            &mut self,
            mut tokens: Bucket,
            trophy_proof: Proof,
            membership_proof: Proof,
        ) -> Bucket {
            if self.closed.is_some() {
                panic!("This collection is permanently closed.");
            }

            // Push a proof of minter badge to the local auth zone for minting a trophy.
            LocalAuthZone::push(self.minter_badge.as_fungible().create_proof_of_amount(1));

            // Update creator badge
            self.update_creator_metadata(tokens.amount());

            let checked_membership_proof =
                membership_proof.check(self.membership_resource_manager.address());

            // Update membership badge
            self.update_membership_metadata(
                checked_membership_proof
                    .as_non_fungible()
                    .non_fungible_local_id(),
                tokens.amount(),
            );

            // Check that the proof is of same resource address.
            let checked_trophy_proof = trophy_proof.check(self.trophy_resource_manager.address());

            self.update_trophy_metadata(
                checked_trophy_proof
                    .as_non_fungible()
                    .non_fungible_local_id(),
                tokens.amount(),
            );

            // Mint thanks tokens equal to the donated amount.
            let thanks = self.thanks_token_resource_manager.mint(tokens.amount());

            // Take fees from the donation.
            self.fees.put(tokens.take(tokens.amount() * dec!(0.04)));

            // Take all tokens, and return trophy.
            self.donations.put(tokens);
            thanks
        }

        // withdraw_donations is a method for the admin to withdraw all donations.
        pub fn withdraw_donations(&mut self) -> Bucket {
            self.donations.take_all()
        }

        // withdraw_fees is a method for the repository owner to withdraw all fees.
        pub fn withdraw_fees(&mut self) -> Bucket {
            self.fees.take_all()
        }

        // close_collection is a method for the collection admin to close the collection
        // permanently. This will prevent any further donations to be made to the collection, and
        // will prevent any further minting or updating to the trophies.
        pub fn close_collection(&mut self) -> Bucket {
            if self.closed.is_some() {
                panic!("This collection is permanently closed.");
            }

            self.closed =
                Some(UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap());

            // Withdraw all remaining donations.
            self.donations.take_all()
        }
    }
}
