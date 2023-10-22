#[path = "./common.rs"]
mod common;
use common::{execute_manifest, mint_collection_owner_badge, new_account, new_runner};

use backeum_blueprint::data::Trophy;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn donate_mint_success() {
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
            "donate_mint_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
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

        assert_eq!(trophy_data.name, "Backer Trophy: Kansuler");
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
    }

    #[test]
    fn donate_mint_failure_too_low_amount() {
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

        // Create collection components
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
            "donate_mint_failure_too_low_amount_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
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
        let collection_admin_account = new_account(&mut base.test_runner);
        let collection_admin_badge_id: NonFungibleGlobalId;
        {
            collection_admin_badge_id =
                mint_collection_owner_badge(&mut base, &collection_admin_account);
        }

        // Create donation account
        let donation_account = new_account(&mut base.test_runner);

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
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
            "donate_mint_failure_closed_collection_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "donate_mint_failure_closed_collection_2",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
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
            "donate_mint_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
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
    fn close_success() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);
        let collection_admin_badge_id: NonFungibleGlobalId;
        {
            collection_admin_badge_id =
                mint_collection_owner_badge(&mut base, &collection_admin_account);
        }

        let donation_account = new_account(&mut base.test_runner);

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
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
            "close_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
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
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!())
            .deposit_batch(collection_admin_account.wallet_address);

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_success_3",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .get_component_balance(collection_admin_account.wallet_address, XRD),
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
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn close_failure_auth() {
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

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
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
            "close_failure_auth_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                donation_account.wallet_address,
                collection_admin_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_auth_2",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn close_failure_closed() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);
        let collection_admin_badge_id: NonFungibleGlobalId;
        {
            collection_admin_badge_id =
                mint_collection_owner_badge(&mut base, &collection_admin_account);
        }

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
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
            "close_failure_closed_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        // Get the resource address
        let collection_component = receipt.expect_commit_success().new_component_addresses()[0];

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_closed_2",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_closed_3",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn close_failure_no_deposit_withdraw_funds() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);
        let collection_admin_badge_id: NonFungibleGlobalId;
        {
            collection_admin_badge_id =
                mint_collection_owner_badge(&mut base, &collection_admin_account);
        }

        let donation_account = new_account(&mut base.test_runner);

        // Create collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
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
            "close_failure_no_deposit_withdraw_funds_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
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
                collection_admin_account.wallet_address,
                collection_admin_badge_id.clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        // Execute it
        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "close_failure_no_deposit_withdraw_funds_3",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn withdraw_donations_success() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account_1 = new_account(&mut base.test_runner);
        let collection_admin_badge_id_1: NonFungibleGlobalId;
        {
            collection_admin_badge_id_1 =
                mint_collection_owner_badge(&mut base, &collection_admin_account_1);
        }

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account_1.wallet_address,
                collection_admin_badge_id_1.clone(),
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
            "withdraw_donations_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account_1.public_key,
            )],
            true,
        );

        // Get the resource address
        let result = receipt.expect_commit_success();
        let collection_component = result.new_component_addresses()[0];

        // Attempt to withdraw with admin proof from the owner account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account_1.wallet_address,
                collection_admin_badge_id_1,
            )
            .call_method(collection_component, "withdraw_donations", manifest_args!())
            .deposit_batch(collection_admin_account_1.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "withdraw_donations_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account_1.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();
    }

    #[test]
    fn withdraw_donations_failure_auth() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account_1 = new_account(&mut base.test_runner);
        let collection_admin_badge_id_1: NonFungibleGlobalId;
        {
            collection_admin_badge_id_1 =
                mint_collection_owner_badge(&mut base, &collection_admin_account_1);
        }

        // Create an component admin account
        let collection_admin_account_2 = new_account(&mut base.test_runner);
        let collection_admin_badge_id_2: NonFungibleGlobalId;
        {
            collection_admin_badge_id_2 =
                mint_collection_owner_badge(&mut base, &collection_admin_account_2);
        }

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account_1.wallet_address,
                collection_admin_badge_id_1.clone(),
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
            "collection_withdraw_donations_failure_auth_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account_1.public_key,
            )],
            true,
        );

        // Get the resource address
        let result = receipt.expect_commit(true);
        let collection_component = result.new_component_addresses()[0];

        // Attempt to withdraw with admin proof from the owner account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account_2.wallet_address,
                collection_admin_badge_id_2,
            )
            .call_method(collection_component, "withdraw_donations", manifest_args!())
            .deposit_batch(collection_admin_account_2.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "collection_withdraw_donations_failure_auth_2",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account_2.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }
}
