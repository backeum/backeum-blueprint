#[path = "./common.rs"]
mod common;
use common::{execute_manifest, mint_collection_owner_badge, new_account, new_runner};

use scrypto::prelude::*;
use transaction::builder::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_royalties_success() {
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

        // Create a donation component
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
            "claim_royalties_success_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .inspect_package_royalty(base.package_address)
                .unwrap(),
            dec!(83.33333333333333333)
        );

        // Get the resource address
        let collection_component = receipt.expect_commit(true).new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .lock_fee(donation_account.wallet_address, 100)
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .assert_worktop_contains(base.trophy_resource_address, dec!(1))
            .assert_worktop_contains(base.thanks_token_resource_address, dec!(100))
            .deposit_batch(donation_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "claim_royalties_success_2",
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
            false,
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .inspect_package_royalty(base.package_address)
                .unwrap(),
            dec!(83.33333333333333333)
        );

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                base.owner_account.wallet_address,
                base.package_owner_badge_global_id,
            )
            .claim_package_royalties(base.package_address)
            .create_proof_from_account_of_non_fungible(
                base.owner_account.wallet_address,
                base.repository_owner_badge_global_id,
            )
            .call_method(collection_component, "withdraw_fees", manifest_args!())
            .assert_worktop_contains_any(XRD)
            .deposit_batch(base.owner_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "claim_royalties_success_3",
            vec![NonFungibleGlobalId::from_public_key(
                &base.owner_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .get_component_balance(base.owner_account.wallet_address, XRD),
            dec!(10087.33333333333333333)
        );
    }

    #[test]
    fn claim_royalties_failure_auth() {
        let mut base = new_runner();

        // Create an component admin account
        let collection_admin_account = new_account(&mut base.test_runner);
        let collection_admin_badge_id: NonFungibleGlobalId;
        {
            collection_admin_badge_id =
                mint_collection_owner_badge(&mut base, &collection_admin_account);
        }

        // Create a donation component
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
            "claim_royalties_failure_auth_1",
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .inspect_package_royalty(base.package_address)
                .unwrap(),
            dec!(83.33333333333333333)
        );

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                base.owner_account.wallet_address,
                base.repository_owner_badge_global_id,
            )
            .claim_package_royalties(base.package_address)
            .assert_worktop_contains_any(XRD)
            .deposit_batch(base.owner_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "claim_royalties_failure_auth_2",
            vec![NonFungibleGlobalId::from_public_key(
                &base.owner_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_badge_id,
            )
            .claim_package_royalties(base.package_address)
            .assert_worktop_contains_any(XRD)
            .deposit_batch(base.owner_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "claim_royalties_failure_auth_3",
            vec![NonFungibleGlobalId::from_public_key(
                &base.owner_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn claim_royalties_failure_empty() {
        let mut base = new_runner();

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                base.owner_account.wallet_address,
                base.package_owner_badge_global_id,
            )
            .claim_package_royalties(base.package_address)
            .assert_worktop_contains_any(XRD)
            .deposit_batch(base.owner_account.wallet_address);

        let receipt = execute_manifest(
            &mut base.test_runner,
            manifest,
            "claim_royalties_failure_empty_1",
            vec![NonFungibleGlobalId::from_public_key(
                &base.owner_account.public_key,
            )],
            true,
        );

        receipt.expect_commit_failure();
    }
}
