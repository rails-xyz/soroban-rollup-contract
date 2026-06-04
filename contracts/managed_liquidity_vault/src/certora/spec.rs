#![allow(dead_code)]

use crate::contract::{ContractError, ManagedLiquidityVaultContract};
use cvlr::asserts::{cvlr_assert, cvlr_satisfy};
use cvlr_soroban_derive::rule;
use soroban_sdk::{Address, Env};

#[rule]
fn sanity(env: Env, addr: Address) {
    let _ = ManagedLiquidityVaultContract::exchange(env);
    let _ = addr;
    cvlr_satisfy!(true);
}

#[rule]
fn deposit_rejects_non_positive_amount(env: Env, amount_xlm: i128) {
    if amount_xlm <= 0 {
        let result = ManagedLiquidityVaultContract::deposit_partner(env, amount_xlm);
        cvlr_assert!(result == Err(ContractError::DepositAmountMustBePositive));
    }
}

#[rule]
fn withdraw_principal_rejects_non_positive_amount(env: Env, to: Address, amount_xlm: i128) {
    if amount_xlm <= 0 {
        let result = ManagedLiquidityVaultContract::withdraw_partner_principal(env, to, amount_xlm);
        cvlr_assert!(result == Err(ContractError::WithdrawAmountMustBePositive));
    }
}

#[rule]
fn set_reserve_rejects_negative_target(env: Env, target_reserved_xlm: i128) {
    if target_reserved_xlm < 0 {
        let result =
            ManagedLiquidityVaultContract::set_reserve(env, target_reserved_xlm, 0, 0, None);
        cvlr_assert!(result == Err(ContractError::TargetReserveMustBeNonNegative));
    }
}

#[rule]
fn set_reserve_rejects_negative_audit_values(
    env: Env,
    target_reserved_xlm: i128,
    reference_credit_usdt0: i128,
    exchange_rate: i128,
) {
    if target_reserved_xlm >= 0 && (reference_credit_usdt0 < 0 || exchange_rate < 0) {
        let result = ManagedLiquidityVaultContract::set_reserve(
            env,
            target_reserved_xlm,
            reference_credit_usdt0,
            exchange_rate,
            None,
        );
        cvlr_assert!(result == Err(ContractError::AuditValuesMustBeNonNegative));
    }
}

#[rule]
fn record_yield_settlement_rejects_negative_amounts(
    env: Env,
    epoch_id: u64,
    yield_due_usdt0: i128,
    yield_paid_usdt0: i128,
) {
    if yield_due_usdt0 < 0 || yield_paid_usdt0 < 0 {
        let result = ManagedLiquidityVaultContract::record_yield_settlement(
            env,
            epoch_id,
            yield_due_usdt0,
            yield_paid_usdt0,
            None,
        );
        cvlr_assert!(result == Err(ContractError::YieldSettlementAmountsMustBeNonNegative));
    }
}

#[rule]
fn pay_yield_rejects_non_positive_amount(env: Env, from: Address, amount_usdt0: i128) {
    if amount_usdt0 <= 0 {
        let result = ManagedLiquidityVaultContract::pay_yield(env, from, amount_usdt0);
        cvlr_assert!(result == Err(ContractError::YieldAmountMustBePositive));
    }
}

#[rule]
fn withdraw_yield_rejects_non_positive_amount(env: Env, to: Address, amount_usdt0: i128) {
    if amount_usdt0 <= 0 {
        let result = ManagedLiquidityVaultContract::withdraw_partner_yield(env, to, amount_usdt0);
        cvlr_assert!(result == Err(ContractError::YieldAmountMustBePositive));
    }
}

#[rule]
fn renounce_ownership_is_disabled(env: Env) {
    let result = ManagedLiquidityVaultContract::renounce_ownership(env);
    cvlr_assert!(result == Err(ContractError::RenounceOwnershipDisabled));
}
