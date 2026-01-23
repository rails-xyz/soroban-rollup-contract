use soroban_sdk::{contract, contractimpl, Address, Env, String};
use stellar_access::ownable;
use stellar_macros::only_owner;
use stellar_tokens::fungible::Base;

#[contract]
pub struct MockUsdcToken;

#[contractimpl]
impl MockUsdcToken {
    pub fn __constructor(env: Env, admin: Address) {
        ownable::set_owner(&env, &admin);
        Base::set_metadata(
            &env,
            6,
            String::from_str(&env, "Mock USDC"),
            String::from_str(&env, "USDC"),
        );
    }

    #[only_owner]
    pub fn mint(env: Env, to: Address, amount: i128) {
        Base::mint(&env, &to, amount);
    }

    // SEP-41 Token Interface
    pub fn total_supply(env: Env) -> i128 {
        Base::total_supply(&env)
    }

    pub fn balance(env: Env, account: Address) -> i128 {
        Base::balance(&env, &account)
    }

    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        Base::allowance(&env, &owner, &spender)
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        Base::transfer(&env, &from, &to, amount);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        Base::transfer_from(&env, &spender, &from, &to, amount);
    }

    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        Base::approve(&env, &owner, &spender, amount, expiration_ledger);
    }

    pub fn decimals(env: Env) -> u32 {
        Base::decimals(&env)
    }

    pub fn name(env: Env) -> String {
        Base::name(&env)
    }

    pub fn symbol(env: Env) -> String {
        Base::symbol(&env)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

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
}
