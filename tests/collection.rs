#[path = "./common.rs"]
mod common;
use common::{execute_manifest, mint_creator_badge, new_account, new_runner};

use backeum_blueprint::data::{Membership, Trophy};
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn donate_mint_success() {
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
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(150))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.membership_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_success_2",
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
            dec!(9850)
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

        assert_eq!(trophy_data.name, "Trophy name");
        assert_eq!(trophy_data.description, "Kansulers trophy");
        assert_eq!(trophy_data.creator_slug, "kansuler");
        assert_eq!(trophy_data.creator_name, "Kansuler");
        assert_eq!(
            trophy_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(trophy_data.created, "1970-01-01");
        assert_eq!(trophy_data.donated, dec!(150));

        assert_eq!(
            trophy_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=150&created=1970-01-01",
                trophy_data.collection_id
            ))
        );

        let membership_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.membership_resource_address,
        );

        let membership_id: NonFungibleLocalId;
        {
            let mut membership = base
                .test_runner
                .inspect_non_fungible_vault(membership_vault[0])
                .unwrap()
                .1;

            membership_id = membership.next().unwrap().clone();
        }

        let membership_data: Membership = base
            .test_runner
            .get_non_fungible_data(base.membership_resource_address, membership_id.clone());

        assert_eq!(membership_data.name, "Membership: Kansuler");
        assert_eq!(
            membership_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(membership_data.created, "1970-01-01");
        assert_eq!(membership_data.donated, dec!(150));

        assert_eq!(
            membership_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/membership/{}?donated=150&created=1970-01-01",
                membership_data.creator_slug
            ))
        );
    }

    #[test]
    fn donate_mint_with_membership_success() {
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
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_with_membership_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(150))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.membership_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_with_membership_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        let membership_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.membership_resource_address,
        );

        let membership_id: NonFungibleLocalId;
        {
            let mut trophies = base
                .test_runner
                .inspect_non_fungible_vault(membership_vault[0])
                .unwrap()
                .1;

            membership_id = trophies.next().unwrap().clone();
        }

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(150))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .create_proof_from_account_of_non_fungible(
                donation_account.wallet_address,
                NonFungibleGlobalId::new(base.membership_resource_address, membership_id.clone()),
            )
            .create_proof_from_auth_zone_of_non_fungibles(
                base.membership_resource_address,
                vec![membership_id.clone()],
                "membership_proof",
            )
            .call_method_with_name_lookup(
                collection_component,
                "donate_mint_with_membership",
                |lookup| {
                    (
                        lookup.bucket("donation_amount"),
                        lookup.proof("membership_proof"),
                    )
                },
            )
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.membership_resource_address, dec!(0))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_with_membership_success_3",
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
            dec!(2)
        );
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
            dec!(9700)
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

        assert_eq!(trophy_data.name, "Trophy name");
        assert_eq!(trophy_data.description, "Kansulers trophy");
        assert_eq!(trophy_data.creator_slug, "kansuler");
        assert_eq!(trophy_data.creator_name, "Kansuler");
        assert_eq!(
            trophy_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(trophy_data.created, "1970-01-01");
        assert_eq!(trophy_data.donated, dec!(150));

        assert_eq!(
            trophy_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=150&created=1970-01-01",
                trophy_data.collection_id
            ))
        );

        let membership_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.membership_resource_address,
        );

        let membership_id: NonFungibleLocalId;
        {
            let mut membership = base
                .test_runner
                .inspect_non_fungible_vault(membership_vault[0])
                .unwrap()
                .1;

            membership_id = membership.next().unwrap().clone();
        }

        let membership_data: Membership = base
            .test_runner
            .get_non_fungible_data(base.membership_resource_address, membership_id.clone());

        assert_eq!(membership_data.name, "Membership: Kansuler");
        assert_eq!(
            membership_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(membership_data.created, "1970-01-01");
        assert_eq!(membership_data.donated, dec!(300));

        assert_eq!(
            membership_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/membership/{}?donated=300&created=1970-01-01",
                membership_data.creator_slug
            ))
        );
    }

    #[test]
    fn donate_mint_failure_too_low_amount() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id,
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_failure_too_low_amount_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(0))
            .take_from_worktop(XRD, dec!(0), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(0))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_failure_too_low_amount_2",
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

    #[test]
    fn donate_mint_failure_closed_collection() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_failure_closed_collection_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_failure_closed_collection_2",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(150))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_failure_closed_collection_3",
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

    #[test]
    fn donate_update_success() {
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
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(150))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.membership_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_update_success_1",
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
            dec!(9850)
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

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(300))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .create_proof_from_account_of_non_fungible(
                donation_account.wallet_address,
                NonFungibleGlobalId::new(base.trophy_resource_address, trophy_id.clone()),
            )
            .create_proof_from_auth_zone_of_non_fungibles(
                base.trophy_resource_address,
                vec![trophy_id.clone()],
                "proof",
            )
            .call_method_with_name_lookup(collection_component, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("proof"))
            })
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .assert_worktop_contains(base.membership_resource_address, dec!(1))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_update_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        let trophy_data: Trophy = base
            .test_runner
            .get_non_fungible_data(base.trophy_resource_address, trophy_id.clone());

        let result = AddressBech32Encoder::new(&NetworkDefinition::simulator())
            .encode(&collection_component.to_vec())
            .unwrap();

        assert_eq!(trophy_data.collection_id, result);

        assert_eq!(trophy_data.name, "Trophy name");
        assert_eq!(trophy_data.description, "Kansulers trophy");
        assert_eq!(trophy_data.creator_slug, "kansuler");
        assert_eq!(trophy_data.creator_name, "Kansuler");
        assert_eq!(
            trophy_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(trophy_data.created, "1970-01-01");
        assert_eq!(trophy_data.donated, dec!(300));

        assert_eq!(
            trophy_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=300&created=1970-01-01",
                trophy_data.collection_id
            ))
        );
    }

    #[test]
    fn donate_update_with_membership_success() {
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
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(150))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.membership_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_update_with_membership_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

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

        let membership_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.membership_resource_address,
        );

        let membership_id: NonFungibleLocalId;
        {
            let mut trophies = base
                .test_runner
                .inspect_non_fungible_vault(membership_vault[0])
                .unwrap()
                .1;

            membership_id = trophies.next().unwrap().clone();
        }

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(300))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .create_proof_from_account_of_non_fungible(
                donation_account.wallet_address,
                NonFungibleGlobalId::new(base.trophy_resource_address, trophy_id.clone()),
            )
            .create_proof_from_auth_zone_of_non_fungibles(
                base.trophy_resource_address,
                vec![trophy_id.clone()],
                "trophy_proof",
            )
            .create_proof_from_account_of_non_fungible(
                donation_account.wallet_address,
                NonFungibleGlobalId::new(base.membership_resource_address, membership_id.clone()),
            )
            .create_proof_from_auth_zone_of_non_fungibles(
                base.membership_resource_address,
                vec![membership_id.clone()],
                "membership_proof",
            )
            .call_method_with_name_lookup(
                collection_component,
                "donate_update_with_membership",
                |lookup| {
                    (
                        lookup.bucket("donation_amount"),
                        lookup.proof("trophy_proof"),
                        lookup.proof("membership_proof"),
                    )
                },
            )
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_update_with_membership_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        let trophy_data: Trophy = base
            .test_runner
            .get_non_fungible_data(base.trophy_resource_address, trophy_id.clone());

        let result = AddressBech32Encoder::new(&NetworkDefinition::simulator())
            .encode(&collection_component.to_vec())
            .unwrap();

        assert_eq!(trophy_data.collection_id, result);

        assert_eq!(trophy_data.name, "Trophy name");
        assert_eq!(trophy_data.description, "Kansulers trophy");
        assert_eq!(trophy_data.creator_slug, "kansuler");
        assert_eq!(trophy_data.creator_name, "Kansuler");
        assert_eq!(
            trophy_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(trophy_data.created, "1970-01-01");
        assert_eq!(trophy_data.donated, dec!(300));

        assert_eq!(
            trophy_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=300&created=1970-01-01",
                trophy_data.collection_id
            ))
        );

        let membership_data: Membership = base
            .test_runner
            .get_non_fungible_data(base.membership_resource_address, membership_id.clone());

        assert_eq!(membership_data.name, "Membership: Kansuler");
        assert_eq!(
            membership_data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler".to_owned())
        );
        assert_eq!(membership_data.created, "1970-01-01");
        assert_eq!(membership_data.donated, dec!(300));

        assert_eq!(
            membership_data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/membership/{}?donated=300&created=1970-01-01",
                membership_data.creator_slug
            ))
        );
    }

    #[test]
    fn close_success() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        let donation_account = new_account(&mut base.test_runner);

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(150))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_success_2",
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
            dec!(9850)
        );

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!())
            .deposit_batch(creator_badge_account.wallet_address);

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_success_3",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .get_component_balance(creator_badge_account.wallet_address, XRD),
            dec!(10144)
        );

        let donation_account = new_account(&mut base.test_runner);

        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(100))
            .deposit_batch(donation_account.wallet_address);

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_success_4",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn close_failure_auth() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_auth_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                donation_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_auth_2",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn close_failure_closed() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_closed_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_closed_2",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_closed_3",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn close_failure_no_deposit_withdraw_funds() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account = new_account(&mut base.test_runner);
        let creator_badge_badge_id: NonFungibleGlobalId;
        {
            creator_badge_badge_id = mint_creator_badge(&mut base, &creator_badge_account);
        }

        let donation_account = new_account(&mut base.test_runner);

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_no_deposit_withdraw_funds_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(150))
            .take_from_worktop(XRD, dec!(150), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(150))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_no_deposit_withdraw_funds_2",
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
            dec!(9850)
        );

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account.wallet_address,
                creator_badge_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_no_deposit_withdraw_funds_3",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn withdraw_donations_success() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account_1 = new_account(&mut base.test_runner);
        let creator_badge_badge_id_1: NonFungibleGlobalId;
        {
            creator_badge_badge_id_1 = mint_creator_badge(&mut base, &creator_badge_account_1);
        }

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account_1.wallet_address,
                creator_badge_badge_id_1.clone(),
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "withdraw_donations_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account_1.public_key,
            )],
            true,
        );

        // Get the resource address
        let result = receipt.expect_commit_success();
        let collection_component = result.new_component_addresses()[0];

        // Attempt to withdraw with admin proof from the owner account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account_1.wallet_address,
                creator_badge_badge_id_1,
            )
            .call_method(collection_component, "withdraw_donations", manifest_args!())
            .deposit_batch(creator_badge_account_1.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "withdraw_donations_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account_1.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();
    }

    #[test]
    fn withdraw_donations_failure_auth() {
        let mut base = new_runner();

        // Create an component admin account
        let creator_badge_account_1 = new_account(&mut base.test_runner);
        let creator_badge_badge_id_1: NonFungibleGlobalId;
        {
            creator_badge_badge_id_1 = mint_creator_badge(&mut base, &creator_badge_account_1);
        }

        // Create an component admin account
        let creator_badge_account_2 = new_account(&mut base.test_runner);
        let creator_badge_badge_id_2: NonFungibleGlobalId;
        {
            creator_badge_badge_id_2 = mint_creator_badge(&mut base, &creator_badge_account_2);
        }

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account_1.wallet_address,
                creator_badge_badge_id_1.clone(),
            )
            .pop_from_auth_zone("creator_badge_proof")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        lookup.proof("creator_badge_proof"),
                        "Trophy name",
                        "Kansulers trophy",
                    )
                },
            );

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "collection_withdraw_donations_failure_auth_1",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account_1.public_key,
            )],
            true,
        );

        // Get the resource address
        let result = receipt.expect_commit(true);
        let collection_component = result.new_component_addresses()[0];

        // Attempt to withdraw with admin proof from the owner account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                creator_badge_account_2.wallet_address,
                creator_badge_badge_id_2,
            )
            .call_method(collection_component, "withdraw_donations", manifest_args!())
            .deposit_batch(creator_badge_account_2.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "collection_withdraw_donations_failure_auth_2",
            vec![NonFungibleGlobalId::from_public_key(
                &creator_badge_account_2.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }
}
