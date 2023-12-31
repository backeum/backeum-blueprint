use radix_engine::transaction::TransactionReceipt;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::{
    builder::ManifestBuilder, prelude::Secp256k1PrivateKey, prelude::Secp256k1PublicKey,
};

#[cfg(test)]
#[derive(ScryptoSbor, ManifestSbor, NonFungibleData)]
pub struct Nft {
    pub name: String,
    pub description: String,
}

#[cfg(test)]
pub fn execute_manifest<T>(
    test_runner: &mut DefaultTestRunner,
    manifest: ManifestBuilder,
    name: &str,
    initial_proofs: T,
    ignore_fee: bool,
) -> TransactionReceipt
where
    T: IntoIterator<Item = NonFungibleGlobalId>,
{
    let manifest_objects = manifest.object_names();
    let built_manifest = manifest.build();
    {
        if name != "" {
            dump_manifest_to_file_system(
                manifest_objects,
                &built_manifest,
                "./transaction-manifest",
                Some(name),
                &NetworkDefinition::simulator(),
            )
            .err();
        }
    }

    if ignore_fee {
        test_runner.execute_manifest_ignoring_fee(built_manifest, initial_proofs)
    } else {
        test_runner.execute_manifest(built_manifest, initial_proofs)
    }
}

#[cfg(test)]
pub struct Account {
    pub public_key: Secp256k1PublicKey,
    pub _private_key: Secp256k1PrivateKey,
    pub wallet_address: ComponentAddress,
}

#[cfg(test)]
pub fn new_account(test_runner: &mut DefaultTestRunner) -> Account {
    let (public_key, _private_key, component_address) = test_runner.new_allocated_account();
    Account {
        public_key,
        _private_key,
        wallet_address: component_address,
    }
}

#[cfg(test)]
pub fn mint_creator_badge(base: &mut TestRunner, account: &Account) -> NonFungibleGlobalId {
    // Test the repository component via the new function.
    let manifest = ManifestBuilder::new()
        .call_method(
            base.repository_component,
            "mint_creator_badge",
            manifest_args!("Kansuler", "kansuler"),
        )
        .assert_worktop_contains_any(base.creator_badge_resource_address)
        .deposit_batch(account.wallet_address);

    // Execute the manifest.
    let receipt = execute_manifest(
        &mut base.test_runner,
        manifest,
        "mint_creator_badge",
        vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        true,
    );

    receipt.expect_commit_success();

    // Get the repository component vault
    let creator_badge_vault = base
        .test_runner
        .get_component_vaults(account.wallet_address, base.creator_badge_resource_address);

    let (_, iterator) = base
        .test_runner
        .inspect_non_fungible_vault(creator_badge_vault[0])
        .unwrap();

    // Get the collection owner badge
    let creator_badge_id = iterator.last().unwrap();

    // Return global ID
    NonFungibleGlobalId::new(
        base.creator_badge_resource_address,
        creator_badge_id.clone(),
    )
}

#[cfg(test)]
pub struct TestRunner {
    pub test_runner: DefaultTestRunner,
    pub repository_component: ComponentAddress,
    pub owner_account: Account,
    pub package_address: PackageAddress,
    pub package_owner_badge_global_id: NonFungibleGlobalId,
    pub creator_badge_resource_address: ResourceAddress,
    pub repository_owner_badge_global_id: NonFungibleGlobalId,
    pub membership_resource_address: ResourceAddress,
    pub trophy_resource_address: ResourceAddress,
    pub thanks_token_resource_address: ResourceAddress,
}

