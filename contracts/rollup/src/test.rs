#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::TokenClient,
    vec, Address, BytesN, Env,
};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> TokenClient<'a> {
    TokenClient::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

fn deploy_fixture(env: &Env) -> (Address, Address, Address, TokenClient, RollupContractClient) {
    let owner = Address::generate(env);
    let other_account = Address::generate(env);
    let fee_account = Address::generate(env);

    let token_admin = Address::generate(env);
    let token_client = create_token_contract(env, &token_admin);

    let contract_id = env.register(RollupContract, ());
    let client = RollupContractClient::new(env, &contract_id);

    client.initialize(&token_client.address, &owner);

    // Mint some tokens to other_account
    token_client.mint(&other_account, &20_0000000); // 20 tokens with 7 decimals? Wait, Stellar assets have 7 decimals usually.

    // Actually, Soroban tokens can have different decimals, but for simplicity, assume 7.

    (owner, other_account, fee_account, token_client, client)
}

#[test]
fn test_deployment() {
    let env = Env::default();
    let (_owner, _other_account, _fee_account, _token_client, client) = deploy_fixture(&env);
    // Just check it deploys
    assert!(client.owner() == _owner);
}

#[test]
fn test_deposit() {
    let env = Env::default();
    let (owner, other_account, _fee_account, token_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_0000000; // 10 tokens

    // Approve spending
    token_client.approve(&other_account, &client.address, &deposit_amount, &1000000); // ledger bump

    client.deposit(&other_account, &deposit_amount);

    let contract_balance = token_client.balance(&client.address);
    assert_eq!(contract_balance, deposit_amount);
}

#[test]
fn test_deposit_zero_amount() {
    let env = Env::default();
    let (_owner, other_account, _fee_account, _token_client, client) = deploy_fixture(&env);

    // This should panic
    let result = std::panic::catch_unwind(|| {
        client.deposit(&other_account, &0);
    });
    assert!(result.is_err());
}

#[test]
fn test_rollup() {
    let env = Env::default();
    let (owner, other_account, _fee_account, token_client, client) = deploy_fixture(&env);

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
fn test_rollup_non_owner() {
    let env = Env::default();
    let (_owner, other_account, _fee_account, _token_client, client) = deploy_fixture(&env);

    let old_root = client.latest_block_hash();
    let new_root = BytesN::from_array(&env, &[1; 32]);

    // This should fail auth
    let result = std::panic::catch_unwind(|| {
        client.rollup(&old_root, &new_root, &vec![&env], &vec![&env], &0, &0);
    });
    assert!(result.is_err());
}

#[test]
fn test_withdraw() {
    let env = Env::default();
    let (owner, other_account, _fee_account, token_client, client) = deploy_fixture(&env);

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
    client.withdraw();
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

// Add more tests as needed, similar to the TS ones.
