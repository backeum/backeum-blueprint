use backeum_blueprint::data::Trophy;
use radix_engine::transaction::TransactionReceipt;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::manifest::decompiler::ManifestObjectNames;
use transaction::{
    builder::ManifestBuilder, prelude::Secp256k1PrivateKey, prelude::Secp256k1PublicKey,
    prelude::TransactionManifestV1,
};

#[derive(ScryptoSbor, ManifestSbor, NonFungibleData)]
struct Nft {
    name: String,
    description: String,
    icon_url: UncheckedUrl,
    info_url: UncheckedUrl,
    tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Execute {}

    impl Execute {
        fn execute_manifest_ignoring_fee<T>(
            test_runner: &mut DefaultTestRunner,
            manifest_names: ManifestObjectNames,
            manifest: TransactionManifestV1,
            name: &str,
            network: &NetworkDefinition,
            initial_proofs: T,
        ) -> TransactionReceipt
        where
            T: IntoIterator<Item = NonFungibleGlobalId>,
        {
            if name != "" {
                dump_manifest_to_file_system(
                    manifest_names,
                    &manifest,
                    "./transaction-manifest",
                    Some(name),
                    network,
                )
                .err();
            }

            test_runner.execute_manifest_ignoring_fee(manifest, initial_proofs)
        }

        fn execute_manifest<T>(
            test_runner: &mut DefaultTestRunner,
            manifest_names: ManifestObjectNames,
            manifest: TransactionManifestV1,
            name: &str,
            network: &NetworkDefinition,
            initial_proofs: T,
        ) -> TransactionReceipt
        where
            T: IntoIterator<Item = NonFungibleGlobalId>,
        {
            if name != "" {
                dump_manifest_to_file_system(
                    manifest_names,
                    &manifest,
                    "./transaction-manifest",
                    Some(name),
                    network,
                )
                .err();
            }

            test_runner.execute_manifest(manifest, initial_proofs)
        }
    }

    struct TestAccount {
        public_key: Secp256k1PublicKey,
        _private_key: Secp256k1PrivateKey,
        wallet_address: ComponentAddress,
        backeum_collection_owner_badge_global_id: NonFungibleGlobalId,
    }

    impl TestAccount {
        fn new(test_runner: &mut DefaultTestRunner) -> Self {
            let (public_key, _private_key, component_address) = test_runner.new_allocated_account();

            let mut metadata = BTreeMap::new();
            metadata.insert(
                "name".to_owned(),
                MetadataValue::String("Backeum Collection Owner Badge".to_owned()),
            );
            metadata.insert(
                "description".to_owned(),
                MetadataValue::String(
                    "Grants collection ownership of Backeum components".to_owned(),
                ),
            );
            metadata.insert(
                "info_url".to_owned(),
                MetadataValue::Url(UncheckedUrl("https://staging.backeum.com".to_owned())),
            );
            metadata.insert(
                "tags".to_owned(),
                MetadataValue::StringArray(vec!["backeum".to_owned(), "owner".to_owned()]),
            );
            metadata.insert(
                "icon_url".to_string(),
                MetadataValue::Url(UncheckedUrl(
                    "https://staging.backeum.com/bucket/assets/wallet-assets/admin-badge.png"
                        .to_owned(),
                )),
            );

            let metadata = ModuleConfig {
                init: metadata.into(),
                roles: RoleAssignmentInit::default(),
            };

            // Create an owner badge used for repository component.
            let manifest = ManifestBuilder::new()
                .create_ruid_non_fungible_resource(
                    OwnerRole::None,
                    false,
                    metadata,
                    Default::default(),
                    Some([Nft {
                        name: "Owner Badge".to_owned(),
                        description: "Owner badge for Backeum collections".to_owned(),
                        icon_url: UncheckedUrl::of(
                            "https://staging.backeum.com/bucket/assets/wallet-assets/admin-badge.png"
                                .to_owned(),
                        ),
                        info_url: UncheckedUrl::of("https://staging.backeum.com".to_owned()),
                        tags: vec!["backeum".to_owned(), "badge".to_owned()],
                    }]
                    ))
                .deposit_batch(component_address);

            // Execute the manifest.
            let receipt = Execute::execute_manifest_ignoring_fee(
                test_runner,
                manifest.object_names(),
                manifest.build(),
                "create_collection_owner_badge",
                &NetworkDefinition::simulator(),
                vec![NonFungibleGlobalId::from_public_key(&public_key)],
            );

            let result = receipt.expect_commit(true);

            // Get the repository component address.
            let backeum_collection_owner_badge_resource_address =
                result.new_resource_addresses()[0];

            let backeum_collection_owner_badge_vault = test_runner.get_component_vaults(
                component_address,
                backeum_collection_owner_badge_resource_address,
            );
            let backeum_collection_owner_badge_id = test_runner
                .inspect_non_fungible_vault(backeum_collection_owner_badge_vault[0])
                .unwrap()
                .1
                .next()
                .unwrap();
            let backeum_collection_owner_badge_global_id = NonFungibleGlobalId::new(
                backeum_collection_owner_badge_resource_address,
                backeum_collection_owner_badge_id.clone(),
            );

            Self {
                public_key,
                _private_key,
                wallet_address: component_address,
                backeum_collection_owner_badge_global_id,
            }
        }
    }

