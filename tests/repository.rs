#[path = "./common.rs"]
mod common;
use common::{execute_manifest, mint_creator_badge, new_account, new_runner, Nft};

use backeum_blueprint::data::{Membership, Trophy};
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_collection_component_success() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id,
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| (lookup.proof("creator_badge_proof"),),
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();
    }

    #[test]
    fn new_collection_component_failure_unknown_badge() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);

        let mut fake_creator_badges = BTreeMap::new();
        fake_creator_badges.insert(
            "name".to_owned(),
            MetadataValue::String("Backeum Repository Owner Badges".to_owned()),
        );
        fake_creator_badges.insert(
            "description".to_owned(),
            MetadataValue::String(
                "Grants component ownership of backeum repository and collection components"
                    .to_owned(),
            ),
        );
        fake_creator_badges.insert(
            "info_url".to_owned(),
            MetadataValue::Url(UncheckedUrl("https://staging.backeum.com".to_owned())),
        );
        fake_creator_badges.insert(
            "tags".to_owned(),
            MetadataValue::StringArray(vec!["backeum".to_owned()]),
        );
        fake_creator_badges.insert(
            "icon_url".to_string(),
            MetadataValue::Url(UncheckedUrl(
                "https://staging.backeum.com/bucket/assets/wallet-assets/component-owner-badge.png"
                    .to_owned(),
            )),
        );

        let fake_creator_badge_metadata = ModuleConfig {
            init: fake_creator_badges.into(),
            roles: RoleAssignmentInit::default(),
        };

        // Create an owner badge used for repository component.
        let manifest = ManifestBuilder::new()
            .create_ruid_non_fungible_resource(
                OwnerRole::None,
                false,
                fake_creator_badge_metadata,
                Default::default(),
                Some([Nft {
                    name: "Badge".to_owned(),
                    description: "Owner badge for packages deployed for Backeum".to_owned(),
                }]),
            )
            .deposit_batch(creator_badge_account.wallet_address);

        // Execute the manifest.
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_failure_unknown_badge_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        let mut tmp_domains = BTreeMap::new();
        tmp_domains.insert(
            "name".to_owned(),
            MetadataValue::String("Backeum Repository Owner Badges".to_owned()),
        );

        let tmp_domains_metadata = ModuleConfig {
            init: tmp_domains.into(),
            roles: RoleAssignmentInit::default(),
        };

        // Create an owner badge used for repository component.
        let manifest = ManifestBuilder::new()
            .create_ruid_non_fungible_resource(
                OwnerRole::None,
                false,
                tmp_domains_metadata,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                Some([Nft {
                    name: "Test.xrd".to_owned(),
                    description: "Owner badge for packages deployed for Backeum".to_owned(),
                }]),
            )
            .deposit_batch(creator_badge_account.wallet_address);

        // Execute the manifest.
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "create_tmp_domains",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        let result = receipt.expect_commit_success();

        // Get the repository component address.
        let fake_creator_badge_resource_address = result.new_resource_addresses()[0];

        let fake_creator_badge_vault = base.test_runner.get_component_vaults(
            creator_badge_account.wallet_address,
            fake_creator_badge_resource_address,
        );

        let fake_creator_badge_local_id = base
            .test_runner
            .inspect_non_fungible_vault(fake_creator_badge_vault[0])
            .unwrap()
            .1
            .next()
            .unwrap();

        let fake_creator_badge_global_id = NonFungibleGlobalId::new(
            fake_creator_badge_resource_address,
            fake_creator_badge_local_id.clone(),
        );

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                fake_creator_badge_global_id,
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| (lookup.proof("creator_badge_proof"),),
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_failure_unknown_badge_2",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn new_collection_component_and_badge_success() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .call_method(
                base.repository_component,
                "new_collection_component_and_badge",
                manifest_args!("Kansuler", "kansuler",),
            )
            .assert_worktop_contains(base.creator_badge_resource_address, dec!(1))
            .deposit_batch(creator_badge_account.wallet_address);

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_and_badge_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        let result = receipt.expect_commit_success();

        let collection_component_address = result.new_component_addresses()[0];
        let creator_badge_vault = base.test_runner.get_component_vaults(
            creator_badge_account.wallet_address,
            base.creator_badge_resource_address,
        );

        let creator_badge_local_id = base
            .test_runner
            .inspect_non_fungible_vault(creator_badge_vault[0])
            .unwrap()
            .1
            .next()
            .unwrap();

        let creator_badge_global_id = NonFungibleGlobalId::new(
            base.creator_badge_resource_address,
            creator_badge_local_id.clone(),
        );

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_global_id,
            )
            .call_method(
                collection_component_address,
                "withdraw_donations",
                manifest_args!(),
            )
            .deposit_batch(creator_badge_account.wallet_address);

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "new_collection_component_and_badge_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();
    }

    #[test]
    fn merge_trophies_success() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id,
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| (lookup.proof("creator_badge_proof"),),
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "merge_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
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
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(500))
            .deposit_batch(donation_account.wallet_address);

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

        assert_eq!(trophy_data.name, "Trophy: Kansuler");
        assert_eq!(
            trophy_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(trophy_data.created, "1970-01-01");
        assert_eq!(trophy_data.donated, dec!(500));

        assert_eq!(
            trophy_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=500&created=1970-01-01",
                trophy_data.collection_id
            ))
        );
    }

    #[test]
    fn merge_membership_success() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id,
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| (lookup.proof("creator_badge_proof"),),
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "merge_membership_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
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
            .assert_worktop_contains(base.membership_resource_address, dec!(1))
            .take_from_worktop(XRD, dec!(250), "donation_amount_2")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_2"),)
            })
            .assert_worktop_contains(base.membership_resource_address, dec!(2))
            .take_all_from_worktop(base.membership_resource_address, "memberships")
            .call_method_with_name_lookup(
                base.repository_component,
                "merge_memberships",
                |lookup| (lookup.bucket("memberships"),),
            )
            .assert_worktop_contains(base.membership_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(500))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "merge_membership_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();
        assert_eq!(
            base.test_runner.get_component_balance(
                donation_account.wallet_address,
                base.membership_resource_address
            ),
            dec!(1)
        );
        assert_eq!(
            base.test_runner
                .get_component_balance(donation_account.wallet_address, XRD),
            dec!(9500)
        );

        let membership_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.membership_resource_address,
        );

        let membership_id: NonFungibleLocalId;
        {
            let mut memberships = base
                .test_runner
                .inspect_non_fungible_vault(membership_vault[0])
                .unwrap()
                .1;

            membership_id = memberships.next().unwrap().clone();
        }

        let membership_data: Membership = base
            .test_runner
            .get_non_fungible_data(base.membership_resource_address, membership_id.clone());

        assert_eq!(membership_data.user_slug, "kansuler");
        assert_eq!(membership_data.user_name, "Kansuler");

        assert_eq!(membership_data.name, "Membership: Kansuler");
        assert_eq!(
            membership_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(membership_data.created, "1970-01-01");
        assert_eq!(membership_data.donated, dec!(500));

        assert_eq!(
            membership_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/membership/{}?donated=500&created=1970-01-01",
                membership_data.user_slug
            ))
        );
    }

    #[test]
    fn merge_failure_different_collection() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id,
            )
            .pop_from_auth_zone("creator_badge_proof")
            .clone_proof("creator_badge_proof", "creator_badge_proof_1")
            .clone_proof("creator_badge_proof", "creator_badge_proof_2")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| (lookup.proof("creator_badge_proof_1"),),
            )
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| (lookup.proof("creator_badge_proof_2"),),
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "merge_failure_different_collection_1merge_failure_different_collection",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
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
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(500))
            .deposit_batch(donation_account.wallet_address);

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
