extern crate std;

use soroban_sdk::{
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    vec, Address, BytesN, Env,
};

use super::contract::{RollupContract, RollupContractClient};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (TokenClient<'a>, StellarAssetClient<'a>) {
    let address = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    (
        TokenClient::new(env, &address),
        StellarAssetClient::new(env, &address),
    )
}

fn deploy_fixture(
    env: &Env,
) -> (
    Address,
    Address,
    Address,
    TokenClient<'_>,
    RollupContractClient<'_>,
) {
    env.mock_all_auths();

    let owner = Address::generate(env);
    let other_account = Address::generate(env);
    let fee_account = Address::generate(env);

    let token_admin = Address::generate(env);
    let (token_client, token_admin_client) = create_token_contract(env, &token_admin);

    let contract_id = env.register(RollupContract, (&token_client.address, &owner));
    let client = RollupContractClient::new(env, &contract_id);

    // Mint some tokens to other_account and owner
    token_admin_client.mint(&other_account, &20_0000000);
    token_admin_client.mint(&owner, &20_0000000);

    (owner, other_account, fee_account, token_client, client)
}

#[test]
fn test_deployment() {
    let env = Env::default();
    let (_owner, _other_account, _fee_account, _token_client, client) = deploy_fixture(&env);
    // Just check it deploys - verify initial state
    assert_eq!(
        client.latest_block_hash(),
        BytesN::from_array(&env, &[0; 32])
    );
    assert_eq!(client.fees(), 0);
    assert_eq!(client.total_withdrawable(), 0);
}

#[test]
fn test_deposit() {
    let env = Env::default();
    let (_owner, other_account, _fee_account, token_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_0000000; // 10 tokens

    // Approve spending
    token_client.approve(&other_account, &client.address, &deposit_amount, &1000000); // ledger bump

    client.deposit(&other_account, &deposit_amount);

    let contract_balance = token_client.balance(&client.address);
    assert_eq!(contract_balance, deposit_amount);
}

#[test]
#[should_panic(expected = "Deposit amount must be greater than 0")]
fn test_deposit_zero_amount() {
    let env = Env::default();
    let (_owner, other_account, _fee_account, _token_client, client) = deploy_fixture(&env);

    client.deposit(&other_account, &0);
}

#[test]
fn test_rollup() {
    let env = Env::default();
    let (_owner, other_account, _fee_account, token_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_0000000;
    let withdrawal_amount = 5_0000000;
    let fee_amount = 500000;

    token_client.approve(&other_account, &client.address, &deposit_amount, &1000000);
    client.deposit(&other_account, &deposit_amount);

    let old_root = client.latest_block_hash();
    let new_root = BytesN::from_array(&env, &[1; 32]);

    client.rollup(
        &old_root,
        &new_root,
        &vec![&env, other_account.clone()],
        &vec![&env, withdrawal_amount],
        &withdrawal_amount,
        &fee_amount,
    );

    let allowance = client.withdrawal_allowances(&other_account);
    assert_eq!(allowance, withdrawal_amount);
    assert_eq!(client.fees(), fee_amount);
}

#[test]
#[should_panic]
fn test_rollup_non_owner() {
    let env = Env::default();
    // Don't use deploy_fixture since it calls mock_all_auths
    // Setup without mocking auth to test authorization failure
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_client, _token_admin_client) = create_token_contract(&env, &token_admin);

    let contract_id = env.register(RollupContract, (&token_client.address, &owner));
    let client = RollupContractClient::new(&env, &contract_id);

    let old_root = client.latest_block_hash();
    let new_root = BytesN::from_array(&env, &[1; 32]);

    // This should fail auth - no mock_all_auths called, so owner auth will fail
    client.rollup(&old_root, &new_root, &vec![&env], &vec![&env], &0, &0);
}

#[test]
fn test_withdraw() {
    let env = Env::default();
    let (_owner, other_account, _fee_account, token_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_0000000;
    let withdrawal_amount = 5_0000000;
    let fee_amount = 500000;

    token_client.approve(&other_account, &client.address, &deposit_amount, &1000000);
    client.deposit(&other_account, &deposit_amount);

    let old_root = client.latest_block_hash();
    let new_root = BytesN::from_array(&env, &[1; 32]);

    client.rollup(
        &old_root,
        &new_root,
        &vec![&env, other_account.clone()],
        &vec![&env, withdrawal_amount],
        &withdrawal_amount,
        &fee_amount,
    );

    // Now withdraw
    let initial_balance = token_client.balance(&other_account);
    client.withdraw(&other_account);
    let final_balance = token_client.balance(&other_account);
    assert_eq!(final_balance, initial_balance + withdrawal_amount);
}

#[test]
fn test_collect_fees() {
    let env = Env::default();
    let (owner, fee_account, _other_account, token_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_0000000;
    let fee_amount = 500000;

    token_client.approve(&owner, &client.address, &deposit_amount, &1000000);
    client.deposit(&owner, &deposit_amount);

    let old_root = client.latest_block_hash();
    let new_root = BytesN::from_array(&env, &[1; 32]);

    client.rollup(
        &old_root,
        &new_root,
        &vec![&env],
        &vec![&env],
        &0,
        &fee_amount,
    );

    let initial_balance = token_client.balance(&fee_account);
    client.collect_fees(&fee_account);
    let final_balance = token_client.balance(&fee_account);
    assert_eq!(final_balance, initial_balance + fee_amount);
}

#[test]
fn test_withdraw_verifies_auth() {
    // This test verifies that withdraw() requires user authorization
    // by checking that the correct auth entries are recorded
    let env = Env::default();
    let (_owner, other_account, _fee_account, token_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_0000000;
    let withdrawal_amount = 5_0000000;

    token_client.approve(&other_account, &client.address, &deposit_amount, &1000000);
    client.deposit(&other_account, &deposit_amount);

    let old_root = client.latest_block_hash();
    let new_root = BytesN::from_array(&env, &[1; 32]);

    client.rollup(
        &old_root,
        &new_root,
        &vec![&env, other_account.clone()],
        &vec![&env, withdrawal_amount],
        &withdrawal_amount,
        &0,
    );

    // Clear previous auth records
    let _ = env.auths();

    // Call withdraw
    client.withdraw(&other_account);

    // Verify that authorization was required from the user
    let auths = env.auths();
    assert!(!auths.is_empty(), "withdraw should require authorization");

    // Check that the auth was for the correct user (other_account)
    let (auth_address, _) = &auths[0];
    assert_eq!(
        auth_address, &other_account,
        "auth should be required from the withdrawing user"
    );
}