    struct TestSetup {
        test_runner: DefaultTestRunner,
        repository_component: ComponentAddress,
        owner_account: TestAccount,
        package_address: PackageAddress,
        // package_owner_badge_resource_address: ResourceAddress,
        package_owner_badge_global_id: NonFungibleGlobalId,
        component_owner_badge_resource_address: ResourceAddress,
        component_owner_badge_global_id: NonFungibleGlobalId,
        trophy_resource_address: ResourceAddress,
    }

    impl TestSetup {
        fn new() -> Self {
            let mut test_runner = TestRunnerBuilder::new().without_trace().build();

            // Create an owner account
            let owner_account = TestAccount::new(&mut test_runner);

            let mut package_owner_badge_metadata = BTreeMap::new();
            package_owner_badge_metadata.insert(
                "name".to_owned(),
                MetadataValue::String("Backeum Package Owner Badges".to_owned()),
            );
            package_owner_badge_metadata.insert(
                "description".to_owned(),
                MetadataValue::String("Grants package ownership of backeum packages".to_owned()),
            );
            package_owner_badge_metadata.insert(
                "info_url".to_owned(),
                MetadataValue::Url(UncheckedUrl("https://staging.backeum.com".to_owned())),
            );
            package_owner_badge_metadata.insert(
                "tags".to_owned(),
                MetadataValue::StringArray(vec!["backeum".to_owned()]),
            );
            package_owner_badge_metadata.insert(
                "icon_url".to_string(),
                MetadataValue::Url(UncheckedUrl(
                    "https://staging.backeum.com/bucket/assets/wallet-assets/package-owner-badge.png"
                        .to_owned(),
                )),
            );
            package_owner_badge_metadata.insert(
                "dapp_definitions".to_string(),
                MetadataValue::GlobalAddressArray(vec![GlobalAddress::new_or_panic(
                    owner_account.wallet_address.into(),
                )]),
            );

            let package_owner_badge_metadata = ModuleConfig {
                init: package_owner_badge_metadata.into(),
                roles: RoleAssignmentInit::default(),
            };

            let mut component_owner_badge_metadata = BTreeMap::new();
            component_owner_badge_metadata.insert(
                "name".to_owned(),
                MetadataValue::String("Backeum Component Owner Badges".to_owned()),
            );
            component_owner_badge_metadata.insert(
                "description".to_owned(),
                MetadataValue::String(
                    "Grants component ownership of backeum repository and collection components"
                        .to_owned(),
                ),
            );
            component_owner_badge_metadata.insert(
                "info_url".to_owned(),
                MetadataValue::Url(UncheckedUrl("https://staging.backeum.com".to_owned())),
            );
            component_owner_badge_metadata.insert(
                "tags".to_owned(),
                MetadataValue::StringArray(vec!["backeum".to_owned()]),
            );
            component_owner_badge_metadata.insert(
                "icon_url".to_string(),
                MetadataValue::Url(UncheckedUrl(
                    "https://staging.backeum.com/bucket/assets/wallet-assets/component-owner-badge.png"
                        .to_owned(),
                )),
            );
            component_owner_badge_metadata.insert(
                "dapp_definitions".to_string(),
                MetadataValue::GlobalAddressArray(vec![GlobalAddress::new_or_panic(
                    owner_account.wallet_address.into(),
                )]),
            );

            let component_owner_badge_metadata = ModuleConfig {
                init: component_owner_badge_metadata.into(),
                roles: RoleAssignmentInit::default(),
            };

            // Create an owner badge used for repository component.
            let manifest = ManifestBuilder::new()
                .create_ruid_non_fungible_resource(
                    OwnerRole::None,
                    false,
                    package_owner_badge_metadata,
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
                .create_ruid_non_fungible_resource(
                    OwnerRole::None,
                    false,
                    component_owner_badge_metadata,
                    Default::default(),
                    Some([Nft {
                        name: "Badge".to_owned(),
                        description: "Owner badge for components instantiated on Backeum"
                            .to_owned(),
                        icon_url: UncheckedUrl::of(
                            "https://staging.backeum.com/bucket/assets/wallet-assets/badge.png"
                                .to_owned(),
                        ),
                        info_url: UncheckedUrl::of("https://staging.backeum.com".to_owned()),
                        tags: vec!["backeum".to_owned(), "badge".to_owned()],
                    }]),
                )
                .deposit_batch(owner_account.wallet_address);

            // Execute the manifest.
            let receipt = Execute::execute_manifest_ignoring_fee(
                &mut test_runner,
                manifest.object_names(),
                manifest.build(),
                "create_package_owner_badge",
                &NetworkDefinition::simulator(),
                vec![NonFungibleGlobalId::from_public_key(
                    &owner_account.public_key,
                )],
            );

            let result = receipt.expect_commit(true);

            // Get the repository component address.
            let package_owner_badge_resource_address = result.new_resource_addresses()[0];

            let package_owner_badge_vault = test_runner.get_component_vaults(
                owner_account.wallet_address,
                package_owner_badge_resource_address,
            );
            let package_owner_badge_id = test_runner
                .inspect_non_fungible_vault(package_owner_badge_vault[0])
                .unwrap()
                .1
                .next()
                .unwrap();
            let package_owner_badge_global_id = NonFungibleGlobalId::new(
                package_owner_badge_resource_address,
                package_owner_badge_id.clone(),
            );

            // Get the repository component address.
            let component_owner_badge_resource_address = result.new_resource_addresses()[1];

            let component_owner_badge_vault = test_runner.get_component_vaults(
                owner_account.wallet_address,
                component_owner_badge_resource_address,
            );
            let component_owner_badge_id = test_runner
                .inspect_non_fungible_vault(component_owner_badge_vault[0])
                .unwrap()
                .1
                .next()
                .unwrap();
            let component_owner_badge_global_id = NonFungibleGlobalId::new(
                component_owner_badge_resource_address,
                component_owner_badge_id.clone(),
            );

            // Publish package
            let package_address = test_runner.compile_and_publish_with_owner(
                this_package!(),
                package_owner_badge_global_id.clone(),
            );

            // Test the repository component via the new function.
            let manifest = ManifestBuilder::new()
                .call_function(
                    package_address,
                    "Repository",
                    "new",
                    manifest_args!(
                        "https://localhost:8080",
                        component_owner_badge_resource_address,
                        owner_account.wallet_address,
                    ),
                )
                .try_deposit_batch_or_abort(
                    owner_account.wallet_address,
                    ManifestExpression::EntireWorktop,
                    None,
                );

            // Execute the manifest.
            let receipt = Execute::execute_manifest_ignoring_fee(
                &mut test_runner,
                manifest.object_names(),
                manifest.build(),
                "instantiate_new_repository",
                &NetworkDefinition::simulator(),
                vec![NonFungibleGlobalId::from_public_key(
                    &owner_account.public_key,
                )],
            );

            let result = receipt.expect_commit(true);

            // Get the repository component address.
            let repository_component = result.new_component_addresses()[0];

            // Minter badge resource address
            let minter_badge_resource_address = result.new_resource_addresses()[0];

            // Get the trophy resource address.
            let trophy_resource_address = result.new_resource_addresses()[1];

            // Set metadata on dapp definition
            let manifest = ManifestBuilder::new()
                .set_metadata(
                    owner_account.wallet_address,
                    "account_type",
                    MetadataValue::String("dapp definition".to_string()),
                )
                .set_metadata(
                    owner_account.wallet_address,
                    "claimed_entities",
                    MetadataValue::GlobalAddressArray(vec![
                        GlobalAddress::new_or_panic(package_address.into()),
                        GlobalAddress::new_or_panic(repository_component.into()),
                        GlobalAddress::new_or_panic(minter_badge_resource_address.into()),
                        GlobalAddress::new_or_panic(trophy_resource_address.into()),
                        GlobalAddress::new_or_panic(package_owner_badge_resource_address.into()),
                        GlobalAddress::new_or_panic(component_owner_badge_resource_address.into()),
                    ]),
                )
                .set_metadata(
                    owner_account.wallet_address,
                    "claimed_websites",
                    MetadataValue::OriginArray(vec![UncheckedOrigin(
                        "https://staging.backeum.com".to_string(),
                    )]),
                )
                .create_proof_from_account_of_non_fungible(
                    owner_account.wallet_address,
                    package_owner_badge_global_id.clone(),
                )
                .set_metadata(
                    package_address,
                    "dapp_definition",
                    MetadataValue::GlobalAddress(GlobalAddress::new_or_panic(
                        owner_account.wallet_address.into(),
                    )),
                );

            // Execute the manifest.
            let receipt = Execute::execute_manifest_ignoring_fee(
                &mut test_runner,
                manifest.object_names(),
                manifest.build(),
                "set_dapp_account_metadata",
                &NetworkDefinition::simulator(),
                vec![NonFungibleGlobalId::from_public_key(
                    &owner_account.public_key,
                )],
            );

            receipt.expect_commit_success();

            Self {
                test_runner,
                repository_component,
                owner_account,
                package_address,
                // package_owner_badge_resource_address,
                package_owner_badge_global_id,
                component_owner_badge_resource_address,
                component_owner_badge_global_id,
                trophy_resource_address,
            }
        }
    }