#[cfg(test)]
pub fn new_runner() -> TestRunner {
    let mut genesis = CustomGenesis::default(
        Epoch::of(1u64),
        CustomGenesis::default_consensus_manager_config(),
    );

    genesis.initial_time_ms = 1699093188267; // 04 Nov 2023
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_genesis(genesis)
        .without_trace()
        .build();

    // Create an owner account
    let owner_account = new_account(&mut test_runner);

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
            "https://staging.backeum.com/bucket/assets/wallet-assets/package.png".to_owned(),
        )),
    );

    let package_owner_badge_metadata = ModuleConfig {
        init: package_owner_badge_metadata.into(),
        roles: RoleAssignmentInit::default(),
    };

    let mut repository_owner_badge_metadata = BTreeMap::new();
    repository_owner_badge_metadata.insert(
        "name".to_owned(),
        MetadataValue::String("Backeum Repository Owner Badges".to_owned()),
    );
    repository_owner_badge_metadata.insert(
        "description".to_owned(),
        MetadataValue::String(
            "Grants component ownership of backeum repository and collection components".to_owned(),
        ),
    );
    repository_owner_badge_metadata.insert(
        "info_url".to_owned(),
        MetadataValue::Url(UncheckedUrl("https://staging.backeum.com".to_owned())),
    );
    repository_owner_badge_metadata.insert(
        "tags".to_owned(),
        MetadataValue::StringArray(vec!["backeum".to_owned()]),
    );
    repository_owner_badge_metadata.insert(
        "icon_url".to_string(),
        MetadataValue::Url(UncheckedUrl(
            "https://staging.backeum.com/bucket/assets/wallet-assets/repository.png".to_owned(),
        )),
    );

    let repository_owner_badge_metadata = ModuleConfig {
        init: repository_owner_badge_metadata.into(),
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
            }]),
        )
        .create_ruid_non_fungible_resource(
            OwnerRole::None,
            false,
            repository_owner_badge_metadata,
            Default::default(),
            Some([Nft {
                name: "Badge".to_owned(),
                description: "Owner badge for components instantiated on Backeum".to_owned(),
            }]),
        )
        .deposit_batch(owner_account.wallet_address);

    // Execute the manifest.
    let receipt = execute_manifest(
        &mut test_runner,
        manifest,
        "create_package_owner_badge",
        vec![NonFungibleGlobalId::from_public_key(
            &owner_account.public_key,
        )],
        true,
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
    let repository_owner_badge_resource_address = result.new_resource_addresses()[1];

    let repository_owner_badge_vault = test_runner.get_component_vaults(
        owner_account.wallet_address,
        repository_owner_badge_resource_address,
    );
    let repository_owner_badge_id = test_runner
        .inspect_non_fungible_vault(repository_owner_badge_vault[0])
        .unwrap()
        .1
        .next()
        .unwrap();
    let repository_owner_badge_global_id = NonFungibleGlobalId::new(
        repository_owner_badge_resource_address,
        repository_owner_badge_id.clone(),
    );

    // Upload package
    let (code, definition) = Compile::compile(this_package!());
    let manifest = ManifestBuilder::new().publish_package_with_owner(
        code,
        definition,
        package_owner_badge_global_id.clone(),
    );

    // Execute the manifest.
    let receipt = execute_manifest(
        &mut test_runner,
        manifest,
        "upload_package",
        vec![NonFungibleGlobalId::from_public_key(
            &owner_account.public_key,
        )],
        true,
    );

    let package_address = receipt.expect_commit_success().new_package_addresses()[0];

    // Test the repository component via the new function.
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "Repository",
            "new",
            manifest_args!(
                "https://localhost:8080",
                repository_owner_badge_resource_address,
                owner_account.wallet_address,
            ),
        )
        .deposit_batch(owner_account.wallet_address);

    // Execute the manifest.
    let receipt = execute_manifest(
        &mut test_runner,
        manifest,
        "instantiate_new_repository",
        vec![NonFungibleGlobalId::from_public_key(
            &owner_account.public_key,
        )],
        true,
    );

    let result = receipt.expect_commit(true);

    // Get the repository component address
    let repository_component = result.new_component_addresses()[0];

    // Collection owner badge resource address
    let creator_badge_resource_address = result.new_resource_addresses()[1];

    // Get the trophy resource address.
    let trophy_resource_address = result.new_resource_addresses()[2];

    // Get the membership resource address.
    let membership_resource_address = result.new_resource_addresses()[4];

    // Get the thanks token resource address.
    let thanks_token_resource_address = result.new_resource_addresses()[3];

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
                GlobalAddress::new_or_panic(trophy_resource_address.into()),
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
    let receipt = execute_manifest(
        &mut test_runner,
        manifest,
        "set_dapp_account_metadata",
        vec![NonFungibleGlobalId::from_public_key(
            &owner_account.public_key,
        )],
        true,
    );

    receipt.expect_commit_success();

    TestRunner {
        test_runner,
        repository_component,
        owner_account,
        package_address,
        package_owner_badge_global_id,
        creator_badge_resource_address: creator_badge_resource_address,
        repository_owner_badge_global_id,
        membership_resource_address,
        trophy_resource_address,
        thanks_token_resource_address,
    }
}
