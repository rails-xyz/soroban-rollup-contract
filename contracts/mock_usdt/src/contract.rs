use soroban_sdk::{contract, contractimpl, Address, Env, String};
use stellar_access::ownable;
use stellar_macros::only_owner;
use stellar_tokens::fungible::Base;

#[contract]
pub struct MockUsdtToken;

#[contractimpl]
impl MockUsdtToken {
    pub fn __constructor(env: Env, admin: Address) {
        ownable::set_owner(&env, &admin);
        Base::set_metadata(
            &env,
            6,
            String::from_str(&env, "Mock USDT"),
            String::from_str(&env, "USDT"),
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
