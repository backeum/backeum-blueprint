use crate::donation::donation::Donation;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct TrophyData {
    pub created: String,
    pub user_identity: String,
    #[mutable]
    pub donated: Decimal,
    #[mutable]
    pub key_image_url: String,
}

// function to generate the url for the image
fn generate_url(
    base_path: String,
    donated: Decimal,
    created: String,
    nft_id: String,
    user_identity: String,
) -> String {
    format!(
        "{}?donated={}&created={}&nft_id={}&user_identity={}",
        base_path, donated, created, nft_id, user_identity
    )
}

// function to generate the created string with a date format
fn generate_created_string() -> String {
    let time = UtcDateTime::from_instant(&Clock::current_time_rounded_to_minutes()).unwrap();
    format!("{}-{}-{}", time.year(), time.month(), time.day_of_month())
}

#[blueprint]
mod repository {
    enable_method_auth! {
        roles {
            trophy_minter => updatable_by: [OWNER];
        },
        methods {
            new_donation_component => PUBLIC;
            mint => restrict_to: [trophy_minter];
        }
    }

    struct Repository {
        // NFT resource address
        trophy_resource_manager: ResourceManager,

        // Badge for being able to mint trophies
        trophy_minter_badge_manager: ResourceManager,

        // Base path, e.g https://backeum.com/nft_image
        base_path: String,
    }

    impl Repository {
        pub fn new(base_path: String) -> (Global<Repository>, Bucket) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Runtime::blueprint_id());

            let trophy_resource_manager = ResourceBuilder::new_ruid_non_fungible::<TrophyData>(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Backeum Trophies", locked;
                        "description" => "Backeum trophies celebrates the patronage of its holder with donations to individual Backeum creators. A unique symbol of support for the community, it's a vibrant testament to encouragement.", locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();

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

            // Creating an admin badge for the admin role
            let trophy_minter_badge_manager = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Backeum Trophies Minter", locked;
                        "description" => "Grants authorization to mint NFTs from Backeum repository", locked;
                    }
                ))
                .mint_roles(mint_roles!{
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                })
                .withdraw_roles(withdraw_roles!{
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            let component = Self {
                trophy_resource_manager,
                trophy_minter_badge_manager,
                base_path,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(
                owner_badge.resource_address()
            ))))
            .roles(roles! {
                trophy_minter => rule!(require(trophy_minter_badge_manager.address()));
            })
            .with_address(address_reservation)
            .globalize();

            (component, owner_badge)
        }

        // new_donation_component sets up a new donation contract for a user, and give that contract
        // a mint badge that allows for it to create and update trophies. By going through Repository
        // for instantiation we can ensure that the mint badge is only given to a contract that is
        // made by Backeum.
        pub fn new_donation_component(&mut self) -> (Global<Donation>, Bucket) {
            let mint_badge = self.trophy_minter_badge_manager.mint(1);
            Donation::new(mint_badge, "".to_string())
        }

        // Used when new members register an account component to mine and reward a unique NFT token.
        pub fn mint(&mut self, user_identity: String) -> Bucket {
            let created = generate_created_string();
            let data = TrophyData {
                user_identity: user_identity.clone(),
                created: created.clone(),
                donated: dec!(0),
                key_image_url: "".to_string(),
            };

            // Mint the new NFT, and update the key_image_url with the UUID that was assigned to the NFT.
            let trophy = self.trophy_resource_manager.mint_ruid_non_fungible(data);
            let id = trophy
                .as_non_fungible()
                .non_fungible::<TrophyData>()
                .local_id()
                .clone();
            self.trophy_resource_manager.update_non_fungible_data(
                &id,
                "key_image_url",
                generate_url(
                    self.base_path.clone(),
                    Decimal::zero(),
                    created.clone(),
                    id.to_string(),
                    user_identity.clone(),
                ),
            );

            trophy
        }
    }
}
