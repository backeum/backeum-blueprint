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
        // Mints a proof that is used as proof of donated value to the NFT repository.
        promise_token_manager: ResourceManager,

        // NFT minter badge
        nft_minter_badge: Vault,

        // Collected donations
        // TODO: Enable what tokens to accept as donations.
        donations: Vault,

        // Specific user identity that owns this component
        user_identity: String,

        // List of all the trophy ID's that have been donated through this contract.
        minted_trophy_id_list: Vec<NonFungibleLocalId>
    }

    impl Donation {
        pub fn new(promise_token_manager: ResourceManager, minter_badge: Bucket, user_identity: String) -> (Global<Donation>, Bucket) {
            // Creating an admin badge for the admin role, return it to the component creator.
            let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "Owner Badge", locked;
                        "description" => "Used to manage your Backeum donation contract", locked;
                    }
                ))
                .mint_initial_supply(1);

            let component = Self {
                nft_minter_badge: Vault::with_bucket(minter_badge),
                donations: Vault::new(RADIX_TOKEN),
                user_identity,
                promise_token_manager,
                non_fungible_id_list: Vec::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => rule!(require(admin_badge.resource_address()));
            ))
            .globalize();

            (component, admin_badge)
        }

        // donate is a public method, callable by anyone who want to donate to the user.
        pub fn donate(&mut self, tokens: Bucket, nf_proofs: Proof) -> (Proof, Bucket) {
            let checked_proof = nf_proofs.check(self.donor_badge_manager.address());
            let nf_list = checked_proof.as_non_fungible().non_fungible_local_ids();
            println!("NF proof: {:?}", donor_id);
            // Loop through NF LIST and check if any ID exists in the minted_trophy_id_list.

            // If ID found in the minted_trophy_id_list, then update metadata,
            // Otherwise mint new NFT.
        }
    }
}
