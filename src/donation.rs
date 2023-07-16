use scrypto::prelude::*;

#[blueprint]
mod donation {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            donate => PUBLIC;
        }
    }

    struct Donation {
        // NFT minter badge
        nft_minter_badge: Vault,

        // Collected donations
        // TODO: Enable what tokens to accept as donations.
        donations: Vault,

        // Specific user identity that owns this component
        user_identity: String,
    }

    impl Donation {
        pub fn new(minter_badge: Bucket, user_identity: String) -> (Global<Donation>, Bucket) {
            // Creating an admin badge for the admin role, return it to the component creator.
            let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Owner Badge", locked;
                        "description" => "Used to manage your Backeum donation contract", locked;
                    }
                )).mint_initial_supply(1);

            let component = Self {
                nft_minter_badge: Vault::with_bucket(minter_badge),
                donations: Vault::new(RADIX_TOKEN),
                user_identity,
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(
                    roles!(
                        admin => rule!(require(admin_badge.resource_address()));
                    )
                )
                .globalize();

            (component, admin_badge)
        }

        // donate is a public method, callable by anyone who want to donate to the user.
        pub fn donate(&mut self, tokens: Bucket) -> Proof {
            self.donations.put(tokens);
            Proof::from(self.nft_minter_badge.as_fungible().create_proof_of_amount(1))
        }
    }
}
