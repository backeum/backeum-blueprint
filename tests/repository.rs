use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_repository_create_donation_contract() {
    // Setup the environment
    let mut test_runner = TestRunner::builder().without_trace().build();

    // Create an owner account
    let (public_key, _private_key, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "Repository",
            "new",
            manifest_args!("https://localhost:8080/nft_image"),
        )
        .deposit_batch(account)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let repository_component = receipt.expect_commit(true).new_component_addresses()[0];
    let donor_badge_address = receipt.expect_commit(true).new_resource_addresses()[0];
    let resources = test_runner.get_component_resources(repository_component);

    println!("resources: {:?}", resources);
    println!("donor_badge_address: {:?}", donor_badge_address);

    // Create an owner account
    let (public_key2, _, account2) = test_runner.new_allocated_account();

    let manifest2 = ManifestBuilder::new()
        .call_method(
            repository_component,
            "new_donation_component",
            manifest_args!(),
        )
        .deposit_batch(account2)
        .build();

    let receipt2 = test_runner.execute_manifest_ignoring_fee(
        manifest2,
        vec![NonFungibleGlobalId::from_public_key(&public_key2)],
    );

    let donation_component = receipt2.expect_commit(true).new_component_addresses()[0];

    // Create an owner account
    let (public_key3, _, account3) = test_runner.new_allocated_account();

    let manifest3 = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(account3, donor_badge_address, &btreeset!())
        .pop_from_auth_zone("donor_badge")
        .withdraw_from_account(account3, RADIX_TOKEN, dec!(100))
        .take_from_worktop(RADIX_TOKEN, dec!(100), "donation_amount")
        .call_method_with_name_lookup(donation_component, "donate", |lookup| {
            (lookup.bucket("donation_amount"), lookup.proof("donor_badge"))
        })
        .call_method(repository_component, "mint", manifest_args!("id_test"))
        .deposit_batch(account3)
        .build();

    let receipt3 = test_runner.execute_manifest_ignoring_fee(
        manifest3,
        vec![NonFungibleGlobalId::from_public_key(&public_key3)],
    );
    receipt3.expect_commit(true);
}
