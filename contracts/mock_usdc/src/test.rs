extern crate std;

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::contract::{MockUsdcToken, MockUsdcTokenClient};

#[test]
fn test_mint_and_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let contract_id = env.register(MockUsdcToken, (&admin,));
    let client = MockUsdcTokenClient::new(&env, &contract_id);

    // Mint to user1
    client.mint(&user1, &1_000_000); // 1 USDC (6 decimals)
    assert_eq!(client.balance(&user1), 1_000_000);

    // Transfer from user1 to user2
    client.transfer(&user1, &user2, &400_000);
    assert_eq!(client.balance(&user1), 600_000);
    assert_eq!(client.balance(&user2), 400_000);
}

#[test]
fn test_metadata() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let contract_id = env.register(MockUsdcToken, (&admin,));
    let client = MockUsdcTokenClient::new(&env, &contract_id);

    assert_eq!(client.decimals(), 6);
    assert_eq!(client.symbol(), String::from_str(&env, "USDC"));
    assert_eq!(client.name(), String::from_str(&env, "Mock USDC"));
}

#[test]
fn test_total_supply() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register(MockUsdcToken, (&admin,));
    let client = MockUsdcTokenClient::new(&env, &contract_id);

    assert_eq!(client.total_supply(), 0);

    client.mint(&user, &5_000_000);
    assert_eq!(client.total_supply(), 5_000_000);
}
