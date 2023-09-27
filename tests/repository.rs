#[path = "./common.rs"]
mod common;
use common::{execute_manifest, mint_collection_owner_badge, new_account, new_runner, Nft};

use backeum_blueprint::data::Trophy;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_collection_component_success() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);
        let collection_admin_badge_id: NonFungibleGlobalId;
        {
            collection_admin_badge_id =
                mint_collection_owner_badge(&mut base, &collection_admin_account);
        }

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id,
            )
            .pop_from_auth_zone("collection_owner_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_owner_badge_proof"),
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();
    }

    #[test]
    fn new_collection_component_failure_unknown_badge() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);

        let mut fake_collection_owner_badges = BTreeMap::new();
        fake_collection_owner_badges.insert(
            "name".to_owned(),
            MetadataValue::String("Backeum Repository Owner Badges".to_owned()),
        );
        fake_collection_owner_badges.insert(
            "description".to_owned(),
            MetadataValue::String(
                "Grants component ownership of backeum repository and collection components"
                    .to_owned(),
            ),
        );
        fake_collection_owner_badges.insert(
            "info_url".to_owned(),
            MetadataValue::Url(UncheckedUrl("https://staging.backeum.com".to_owned())),
        );
        fake_collection_owner_badges.insert(
            "tags".to_owned(),
            MetadataValue::StringArray(vec!["backeum".to_owned()]),
        );
        fake_collection_owner_badges.insert(
            "icon_url".to_string(),
            MetadataValue::Url(UncheckedUrl(
                "https://staging.backeum.com/bucket/assets/wallet-assets/component-owner-badge.png"
                    .to_owned(),
            )),
        );

        let fake_collection_owner_badge_metadata = ModuleConfig {
            init: fake_collection_owner_badges.into(),
            roles: RoleAssignmentInit::default(),
        };

        // Create an owner badge used for repository component.
        let manifest = ManifestBuilder::new()
            .create_ruid_non_fungible_resource(
                OwnerRole::None,
                false,
                fake_collection_owner_badge_metadata,
                Default::default(),
                Some([Nft {
                    name: "Badge".to_owned(),
                    description: "Owner badge for packages deployed for Backeum".to_owned(),
                    icon_url: UncheckedUrl::of(
                        "https://staging.backeum.com/bucket/assets/wallet-assets/badge.png"
                            .to_owned(),
                    ),
                    info_url: UncheckedUrl::of("https://staging.backeum.com".to_owned()),
                    tags: vec!["backeum".to_owned(), "badge".to_owned()],
                }]),
            )
            .deposit_batch(collection_admin_account.wallet_address);

        // Execute the manifest.
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_failure_unknown_badge_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        let result = receipt.expect_commit_success();

        // Get the repository component address.
        let fake_collection_owner_badge_resource_address = result.new_resource_addresses()[0];

        let fake_collection_owner_badge_vault = base.test_runner.get_component_vaults(
            collection_admin_account.wallet_address,
            fake_collection_owner_badge_resource_address,
        );

        let fake_collection_owner_badge_local_id = base
            .test_runner
            .inspect_non_fungible_vault(fake_collection_owner_badge_vault[0])
            .unwrap()
            .1
            .next()
            .unwrap();

        let fake_collection_owner_badge_global_id = NonFungibleGlobalId::new(
            fake_collection_owner_badge_resource_address,
            fake_collection_owner_badge_local_id.clone(),
        );

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                fake_collection_owner_badge_global_id,
            )
            .pop_from_auth_zone("collection_owner_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_owner_badge_proof"),
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_failure_unknown_badge_2",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn new_collection_component_and_badge_success() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .call_method(
                base.repository_component,
                "new_collection_component_and_badge",
                manifest_args!("Kansuler", "kansuler",),
            )
            .assert_worktop_contains(base.collection_owner_badge_resource_address, dec!(1))
            .deposit_batch(collection_admin_account.wallet_address);

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_and_badge_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        let result = receipt.expect_commit_success();

        let collection_component_address = result.new_component_addresses()[0];
        let collection_owner_badge_vault = base.test_runner.get_component_vaults(
            collection_admin_account.wallet_address,
            base.collection_owner_badge_resource_address,
        );

        let collection_owner_badge_local_id = base
            .test_runner
            .inspect_non_fungible_vault(collection_owner_badge_vault[0])
            .unwrap()
            .1
            .next()
            .unwrap();

        let collection_owner_badge_global_id = NonFungibleGlobalId::new(
            base.collection_owner_badge_resource_address,
            collection_owner_badge_local_id.clone(),
        );

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_owner_badge_global_id,
            )
            .call_method(
                collection_component_address,
                "withdraw_donations",
                manifest_args!(),
            )
            .deposit_batch(collection_admin_account.wallet_address);

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_and_badge_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();
    }

    #[test]
    fn merge_success() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);
        let collection_admin_badge_id: NonFungibleGlobalId;
        {
            collection_admin_badge_id =
                mint_collection_owner_badge(&mut base, &collection_admin_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id,
            )
            .pop_from_auth_zone("collection_owner_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_owner_badge_proof"),
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "merge_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(500))
            .take_from_worktop(XRD, dec!(250), "donation_amount_1")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_1"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .take_from_worktop(XRD, dec!(250), "donation_amount_2")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_2"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(2))
            .take_all_from_worktop(base.trophy_resource_address, "trophies")
            .call_method_with_name_lookup(base.repository_component, "merge_trophies", |lookup| {
                (lookup.bucket("trophies"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .take_all_from_worktop(base.trophy_resource_address, "new_trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "new_trophy");

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "merge_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();
        assert_eq!(
            base.test_runner.get_component_balance(
                donation_account.wallet_address,
                base.trophy_resource_address
            ),
            dec!(1)
        );
        assert_eq!(
            base.test_runner
                .get_component_balance(donation_account.wallet_address, XRD),
            dec!(9500)
        );

        let trophy_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.trophy_resource_address,
        );

        let trophy_id: NonFungibleLocalId;
        {
            let mut trophies = base
                .test_runner
                .inspect_non_fungible_vault(trophy_vault[0])
                .unwrap()
                .1;

            trophy_id = trophies.next().unwrap().clone();
        }

        let trophy_data: Trophy = base
            .test_runner
            .get_non_fungible_data(base.trophy_resource_address, trophy_id.clone());

        let result = AddressBech32Encoder::new(&NetworkDefinition::simulator())
            .encode(&collection_component.to_vec())
            .unwrap();

        assert_eq!(trophy_data.collection_id, result);

        assert_eq!(trophy_data.name, "Backer Trophy: Kansuler");
        assert_eq!(
            trophy_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(trophy_data.created, "1970-01-01");
        assert_eq!(trophy_data.donated, dec!(540));

        assert_eq!(
            trophy_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=540&created=1970-01-01",
                trophy_data.collection_id
            ))
        );
    }

    #[test]
    fn merge_failure_different_collection() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);
        let collection_admin_badge_id: NonFungibleGlobalId;
        {
            collection_admin_badge_id =
                mint_collection_owner_badge(&mut base, &collection_admin_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id,
            )
            .pop_from_auth_zone("collection_owner_badge_proof")
            .clone_proof(
                "collection_owner_badge_proof",
                "collection_owner_badge_proof_1",
            )
            .clone_proof(
                "collection_owner_badge_proof",
                "collection_owner_badge_proof_2",
            )
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_owner_badge_proof_1"),
                    )
                },
            )
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_owner_badge_proof_2"),
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "merge_failure_different_collection_1merge_failure_different_collection",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component_1 = receipt.expect_commit_success().new_component_addresses()[0];
        let collection_component_2 = receipt.expect_commit_success().new_component_addresses()[1];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(500))
            .take_from_worktop(XRD, dec!(250), "donation_amount_1")
            .call_method_with_name_lookup(collection_component_1, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_1"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .take_from_worktop(XRD, dec!(250), "donation_amount_2")
            .call_method_with_name_lookup(collection_component_2, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_2"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(2))
            .take_all_from_worktop(base.trophy_resource_address, "trophies")
            .call_method_with_name_lookup(base.repository_component, "merge_trophies", |lookup| {
                (lookup.bucket("trophies"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .take_all_from_worktop(base.trophy_resource_address, "new_trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "new_trophy");

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "merge_failure_different_collection_2",
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();

        assert_eq!(
            base.test_runner.get_component_balance(
                donation_account.wallet_address,
                base.trophy_resource_address
            ),
            dec!(0)
        );
        assert_eq!(
            base.test_runner
                .get_component_balance(donation_account.wallet_address, XRD),
            dec!(10000)
        );
    }
}
