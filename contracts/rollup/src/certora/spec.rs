#![allow(dead_code)]

//! Formal specification rules for the Certora Prover.
//!
//! # Modelling limits
//!
//! The prover treats the host comparison of two `Address` or `BytesN` objects
//! as uninterpreted, and every storage read builds a fresh object. A rule that
//! reads a value out of storage and expects the contract to compare it as equal
//! to the same value therefore has a spurious counterexample. Where a guard
//! rests on such a comparison, the rule asserts that the call fails rather than
//! which guard rejects it. The block-hash guards of `rollup` and the
//! collateral-token guard of `recover` stay covered by the unit tests.
//!
//! A collateral transfer is a call into the token contract, and the prover does
//! not converge on a rule that puts one on the path to the property. Each rule
//! below therefore asserts a property that a single call establishes.

use crate::contract::{ContractError, RollupContract};
use cvlr::asserts::{cvlr_assert, cvlr_assume, cvlr_satisfy};
use cvlr_soroban_derive::rule;
use soroban_sdk::{Address, BytesN, Env, Vec};

/// Returns the zero hash, the value the contract rejects as a new block hash.
fn zero_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0; 32])
}

/// Returns the empty withdrawal vectors the `rollup` rules post.
///
/// The prover reads the length of a vector as a symbolic host value, so it
/// unrolls the withdrawal loop of `rollup` and reports an unwinding condition
/// unless the length is pinned. The vectors are empty by construction, so the
/// assumption adds no unsoundness and leaves the loop body unreachable.
fn empty_withdrawals(env: &Env) -> (Vec<Address>, Vec<i128>) {
    let addresses: Vec<Address> = Vec::new(env);
    let amounts: Vec<i128> = Vec::new(env);
    cvlr_assume!(addresses.is_empty());
    cvlr_assume!(amounts.is_empty());
    (addresses, amounts)
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
        let (addresses, amounts) = empty_withdrawals(&env);
        let result = RollupContract::rollup(
            env.clone(),
            zero_hash(&env),
            zero_hash(&env),
            addresses,
            amounts,
            new_withdrawal_sum,
            new_fees,
        );
        cvlr_assert!(result == Err(ContractError::FeesMustBeNonNegative));
    }
}

/// A withdrawal pays out the full allowance.
///
/// `withdraw` transfers the whole stored allowance and removes the entry, so
/// nothing is left to withdraw a second time.
#[rule]
fn withdraw_clears_allowance(env: Env, user: Address) {
    let result = RollupContract::withdraw(env.clone(), user.clone());
    if result.is_ok() {
        cvlr_assert!(RollupContract::withdrawal_allowances(env, user) == 0);
    }
}

/// A fee collection pays out the full fee balance.
///
/// `collect_fees` transfers the whole accrued balance and resets it, so nothing
/// is left to collect a second time.
#[rule]
fn collect_fees_clears_balance(env: Env, to: Address) {
    let result = RollupContract::collect_fees(env.clone(), to);
    if result.is_ok() {
        cvlr_assert!(RollupContract::fees(env) == 0);
    }
}

/// A recovery of a non-positive amount never transfers.
///
/// The rule asserts the rejection and not the specific error, because the
/// collateral-token guard runs first and rests on an address comparison the
/// prover cannot resolve.
#[rule]
fn recover_rejects_non_positive_amount(env: Env, token: Address, to: Address, amount: i128) {
    if amount <= 0 {
        let result = RollupContract::recover(env, token, to, amount);
        cvlr_assert!(result.is_err());
    }
}

#[rule]
fn renounce_ownership_always_rejected(env: Env) {
    let result = RollupContract::renounce_ownership(env);
    cvlr_assert!(result == Err(ContractError::RenounceOwnershipDisabled));
}
