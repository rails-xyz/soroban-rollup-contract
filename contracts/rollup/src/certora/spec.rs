#![allow(dead_code)]

use crate::contract::{ContractError, DataKey, RollupContract};
use cvlr::asserts::{cvlr_assert, cvlr_assume, cvlr_satisfy};
use cvlr_soroban_derive::rule;
use soroban_sdk::{Address, BytesN, Env, Vec};

/// Returns the zero hash, the value the contract rejects as a new block hash.
fn zero_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0; 32])
}

/// Reads the configured collateral token.
fn collateral_token(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::CollateralToken)
        .unwrap()
}

#[rule]
fn sanity(env: Env, user: Address) {
    let _ = RollupContract::block_height(env.clone());
    let _ = RollupContract::withdrawal_allowances(env, user);
    cvlr_satisfy!(true);
}

#[rule]
fn deposit_rejects_non_positive_amount(env: Env, user: Address, amount: i128) {
    if amount <= 0 {
        let result = RollupContract::deposit(env, user, amount);
        cvlr_assert!(result == Err(ContractError::DepositAmountMustBePositive));
    }
}

#[rule]
fn rollup_rejects_negative_fees(env: Env, new_withdrawal_sum: i128, new_fees: i128) {
    if new_fees < 0 {
        let result = RollupContract::rollup(
            env.clone(),
            zero_hash(&env),
            zero_hash(&env),
            Vec::new(&env),
            Vec::new(&env),
            new_withdrawal_sum,
            new_fees,
        );
        cvlr_assert!(result == Err(ContractError::FeesMustBeNonNegative));
    }
}

#[rule]
fn rollup_rejects_empty_new_block_hash(env: Env, new_withdrawal_sum: i128, new_fees: i128) {
    cvlr_assume!(new_fees >= 0);
    let result = RollupContract::rollup(
        env.clone(),
        zero_hash(&env),
        zero_hash(&env),
        Vec::new(&env),
        Vec::new(&env),
        new_withdrawal_sum,
        new_fees,
    );
    cvlr_assert!(result == Err(ContractError::NewBlockHashEmpty));
}

#[rule]
fn withdraw_rejects_missing_allowance(env: Env, user: Address) {
    let allowance = RollupContract::withdrawal_allowances(env.clone(), user.clone());
    if allowance <= 0 {
        let result = RollupContract::withdraw(env, user);
        cvlr_assert!(result == Err(ContractError::NoWithdrawalAllowance));
    }
}

#[rule]
fn collect_fees_rejects_empty_fee_balance(env: Env, to: Address) {
    let fees = RollupContract::fees(env.clone());
    if fees <= 0 {
        let result = RollupContract::collect_fees(env, to);
        cvlr_assert!(result == Err(ContractError::NoFeesToCollect));
    }
}

#[rule]
fn recover_rejects_collateral_token(env: Env, to: Address, amount: i128) {
    let token = collateral_token(&env);
    let result = RollupContract::recover(env, token, to, amount);
    cvlr_assert!(result == Err(ContractError::CannotRecoverCollateral));
}

#[rule]
fn recover_rejects_non_positive_amount(env: Env, token: Address, to: Address, amount: i128) {
    cvlr_assume!(token != collateral_token(&env));
    if amount <= 0 {
        let result = RollupContract::recover(env, token, to, amount);
        cvlr_assert!(result == Err(ContractError::RecoverAmountMustBePositive));
    }
}

#[rule]
fn renounce_ownership_always_rejected(env: Env) {
    let result = RollupContract::renounce_ownership(env);
    cvlr_assert!(result == Err(ContractError::RenounceOwnershipDisabled));
}