    #[test]
    fn update_base_path() {
        let mut base = TestSetup::new();

        // Create an component admin account
        let collection_admin_account = TestAccount::new(&mut base.test_runner);
        // Create donation account
        let donation_account = TestAccount::new(&mut base.test_runner);

        // Create a donation component
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .pop_from_auth_zone("collection_owner_badge")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_owner_badge"),
                    )
                },
            )
            .deposit_batch(collection_admin_account.wallet_address);

        // Execute the manifest.
        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "instantiate_new_collection_component",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
        );

        // Get the resource address
        let collection_component = receipt.expect_commit(true).new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
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
            dec!(9900)
        );

        // Get the Non fungible id out of the stack
        let trophy_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.trophy_resource_address,
        );
        let (amount, _) = base
            .test_runner
            .inspect_non_fungible_vault(trophy_vault[0])
            .unwrap();
        assert_eq!(amount, dec!(1));

        let trophy_id = base
            .test_runner
            .inspect_non_fungible_vault(trophy_vault[0])
            .unwrap()
            .1
            .next()
            .unwrap();

        // Test rejection to update the base path with a donation account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                donation_account.wallet_address,
                donation_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .create_proof_from_account_of_amount(
                donation_account.wallet_address,
                base.component_owner_badge_resource_address,
                dec!(1),
            )
            .call_method(
                base.repository_component,
                "update_base_path",
                manifest_args!("https://some_other_url/nft_image", vec![trophy_id.clone()]),
            );

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "update_base_path_repository",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Test rejection to update the base path with a non owner account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                base.component_owner_badge_global_id.clone(),
            )
            .call_method(
                base.repository_component,
                "update_base_path",
                manifest_args!("https://some_other_url/nft_image", vec![trophy_id.clone()]),
            );

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
        );
        receipt.expect_commit_failure();

        // Test rejection to update the base path with a non owner account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                base.owner_account.wallet_address,
                base.component_owner_badge_global_id.clone(),
            )
            .call_method(
                base.repository_component,
                "update_base_path",
                manifest_args!("https://some_other_url/nft_image", vec![trophy_id.clone()]),
            );

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &base.owner_account.public_key,
            )],
        );
        receipt.expect_commit_success();
    }

    #[test]
    fn donate_and_merge() {
        let mut base = TestSetup::new();

        // Create an component admin account
        let collection_admin_account = TestAccount::new(&mut base.test_runner);
        // Create donation account
        let donation_account = TestAccount::new(&mut base.test_runner);
        let donation_account_wrong_nft = TestAccount::new(&mut base.test_runner);

        // Create two collection components
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .pop_from_auth_zone("collection_owner_badge_1")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler_1",
                        "kansuler_1",
                        lookup.proof("collection_owner_badge_1"),
                    )
                },
            )
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .pop_from_auth_zone("collection_owner_badge_2")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler_2",
                        "kansuler_2",
                        lookup.proof("collection_owner_badge_2"),
                    )
                },
            )
            .deposit_batch(collection_admin_account.wallet_address);

        // Execute it
        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
        );

        // Get the resource address
        let collection_component_1 = receipt.expect_commit(true).new_component_addresses()[0];
        let collection_component_2 = receipt.expect_commit(true).new_component_addresses()[1];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(200))
            .take_from_worktop(XRD, dec!(150), "donation_amount_1")
            .take_from_worktop(XRD, dec!(50), "donation_amount_2")
            .call_method_with_name_lookup(collection_component_1, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_1"),)
            })
            .call_method_with_name_lookup(collection_component_2, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_2"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "donation_mint",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
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
            base.test_runner
                .get_component_balance(donation_account.wallet_address, XRD),
            dec!(9800)
        );
        let trophy_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.trophy_resource_address,
        );

        let trophy_id_1: NonFungibleLocalId;
        let trophy_id_2: NonFungibleLocalId;
        {
            let mut trophies = base
                .test_runner
                .inspect_non_fungible_vault(trophy_vault[0])
                .unwrap()
                .1;

            trophy_id_2 = trophies.next().unwrap().clone();
            trophy_id_1 = trophies.next().unwrap().clone();
        }
        println!("Trophy id 1: {:?}", trophy_id_1);
        println!("Trophy id 2: {:?}", trophy_id_2);

        let trophy_data_1: Trophy = base
            .test_runner
            .get_non_fungible_data(base.trophy_resource_address, trophy_id_1.clone());

        let trophy_data_2: Trophy = base
            .test_runner
            .get_non_fungible_data(base.trophy_resource_address, trophy_id_2.clone());

        assert_ne!(trophy_data_1.collection_id, trophy_data_2.collection_id);

        assert_eq!(trophy_data_1.name, "Backer Trophy: Kansuler_2");
        assert_eq!(
            trophy_data_1.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler_2".to_owned())
        );
        assert_eq!(trophy_data_1.created, "1970-01-01");
        assert_eq!(trophy_data_1.donated, dec!(70));

        assert_eq!(
            trophy_data_1.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=70&created=1970-01-01",
                trophy_data_1.collection_id
            ))
        );

        assert_eq!(trophy_data_2.name, "Backer Trophy: Kansuler_1");
        assert_eq!(
            trophy_data_2.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler_1".to_owned())
        );
        assert_eq!(trophy_data_2.created, "1970-01-01");
        assert_eq!(trophy_data_2.donated, dec!(170));
        assert_eq!(
            trophy_data_2.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=170&created=1970-01-01",
                trophy_data_2.collection_id
            ))
        );

        // Donate and update trophy
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungibles(
                donation_account.wallet_address,
                base.trophy_resource_address,
                btreeset!(trophy_id_2.clone()),
            )
            .pop_from_auth_zone("trophy")
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component_1, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("trophy"))
            });

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "donation_update",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        let data: Trophy = base
            .test_runner
            .get_non_fungible_data(base.trophy_resource_address, trophy_id_2.clone());

        assert_eq!(data.name, "Backer Trophy: Kansuler_1");
        assert_eq!(
            data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler_1".to_owned())
        );
        assert_eq!(data.created, "1970-01-01");
        assert_eq!(data.donated, dec!(290));
        assert_eq!(
            data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=290&created=1970-01-01",
                data.collection_id
            ))
        );

        // Donate and update trophy with the wrong account, should fail, admin_account does not have
        // the NFT in account.
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungibles(
                collection_admin_account.wallet_address,
                base.trophy_resource_address,
                btreeset!(trophy_id_2.clone()),
            )
            .pop_from_auth_zone("trophy")
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component_1, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("trophy"))
            });

        // Execute it
        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Donate and mint trophy with new account, and attempt fake update wrong NF id.
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account_wrong_nft.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component_1, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account_wrong_nft.wallet_address, None, "trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
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
                btreeset!(trophy_id_2.clone()),
            )
            .pop_from_auth_zone("trophy")
            .withdraw_from_account(donation_account_wrong_nft.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component_1, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("trophy"))
            });

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account_wrong_nft.public_key,
            )],
        );
        receipt.expect_commit_failure();

        // Merge multiple trophies into one
        let manifest = ManifestBuilder::new()
            .withdraw_non_fungibles_from_account(
                donation_account.wallet_address,
                base.trophy_resource_address,
                btreeset!(trophy_id_1.clone(), trophy_id_2.clone()),
            )
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(500))
            .take_from_worktop(XRD, dec!(500), "donation_amount")
            .call_method_with_name_lookup(collection_component_1, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophies")
            .call_method_with_name_lookup(base.repository_component, "merge_trophies", |lookup| {
                (lookup.bucket("trophies"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "new_trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "new_trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "merge_trophies",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Merge multiple trophies into one
        let manifest = ManifestBuilder::new()
            .withdraw_non_fungibles_from_account(
                donation_account.wallet_address,
                base.trophy_resource_address,
                btreeset!(trophy_id_2.clone()),
            )
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(1000))
            .take_from_worktop(XRD, dec!(700), "donation_amount_1")
            .call_method_with_name_lookup(collection_component_1, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_1"),)
            })
            .take_from_worktop(XRD, dec!(300), "donation_amount_2")
            .call_method_with_name_lookup(collection_component_1, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount_2"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophies")
            .call_method_with_name_lookup(base.repository_component, "merge_trophies", |lookup| {
                (lookup.bucket("trophies"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "new_trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "new_trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "merge_trophies",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        let trophy_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.trophy_resource_address,
        );

        let trophy_id_3: NonFungibleLocalId;
        {
            let (length, mut trophies) = base
                .test_runner
                .inspect_non_fungible_vault(trophy_vault[0])
                .unwrap();

            assert_eq!(length, dec!(2));

            trophy_id_3 = trophies.next().unwrap().clone();
        }

        let data: Trophy = base
            .test_runner
            .get_non_fungible_data(base.trophy_resource_address, trophy_id_3.clone());

        assert_eq!(data.name, "Backer Trophy: Kansuler_1");
        assert_eq!(
            data.info_url,
            UncheckedUrl::of("https://localhost:8080/p/kansuler_1".to_owned())
        );
        assert_eq!(data.created, "1970-01-01");
        assert_eq!(data.donated, dec!(1330));
        assert_eq!(
            data.key_image_url,
            UncheckedUrl::of(format!(
                "https://localhost:8080/nft/collection/{}?donated=1330&created=1970-01-01",
                data.collection_id
            ))
        );
    }

    #[test]
    fn withdraw_donations() {
        let mut base = TestSetup::new();

        // Create an component admin account
        let collection_admin_account = TestAccount::new(&mut base.test_runner);
        // Create donation account
        let no_access_account = TestAccount::new(&mut base.test_runner);

        // Create a donation component
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .pop_from_auth_zone("collection_admin_badge")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_admin_badge"),
                    )
                },
            );

        // Execute it
        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
        );

        // Get the resource address
        let result = receipt.expect_commit(true);
        let collection_component = result.new_component_addresses()[0];

        // Donate and mint trophy with the no access account
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(no_access_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(no_access_account.wallet_address, None, "trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &no_access_account.public_key,
            )],
        );

        receipt.expect_commit_success();
        assert_eq!(
            base.test_runner.get_component_balance(
                no_access_account.wallet_address,
                base.trophy_resource_address
            ),
            dec!(1)
        );
        assert_eq!(
            base.test_runner
                .get_component_balance(no_access_account.wallet_address, XRD),
            dec!(9900)
        );
        let rdx_vaults = base
            .test_runner
            .get_component_vaults(collection_component, XRD)[0];
        assert_eq!(
            base.test_runner.inspect_vault_balance(rdx_vaults),
            Some(dec!(100))
        );

        // Attempt to withdraw donations with the no access account by creating proof of admin badge
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                no_access_account.wallet_address,
                no_access_account.backeum_collection_owner_badge_global_id,
            )
            .call_method(collection_component, "withdraw_donations", manifest_args!())
            .deposit_batch(no_access_account.wallet_address);

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "withdraw_donations_no_access",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &no_access_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Attempt to withdraw donations without any proof
        let manifest = ManifestBuilder::new()
            .call_method(collection_component, "withdraw_donations", manifest_args!())
            .deposit_batch(no_access_account.wallet_address);

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &no_access_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Attempt to withdraw with admin proof from the owner account
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .call_method(collection_component, "withdraw_donations", manifest_args!())
            .deposit_batch(collection_admin_account.wallet_address);

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "withdraw_donations_admin",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .get_component_balance(collection_admin_account.wallet_address, XRD),
            dec!(10100)
        );

        let rdx_vaults = base
            .test_runner
            .get_component_vaults(collection_component, XRD)[0];
        assert_eq!(
            base.test_runner.inspect_vault_balance(rdx_vaults),
            Some(dec!(0))
        );
    }

    #[test]
    fn claim_royalties() {
        let mut base = TestSetup::new();

        // Create an component admin account
        let collection_admin_account = TestAccount::new(&mut base.test_runner);
        // Create donation account
        let donation_account = TestAccount::new(&mut base.test_runner);

        // Create a donation component
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .pop_from_auth_zone("collection_owner_badge")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_owner_badge"),
                    )
                },
            )
            .deposit_batch(collection_admin_account.wallet_address);

        // Execute it
        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .inspect_package_royalty(base.package_address)
                .unwrap(),
            dec!(50)
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
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "trophy");

        let receipt = Execute::execute_manifest(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_success();
        assert_eq!(
            base.test_runner
                .inspect_package_royalty(base.package_address)
                .unwrap(),
            dec!(70)
        );

        // Get the Non fungible id out of the stack
        let trophy_vault = base.test_runner.get_component_vaults(
            donation_account.wallet_address,
            base.trophy_resource_address,
        );
        let (amount, _) = base
            .test_runner
            .inspect_non_fungible_vault(trophy_vault[0])
            .unwrap();
        assert_eq!(amount, dec!(1));

        let trophy_id = base
            .test_runner
            .inspect_non_fungible_vault(trophy_vault[0])
            .unwrap()
            .1
            .next()
            .unwrap();

        // Donate and update trophy
        let manifest = ManifestBuilder::new()
            .lock_fee(donation_account.wallet_address, 100)
            .create_proof_from_account_of_non_fungibles(
                donation_account.wallet_address,
                base.trophy_resource_address,
                btreeset!(trophy_id.clone()),
            )
            .pop_from_auth_zone("trophy")
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("trophy"))
            });

        let receipt = Execute::execute_manifest(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .inspect_package_royalty(base.package_address)
                .unwrap(),
            dec!(90)
        );

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                base.owner_account.wallet_address,
                base.component_owner_badge_global_id,
            )
            .claim_package_royalties(base.package_address)
            .deposit_batch(base.owner_account.wallet_address);

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &base.owner_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                base.owner_account.wallet_address,
                base.package_owner_badge_global_id,
            )
            .claim_package_royalties(base.package_address)
            .deposit_batch(base.owner_account.wallet_address);

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "claim_royalties",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &base.owner_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        assert_eq!(
            base.test_runner
                .get_component_balance(base.owner_account.wallet_address, XRD),
            dec!(10090)
        );
    }

    #[test]
    fn close_collection() {
        let mut base = TestSetup::new();

        // Create an component admin account
        let collection_admin_account = TestAccount::new(&mut base.test_runner);
        // Create an component admin account for testing auth
        let collection_admin_account_wrong_badge = TestAccount::new(&mut base.test_runner);
        // Create donation account
        let donation_account = TestAccount::new(&mut base.test_runner);

        // Create a collection component
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .pop_from_auth_zone("collection_owner_badge")
            .call_method_with_name_lookup(
                base.repository_component,
                "new_collection_component",
                |lookup| {
                    (
                        "Kansuler",
                        "kansuler",
                        lookup.proof("collection_owner_badge"),
                    )
                },
            );

        // Execute the manifest.
        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "instantiate_new_collection_component",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
        );

        // Get the resource address
        let collection_component = receipt.expect_commit(true).new_component_addresses()[0];

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        let trophy_id: NonFungibleLocalId;
        {
            // Get the Non fungible id out of the stack
            let trophy_vault = base.test_runner.get_component_vaults(
                donation_account.wallet_address,
                base.trophy_resource_address,
            );
            let (amount, mut trophies) = base
                .test_runner
                .inspect_non_fungible_vault(trophy_vault[0])
                .unwrap();
            assert_eq!(amount, dec!(1));

            trophy_id = trophies.next().unwrap();
        }

        // Close the collection wrong badge
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account_wrong_badge.wallet_address,
                collection_admin_account_wrong_badge
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account_wrong_badge.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Close the collection
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungible(
                collection_admin_account.wallet_address,
                collection_admin_account
                    .backeum_collection_owner_badge_global_id
                    .clone(),
            )
            .call_method(collection_component, "close_collection", manifest_args!());

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "close_collection",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &collection_admin_account.public_key,
            )],
        );

        receipt.expect_commit_success();

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Donate and mint trophy
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_mint", |lookup| {
                (lookup.bucket("donation_amount"),)
            })
            .take_all_from_worktop(base.trophy_resource_address, "trophy")
            .try_deposit_or_abort(donation_account.wallet_address, None, "trophy");

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_failure();

        // Donate and update trophy
        let manifest = ManifestBuilder::new()
            .create_proof_from_account_of_non_fungibles(
                donation_account.wallet_address,
                base.trophy_resource_address,
                btreeset!(trophy_id.clone()),
            )
            .pop_from_auth_zone("trophy")
            .withdraw_from_account(donation_account.wallet_address, XRD, dec!(100))
            .take_from_worktop(XRD, dec!(100), "donation_amount")
            .call_method_with_name_lookup(collection_component, "donate_update", |lookup| {
                (lookup.bucket("donation_amount"), lookup.proof("trophy"))
            });

        let receipt = Execute::execute_manifest_ignoring_fee(
            &mut base.test_runner,
            manifest.object_names(),
            manifest.build(),
            "donation_update",
            &NetworkDefinition::simulator(),
            vec![NonFungibleGlobalId::from_public_key(
                &donation_account.public_key,
            )],
        );

        receipt.expect_commit_failure();
    }
}
