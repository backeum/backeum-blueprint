use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::{
    builder::ManifestBuilder, prelude::Secp256k1PrivateKey, prelude::Secp256k1PublicKey,
};

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAccount {
        public_key: Secp256k1PublicKey,
        _private_key: Secp256k1PrivateKey,
        wallet_address: ComponentAddress,
    }

    impl TestAccount {
        fn new(test_runner: &mut TestRunner) -> Self {
            let (public_key, _private_key, component_address) = test_runner.new_allocated_account();
            Self {
                public_key,
                _private_key,
                wallet_address: component_address,
            }
        }
    }

    struct TestSetup {
        test_runner: TestRunner,
        package_address: PackageAddress,
        repository_component: ComponentAddress,
        owner_account: TestAccount,
        owner_badge_resource_address: ResourceAddress,
        trophy_resource_address: ResourceAddress,
    }

    impl TestSetup {
        fn new() -> Self {
            let mut test_runner = TestRunner::builder().without_trace().build();

            // Create an owner account
            let owner_account = TestAccount::new(&mut test_runner);

            // Publish package
            let package_address = test_runner.compile_and_publish(this_package!());

            // Create an owner badge used for repository component.
            let manifest1 = ManifestBuilder::new()
                .new_badge_fixed(OwnerRole::None, Default::default(), dec!(1))
                .deposit_batch(owner_account.wallet_address)
                .build();

            // Execute the manifest.
            let receipt1 = test_runner.execute_manifest_ignoring_fee(
                manifest1,
                vec![NonFungibleGlobalId::from_public_key(
                    &owner_account.public_key,
                )],
            );

            let result1 = receipt1.expect_commit(true);

            // Get the repository component address.
            let owner_badge_resource_address = result1.new_resource_addresses()[0];

            // Test the repository component via the new function.
            let manifest2 = ManifestBuilder::new()
                .call_function(
                    package_address,
                    "Repository",
                    "new",
                    manifest_args!(
                        "https://localhost:8080/nft_image",
                        owner_badge_resource_address
                    ),
                )
                .try_deposit_batch_or_abort(owner_account.wallet_address)
                .build();

            // Execute the manifest.
            let receipt2 = test_runner.execute_manifest_ignoring_fee(
                manifest2,
                vec![NonFungibleGlobalId::from_public_key(
                    &owner_account.public_key,
                )],
            );

            let result2 = receipt2.expect_commit(true);

            // Get the repository component address.
            let repository_component = result2.new_component_addresses()[0];

            // Get the trophy resource address.
            let trophy_resource_address = result2.new_resource_addresses()[1];

            Self {
                test_runner,
                package_address,
                repository_component,
                owner_account,
                owner_badge_resource_address,
                trophy_resource_address,
            }
        }
    }

    #[test]
    fn repository_new() {
        TestSetup::new();
    }

    #[test]
    fn repository_update_base_path() {
        let mut base = TestSetup::new();

        // Create an component admin account
        let admin_account = TestAccount::new(&mut base.test_runner);
        // Create donation account
        let donation_account = TestAccount::new(&mut base.test_runner);

        // Create a donation component
        let manifest1 = ManifestBuilder::new()
            .call_method(
                base.repository_component,
                "new_donation_component",
                manifest_args!(),
            )
            .deposit_batch(admin_account.wallet_address)
            .build();

        // Execute it
        let receipt1 = base.test_runner.execute_manifest_ignoring_fee(
            manifest1,
            vec![NonFungibleGlobalId::from_public_key(
                &admin_account.public_key,
            )],
        );

        // Get the resource address
        let donation_component = receipt1.expect_commit(true).new_component_addresses()[0];

        // Donate and mint trophy
        let manifest2 = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, RADIX_TOKEN, dec!(100))
            .take_from_worktop(RADIX_TOKEN, dec!(100), "donation_amount")
            .call_method_with_name_lookup(donation_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account.wallet_address, "trophy")
            .build();

        let receipt2 = base.test_runner.execute_manifest_ignoring_fee(
            manifest2,
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt2.expect_commit_success();
        assert_eq!(
            base.test_runner.account_balance(
                donation_account.wallet_address,
                base.trophy_resource_address
            ),
            Some(dec!(1))
        );
        assert_eq!(
            base.test_runner
                .account_balance(donation_account.wallet_address, RADIX_TOKEN),
            Some(dec!(9900))
        );

        // Get the Non fungible id out of the stack
        let trophy_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.trophy_resource_address,
        );
        let vault_content = base
            .test_runner
            .inspect_non_fungible_vault(trophy_vault[0])
            .unwrap();
        assert_eq!(vault_content.0, dec!(1));
        let trophy_id = vault_content.1.unwrap();

        // Test rejection to update the base path with a donation account
        let manifest3 = ManifestBuilder::new()
            .create_proof_from_account_of_amount(
                donation_account.wallet_address,
                base.owner_badge_resource_address,
                dec!(1),
            )
            .call_method(
                base.repository_component,
                "update_base_path",
                manifest_args!("https://some_other_url/nft_image", vec![trophy_id.clone()]),
            )
            .build();

        let receipt3 = base.test_runner.execute_manifest_ignoring_fee(
            manifest3,
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt3.expect_commit_failure();

        // Test rejection to update the base path with a non owner account
        let manifest4 = ManifestBuilder::new()
            .create_proof_from_account_of_amount(
                admin_account.wallet_address,
                base.owner_badge_resource_address,
                dec!(1),
            )
            .call_method(
                base.repository_component,
                "update_base_path",
                manifest_args!("https://some_other_url/nft_image", vec![trophy_id.clone()]),
            )
            .build();

        let receipt4 = base.test_runner.execute_manifest_ignoring_fee(
            manifest4,
            vec![NonFungibleGlobalId::from_public_key(
                &admin_account.public_key,
            )],
        );
        receipt4.expect_commit_failure();

        // Test rejection to update the base path with a non owner account
        let manifest5 = ManifestBuilder::new()
            .create_proof_from_account_of_amount(
                base.owner_account.wallet_address,
                base.owner_badge_resource_address,
                dec!(1),
            )
            .call_method(
                base.repository_component,
                "update_base_path",
                manifest_args!("https://some_other_url/nft_image", vec![trophy_id.clone()]),
            )
            .build();

        let receipt5 = base.test_runner.execute_manifest_ignoring_fee(
            manifest5,
            vec![NonFungibleGlobalId::from_public_key(
                &base.owner_account.public_key,
            )],
        );
        let result = receipt5.expect_commit(true);
        let vaults = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.trophy_resource_address,
        );

        // TODO: Check metadata is correct on NFT.
    }

    #[test]
    fn repository_donate_mint_update() {
        let mut base = TestSetup::new();

        // Create an component admin account
        let admin_account = TestAccount::new(&mut base.test_runner);
        // Create donation account
        let donation_account = TestAccount::new(&mut base.test_runner);
        let donation_account_wrong_nft = TestAccount::new(&mut base.test_runner);

        // Create a donation component
        let manifest = ManifestBuilder::new()
            .call_method(
                base.repository_component,
                "new_donation_component",
                manifest_args!(),
            )
            .deposit_batch(admin_account.wallet_address)
            .build();

        // Execute it
        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &admin_account.public_key,
            )],
        );

        // Get the resource address
        let donation_component = receipt.expect_commit(true).new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, RADIX_TOKEN, dec!(100))
            .take_from_worktop(RADIX_TOKEN, dec!(100), "donation_amount")
            .call_method_with_name_lookup(donation_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account.wallet_address, "trophy")
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_success();
        assert_eq!(
            base.test_runner.account_balance(
                donation_account.wallet_address,
                base.trophy_resource_address
            ),
            Some(dec!(1))
        );
        assert_eq!(
            base.test_runner
                .account_balance(donation_account.wallet_address, RADIX_TOKEN),
            Some(dec!(9900))
        );

        // Get the Non fungible id out of the stack
        let trophy_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.trophy_resource_address,
        );
        let vault_content = base
            .test_runner
            .inspect_non_fungible_vault(trophy_vault[0])
            .unwrap();
        assert_eq!(vault_content.0, dec!(1));
        let trophy_id = vault_content.1.unwrap();

        // Donate and update trophy
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungibles(
                donation_account.wallet_address,
                base.trophy_resource_address,
                &btreeset!(trophy_id.clone()),
            )
            .pop_from_auth_zone("trophy")
            .withdraw_from_account(donation_account.wallet_address, RADIX_TOKEN, dec!(100))
            .take_from_worktop(RADIX_TOKEN, dec!(100), "donation_amount")
            .call_method_with_name_lookup(donation_component, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("trophy"))
            })
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_success();
        // TODO: Check metadata is correct on NF when it comes to amount donated.

        // Donate and update trophy with the wrong account, should fail, admin_account does not have
        // the NFT in account.
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungibles(
                admin_account.wallet_address,
                base.trophy_resource_address,
                &btreeset!(trophy_id.clone()),
            )
            .pop_from_auth_zone("trophy")
            .withdraw_from_account(donation_account.wallet_address, RADIX_TOKEN, dec!(100))
            .take_from_worktop(RADIX_TOKEN, dec!(100), "donation_amount")
            .call_method_with_name_lookup(donation_component, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("trophy"))
            })
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Donate and mint trophy with new account, and attempt fake update wrong NF id.
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                donation_account_wrong_nft.wallet_address,
                RADIX_TOKEN,
                dec!(100),
            )
            .take_from_worktop(RADIX_TOKEN, dec!(100), "donation_amount")
            .call_method_with_name_lookup(donation_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account_wrong_nft.wallet_address, "trophy")
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account_wrong_nft.public_key,
            )],
        );

        receipt.expect_commit_success();

        // Donate and update trophy
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungibles(
                donation_account_wrong_nft.wallet_address,
                base.trophy_resource_address,
                &btreeset!(trophy_id.clone()),
            )
            .pop_from_auth_zone("trophy")
            .withdraw_from_account(
                donation_account_wrong_nft.wallet_address,
                RADIX_TOKEN,
                dec!(100),
            )
            .take_from_worktop(RADIX_TOKEN, dec!(100), "donation_amount")
            .call_method_with_name_lookup(donation_component, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("trophy"))
            })
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account_wrong_nft.public_key,
            )],
        );

        receipt.expect_commit_failure();
    }

    #[test]
    fn donation_withdraw_donations() {
        let mut base = TestSetup::new();

        // Create an component admin account
        let admin_account = TestAccount::new(&mut base.test_runner);
        // Create donation account
        let no_access_account = TestAccount::new(&mut base.test_runner);

        // Create a donation component
        let manifest = ManifestBuilder::new()
            .call_method(
                base.repository_component,
                "new_donation_component",
                manifest_args!(),
            )
            .deposit_batch(admin_account.wallet_address)
            .build();

        // Execute it
        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &admin_account.public_key,
            )],
        );

        // Get the resource address
        let result = receipt.expect_commit(true);
        let donation_component = result.new_component_addresses()[0];
        let admin_badge_resource = result.new_resource_addresses()[0];

        // Donate and mint trophy with the no access account
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(no_access_account.wallet_address, RADIX_TOKEN, dec!(100))
            .take_from_worktop(RADIX_TOKEN, dec!(100), "donation_amount")
            .call_method_with_name_lookup(donation_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(no_access_account.wallet_address, "trophy")
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &no_access_account.public_key,
            )],
        );

        receipt.expect_commit_success();
        assert_eq!(
            base.test_runner.account_balance(
                no_access_account.wallet_address,
                base.trophy_resource_address
            ),
            Some(dec!(1))
        );
        assert_eq!(
            base.test_runner
                .account_balance(no_access_account.wallet_address, RADIX_TOKEN),
            Some(dec!(9900))
        );
        let rdx_vaults = base
            .test_runner
            .get_component_vaults(donation_component, RADIX_TOKEN)[1];
        assert_eq!(
            base.test_runner.inspect_vault_balance(rdx_vaults),
            Some(dec!(100))
        );

        // Attempt to withdraw donations with the no access account by creating proof of admin badge
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_amount(
                no_access_account.wallet_address,
                admin_badge_resource,
                dec!(1),
            )
            .call_method(donation_component, "withdraw_donations", manifest_args!())
            .deposit_batch(no_access_account.wallet_address)
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &no_access_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Attempt to withdraw donations without any proof
        let manifest = ManifestBuilder::new()
            .call_method(donation_component, "withdraw_donations", manifest_args!())
            .deposit_batch(no_access_account.wallet_address)
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &no_access_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Attempt to withdraw with admin proof from the owner account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_amount(
                admin_account.wallet_address,
                admin_badge_resource,
                dec!(1),
            )
            .call_method(donation_component, "withdraw_donations", manifest_args!())
            .deposit_batch(admin_account.wallet_address)
            .build();

        let receipt = base.test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &admin_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .account_balance(admin_account.wallet_address, RADIX_TOKEN),
            Some(dec!(10100))
        );

        let rdx_vaults = base
            .test_runner
            .get_component_vaults(donation_component, RADIX_TOKEN)[1];
        assert_eq!(
            base.test_runner.inspect_vault_balance(rdx_vaults),
            Some(dec!(0))
        );
    }
}
