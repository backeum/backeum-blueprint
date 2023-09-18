use crate::data::TrophyData;
use scrypto::prelude::*;

// function to generate the url for the image
pub fn generate_url(
    base_path: String,
    donated: Decimal,
    created: String,
    collection_id: String,
) -> String {
    format!(
        "{}/nft/collection/{}?donated={}&created={}",
        base_path, collection_id, donated, created
    )
}

// function to generate the created string with a date format
fn generate_created_string() -> String {
    let time = UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap();
    let mut month = time.month().to_string();
    match time.month() {
        1 => month = "01".to_owned(),
        2 => month = "02".to_owned(),
        3 => month = "03".to_owned(),
        4 => month = "04".to_owned(),
        5 => month = "05".to_owned(),
        6 => month = "06".to_owned(),
        7 => month = "07".to_owned(),
        8 => month = "08".to_owned(),
        9 => month = "09".to_owned(),
        _ => {}
    }
    let mut day = time.day_of_month().to_string();
    match time.day_of_month() {
        1 => day = "01".to_owned(),
        2 => day = "02".to_owned(),
        3 => day = "03".to_owned(),
        4 => day = "04".to_owned(),
        5 => day = "05".to_owned(),
        6 => day = "06".to_owned(),
        7 => day = "07".to_owned(),
        8 => day = "08".to_owned(),
        9 => day = "09".to_owned(),
        _ => {}
    }
    format!("{}-{}-{}", time.year(), month, day)
}

#[blueprint]
mod collection {
    enable_package_royalties! {
        new => Free;
        donate_mint => Xrd(20.into());
        donate_update => Xrd(20.into());
        withdraw_donations => Free;
    }

    enable_method_auth! {
        roles {
            owner => updatable_by: [];
        },
        methods {
            donate_mint => PUBLIC;
            donate_update => PUBLIC;
            withdraw_donations => restrict_to: [owner];
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

        // Which collection this donation component is for
        collection_id: String,
    }

    impl Collection {
        pub fn new(
            trophy_resource_manager: ResourceManager,
            repository_owner_badge: ResourceAddress,
            collection_owner_badge: ResourceAddress,
            minter_badge: Bucket,
            user_name: String,
            user_slug: String,
            collection_id: String,
        ) -> Global<Collection> {
            Self {
                minter_badge: Vault::with_bucket(minter_badge),
                donations: Vault::new(XRD),
                user_name,
                user_slug,
                collection_id,
                trophy_resource_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(repository_owner_badge))))
            .roles(roles!(
                owner => rule!(require(collection_owner_badge));
            ))
            .globalize()
        }

        // donate_mint is a public method, callable by anyone who want to donate to the user.
        pub fn donate_mint(&mut self, tokens: Bucket) -> Bucket {
            // Push a proof of minter badge to the local auth zone for minting a trophy.
            LocalAuthZone::push(self.minter_badge.as_fungible().create_proof_of_amount(1));

            // Get the domain name used from the trophy resource manager.
            let domain: String = self
                .trophy_resource_manager
                .get_metadata("domain")
                .unwrap()
                .expect("No domain on NFT repository");

            let created = generate_created_string();
            let mut data = TrophyData {
                name: format!("Backer Trophy: {}", self.user_name),
                info_url: UncheckedUrl::of(format!("{}/p/{}", domain, self.user_slug)),
                collection_id: self.collection_id.clone(),
                created: created.clone(),
                donated: dec!(0),
                key_image_url: UncheckedUrl::of(""),
            };

            let trophy = self
                .trophy_resource_manager
                .mint_ruid_non_fungible(data.clone());

            let nft_id = trophy
                .as_non_fungible()
                .non_fungible::<TrophyData>()
                .local_id()
                .clone();

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
            trophy
        }

        // donate is a public method, callable by anyone who want to donate to the user.
        pub fn donate_update(&mut self, tokens: Bucket, proof: Proof) {
            LocalAuthZone::push(self.minter_badge.as_fungible().create_proof_of_amount(1));
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
            let mut data: TrophyData = self.trophy_resource_manager.get_non_fungible_data(&nft_id);

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
    }
}
