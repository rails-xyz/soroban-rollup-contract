//! # Managed Liquidity Vault Contract.
//!
//! This contract implements a single-partner managed vault where:
//! - one `FundingPartner` deposits and withdraws `XLM` principal,
//! - one `Exchange` reserves deposited `XLM` as collateral for off-chain
//!   internal credit and market making,
//! - yield is paid in a separate `USDT0`-like token by an approved payer and
//!   can only be withdrawn by the `FundingPartner`,
//! - and upgrades require both the `Exchange` and `FundingPartner` to
//!   authorize them.
//!
//! The reserve model is intentionally simple. The contract only checks that the
//! posted reserve covers the stated credit using a fixed-point exchange rate.

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, Address,
    BytesN, Env,
};
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::Upgradeable;

const RATE_SCALE: i128 = 10_000_000;

/// Approximate number of ledgers produced in one day (~5s close time).
const DAY_IN_LEDGERS: u32 = 17_280;
/// Target instance TTL set on every extension (~30 days).
const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
/// Re-extend the instance when its remaining TTL drops below this (~29 days).
const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

/// Storage keys used by the managed liquidity vault.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    XlmToken,
    YieldToken,
    Exchange,
    FundingPartner,
    PartnerPrincipalXlm,
    FreePrincipalXlm,
    ReservedForExchangeXlm,
    CollectedYieldUsdt0,
    YieldDebtUsdt0,
    LatestSettlementEpoch,
    LastSetReserveExchangeRate,
    LastSetReserveCredit,
    LastReserveReferenceHash,
    LastYieldSettlementReferenceHash,
}

/// Errors returned by the managed liquidity vault.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    DepositAmountMustBePositive = 1,
    WithdrawAmountMustBePositive = 2,
    YieldAmountMustBePositive = 3,
    TargetReserveMustBeNonNegative = 4,
    ReserveExceedsPrincipal = 5,
    InsufficientFreePrincipal = 6,
    InsufficientCollectedYield = 7,
    YieldSettlementAmountsMustBeNonNegative = 8,
    YieldPaidExceedsDebtAndCurrentDue = 9,
    SettlementEpochMustIncrease = 10,
    AuditValuesMustBeNonNegative = 13,
    Unauthorized = 15,
    ExchangeRateMustBePositive = 16,
    ReserveBelowRequiredCollateral = 17,
    ArithmeticOverflow = 19,
    PrincipalAndYieldTokenMustDiffer = 20,
    ExchangeAndPartnerMustDiffer = 21,
}

/// Managed liquidity vault contract.
#[derive(Upgradeable)]
#[contract]
pub struct ManagedLiquidityVaultContract;

/// Event emitted when the funding partner deposits principal into the vault.
#[contractevent]
#[derive(Clone)]
pub struct PartnerDepositEvt {
    pub amount: i128,
}

/// Event emitted when free principal leaves the vault.
#[contractevent]
#[derive(Clone)]
pub struct PartnerPrincipalOutEvt {
    pub to: Address,
    pub amount: i128,
}

/// Event emitted when the exchange sets the current reserve target.
#[contractevent]
#[derive(Clone)]
pub struct ReserveSetEvt {
    pub target_reserved_xlm: i128,
    pub reference_credit_usdt0: i128,
    pub exchange_rate: i128,
    pub reference_hash: Option<BytesN<32>>,
}

/// Event emitted when yield debt for an epoch is recorded.
#[contractevent]
#[derive(Clone)]
pub struct YieldSettlementEvt {
    pub epoch_id: u64,
    pub yield_due_usdt0: i128,
    pub yield_paid_usdt0: i128,
    pub reference_hash: Option<BytesN<32>>,
}

/// Event emitted when the exchange pays yield outside epoch settlement.
#[contractevent]
#[derive(Clone)]
pub struct YieldPaidEvt {
    pub amount: i128,
}

/// Event emitted when the funding partner withdraws collected yield.
#[contractevent]
#[derive(Clone)]
pub struct PartnerYieldOutEvt {
    pub to: Address,
    pub amount: i128,
}

#[contractimpl]
impl ManagedLiquidityVaultContract {
    /// Initializes the vault.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `xlm_token` - Token contract used as vault principal and collateral.
    /// * `yield_token` - Token contract used for externally paid yield.
    /// * `exchange` - Address authorized to manage reserve and yield
    ///   settlement.
    /// * `funding_partner` - Address authorized to deposit principal and
    ///   withdraw collected yield.
    ///
    /// # Notes
    ///
    /// * Upgrades require both the configured `Exchange` and
    ///   `FundingPartner`.
    /// * Principal and yield balances are initialized to zero.
    pub fn __constructor(
        env: Env,
        xlm_token: Address,
        yield_token: Address,
        exchange: Address,
        funding_partner: Address,
    ) {
        if xlm_token == yield_token {
            panic_with_error!(&env, ContractError::PrincipalAndYieldTokenMustDiffer);
        }
        if exchange == funding_partner {
            panic_with_error!(&env, ContractError::ExchangeAndPartnerMustDiffer);
        }

        let instance = env.storage().instance();
        instance.set(&DataKey::XlmToken, &xlm_token);
        instance.set(&DataKey::YieldToken, &yield_token);
        instance.set(&DataKey::Exchange, &exchange);
        instance.set(&DataKey::FundingPartner, &funding_partner);
        instance.set(&DataKey::PartnerPrincipalXlm, &0i128);
        instance.set(&DataKey::FreePrincipalXlm, &0i128);
        instance.set(&DataKey::ReservedForExchangeXlm, &0i128);
        instance.set(&DataKey::CollectedYieldUsdt0, &0i128);
        instance.set(&DataKey::YieldDebtUsdt0, &0i128);
        instance.set(&DataKey::LatestSettlementEpoch, &0u64);
        instance.set(&DataKey::LastSetReserveExchangeRate, &0i128);
        instance.set(&DataKey::LastSetReserveCredit, &0i128);
        instance.set(
            &DataKey::LastReserveReferenceHash,
            &Option::<BytesN<32>>::None,
        );
        instance.set(
            &DataKey::LastYieldSettlementReferenceHash,
            &Option::<BytesN<32>>::None,
        );

        extend_contract_ttl(&env);
    }

    /// Deposits principal into the vault.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `amount_xlm` - Amount of principal token to deposit.
    ///
    /// # Errors
    ///
    /// * [`ContractError::DepositAmountMustBePositive`] - If `amount_xlm <= 0`.
    ///
    /// # Notes
    ///
    /// * Authorization from `FundingPartner` is required.
    /// * No share token is minted because this design supports a single funding
    ///   partner only.
    pub fn deposit_partner(env: Env, amount_xlm: i128) -> Result<(), ContractError> {
        if amount_xlm <= 0 {
            return Err(ContractError::DepositAmountMustBePositive);
        }
        let funding_partner = get_address(&env, &DataKey::FundingPartner);
        funding_partner.require_auth();
        extend_contract_ttl(&env);

        xlm_client(&env).transfer(
            &funding_partner,
            &env.current_contract_address(),
            &amount_xlm,
        );

        let partner_principal = get_i128(&env, &DataKey::PartnerPrincipalXlm);
        let free_principal = get_i128(&env, &DataKey::FreePrincipalXlm);
        let new_partner_principal = checked_add(partner_principal, amount_xlm)?;
        let new_free_principal = checked_add(free_principal, amount_xlm)?;
        let instance = env.storage().instance();
        instance.set(&DataKey::PartnerPrincipalXlm, &new_partner_principal);
        instance.set(&DataKey::FreePrincipalXlm, &new_free_principal);

        env.events()
            .publish_event(&PartnerDepositEvt { amount: amount_xlm });
        Ok(())
    }

    /// Withdraws free principal from the vault.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `to` - Recipient of the withdrawn principal.
    /// * `amount_xlm` - Amount of free principal to withdraw.
    ///
    /// # Errors
    ///
    /// * [`ContractError::WithdrawAmountMustBePositive`] - If `amount_xlm <= 0`.
    /// * [`ContractError::InsufficientFreePrincipal`] - If `amount_xlm`
    ///   exceeds the unreserved principal balance.
    ///
    /// # Notes
    ///
    /// * Authorization from both `Exchange` and `FundingPartner` is required.
    /// * Reserved principal remains encumbered behind exchange credit and is
    ///   never withdrawable through this method.
    pub fn withdraw_partner_principal(
        env: Env,
        to: Address,
        amount_xlm: i128,
    ) -> Result<(), ContractError> {
        if amount_xlm <= 0 {
            return Err(ContractError::WithdrawAmountMustBePositive);
        }
        require_exchange_and_partner_auth(&env);
        extend_contract_ttl(&env);

        let free_principal = get_i128(&env, &DataKey::FreePrincipalXlm);
        if amount_xlm > free_principal {
            return Err(ContractError::InsufficientFreePrincipal);
        }

        let partner_principal = get_i128(&env, &DataKey::PartnerPrincipalXlm);
        let new_partner_principal = checked_sub(partner_principal, amount_xlm)?;
        let new_free_principal = checked_sub(free_principal, amount_xlm)?;
        let instance = env.storage().instance();
        instance.set(&DataKey::PartnerPrincipalXlm, &new_partner_principal);
        instance.set(&DataKey::FreePrincipalXlm, &new_free_principal);

        xlm_client(&env).transfer(&env.current_contract_address(), &to, &amount_xlm);
        env.events().publish_event(&PartnerPrincipalOutEvt {
            to,
            amount: amount_xlm,
        });
        Ok(())
    }

    /// Sets the exchange reserve target.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `target_reserved_xlm` - Total principal to reserve as exchange
    ///   collateral after this call.
    /// * `reference_credit_usdt0` - Off-chain exchange credit amount that the
    ///   reserve is expected to cover.
    /// * `exchange_rate` - Fixed-point price of `USDT0 per 1 XLM`, scaled by
    ///   [`RATE_SCALE`].
    /// * `reference_hash` - Optional audit reference for the off-chain reserve
    ///   calculation or credit movement.
    ///
    /// # Errors
    ///
    /// * [`ContractError::TargetReserveMustBeNonNegative`] - If
    ///   `target_reserved_xlm < 0`.
    /// * [`ContractError::AuditValuesMustBeNonNegative`] - If
    ///   `reference_credit_usdt0 < 0` or `exchange_rate < 0`.
    /// * [`ContractError::ReserveExceedsPrincipal`] - If the reserve target is
    ///   larger than total partner principal.
    /// * [`ContractError::ExchangeRateMustBePositive`] - If a positive credit
    ///   amount is posted with a zero or negative exchange rate.
    /// * [`ContractError::ReserveBelowRequiredCollateral`] - If the posted
    ///   reserve does not cover the stated credit.
    /// * [`ContractError::ArithmeticOverflow`] - If fixed-point multiplication
    ///   overflows.
    ///
    /// # Notes
    ///
    /// * Authorization from `Exchange` is required.
    /// * The contract only checks whether the posted reserve covers the stated
    ///   credit at the provided exchange rate.
    /// * `reference_hash` is audit metadata only and does not affect
    ///   authorization or balances.
    pub fn set_reserve(
        env: Env,
        target_reserved_xlm: i128,
        reference_credit_usdt0: i128,
        exchange_rate: i128,
        reference_hash: Option<BytesN<32>>,
    ) -> Result<(), ContractError> {
        if target_reserved_xlm < 0 {
            return Err(ContractError::TargetReserveMustBeNonNegative);
        }
        if reference_credit_usdt0 < 0 || exchange_rate < 0 {
            return Err(ContractError::AuditValuesMustBeNonNegative);
        }
        require_exchange_auth(&env);
        extend_contract_ttl(&env);

        let partner_principal = get_i128(&env, &DataKey::PartnerPrincipalXlm);
        if target_reserved_xlm > partner_principal {
            return Err(ContractError::ReserveExceedsPrincipal);
        }
        ensure_reserve_covers_credit(
            &env,
            target_reserved_xlm,
            reference_credit_usdt0,
            exchange_rate,
        )?;

        let new_free_principal = checked_sub(partner_principal, target_reserved_xlm)?;
        let instance = env.storage().instance();
        instance.set(&DataKey::ReservedForExchangeXlm, &target_reserved_xlm);
        instance.set(&DataKey::FreePrincipalXlm, &new_free_principal);
        instance.set(&DataKey::LastSetReserveCredit, &reference_credit_usdt0);
        instance.set(&DataKey::LastSetReserveExchangeRate, &exchange_rate);
        instance.set(&DataKey::LastReserveReferenceHash, &reference_hash);

        env.events().publish_event(&ReserveSetEvt {
            target_reserved_xlm,
            reference_credit_usdt0,
            exchange_rate,
            reference_hash,
        });
        Ok(())
    }

    /// Records yield debt and any concurrent yield payment for a settlement
    /// epoch.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `epoch_id` - Monotonically increasing settlement epoch identifier.
    /// * `yield_due_usdt0` - Additional yield obligation created by this epoch.
    /// * `yield_paid_usdt0` - Portion of total outstanding yield paid now.
    /// * `reference_hash` - Optional audit reference for the off-chain
    ///   settlement package.
    ///
    /// # Errors
    ///
    /// * [`ContractError::YieldSettlementAmountsMustBeNonNegative`] - If either
    ///   yield amount is negative.
    /// * [`ContractError::SettlementEpochMustIncrease`] - If `epoch_id` is not
    ///   greater than the previously recorded epoch.
    /// * [`ContractError::YieldPaidExceedsDebtAndCurrentDue`] - If the payment
    ///   exceeds prior debt plus current epoch due.
    ///
    /// # Notes
    ///
    /// * Authorization from `Exchange` is required.
    /// * `yield_paid_usdt0` must be backed by an actual token transfer.
    /// * Unpaid yield remains debt and is not added to collected yield.
    pub fn record_yield_settlement(
        env: Env,
        epoch_id: u64,
        yield_due_usdt0: i128,
        yield_paid_usdt0: i128,
        reference_hash: Option<BytesN<32>>,
    ) -> Result<(), ContractError> {
        if yield_due_usdt0 < 0 || yield_paid_usdt0 < 0 {
            return Err(ContractError::YieldSettlementAmountsMustBeNonNegative);
        }
        require_exchange_auth(&env);
        extend_contract_ttl(&env);

        let latest_epoch = get_u64(&env, &DataKey::LatestSettlementEpoch);
        if epoch_id <= latest_epoch {
            return Err(ContractError::SettlementEpochMustIncrease);
        }

        let current_debt = get_i128(&env, &DataKey::YieldDebtUsdt0);
        let total_obligation = checked_add(current_debt, yield_due_usdt0)?;
        if yield_paid_usdt0 > total_obligation {
            return Err(ContractError::YieldPaidExceedsDebtAndCurrentDue);
        }

        if yield_paid_usdt0 > 0 {
            yield_client(&env).transfer(
                &get_address(&env, &DataKey::Exchange),
                &env.current_contract_address(),
                &yield_paid_usdt0,
            );
        }

        let collected = get_i128(&env, &DataKey::CollectedYieldUsdt0);
        let new_collected = checked_add(collected, yield_paid_usdt0)?;
        let new_debt = checked_sub(total_obligation, yield_paid_usdt0)?;
        let instance = env.storage().instance();
        instance.set(&DataKey::CollectedYieldUsdt0, &new_collected);
        instance.set(&DataKey::YieldDebtUsdt0, &new_debt);
        instance.set(&DataKey::LatestSettlementEpoch, &epoch_id);
        instance.set(&DataKey::LastYieldSettlementReferenceHash, &reference_hash);

        env.events().publish_event(&YieldSettlementEvt {
            epoch_id,
            yield_due_usdt0,
            yield_paid_usdt0,
            reference_hash,
        });
        Ok(())
    }

    /// Pays yield into the vault outside epoch settlement.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `from` - Address supplying the yield token transfer.
    /// * `amount_usdt0` - Amount of yield token to transfer into the vault.
    ///
    /// # Errors
    ///
    /// * [`ContractError::YieldAmountMustBePositive`] - If `amount_usdt0 <= 0`.
    ///
    /// # Notes
    ///
    /// * Authorization from `from` is required at the root invocation.
    /// * This method can only improve the funding partner position by reducing
    ///   debt and/or increasing collected yield.
    pub fn pay_yield(env: Env, from: Address, amount_usdt0: i128) -> Result<(), ContractError> {
        if amount_usdt0 <= 0 {
            return Err(ContractError::YieldAmountMustBePositive);
        }
        from.require_auth();
        extend_contract_ttl(&env);

        yield_client(&env).transfer(&from, &env.current_contract_address(), &amount_usdt0);

        let debt = get_i128(&env, &DataKey::YieldDebtUsdt0);
        let collected = get_i128(&env, &DataKey::CollectedYieldUsdt0);
        let new_debt = if amount_usdt0 >= debt {
            0
        } else {
            checked_sub(debt, amount_usdt0)?
        };
        let new_collected = checked_add(collected, amount_usdt0)?;

        let instance = env.storage().instance();
        instance.set(&DataKey::YieldDebtUsdt0, &new_debt);
        instance.set(&DataKey::CollectedYieldUsdt0, &new_collected);

        env.events().publish_event(&YieldPaidEvt {
            amount: amount_usdt0,
        });
        Ok(())
    }

    /// Withdraws collected yield from the vault.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `to` - Recipient of withdrawn yield.
    /// * `amount_usdt0` - Amount of collected yield to withdraw.
    ///
    /// # Errors
    ///
    /// * [`ContractError::YieldAmountMustBePositive`] - If `amount_usdt0 <= 0`.
    /// * [`ContractError::InsufficientCollectedYield`] - If `amount_usdt0`
    ///   exceeds collected yield balance.
    ///
    /// # Notes
    ///
    /// * Authorization from `FundingPartner` is required.
    pub fn withdraw_partner_yield(
        env: Env,
        to: Address,
        amount_usdt0: i128,
    ) -> Result<(), ContractError> {
        if amount_usdt0 <= 0 {
            return Err(ContractError::YieldAmountMustBePositive);
        }
        require_partner_auth(&env);
        extend_contract_ttl(&env);

        let collected = get_i128(&env, &DataKey::CollectedYieldUsdt0);
        if amount_usdt0 > collected {
            return Err(ContractError::InsufficientCollectedYield);
        }

        let new_collected = checked_sub(collected, amount_usdt0)?;
        env.storage()
            .instance()
            .set(&DataKey::CollectedYieldUsdt0, &new_collected);
        yield_client(&env).transfer(&env.current_contract_address(), &to, &amount_usdt0);

        env.events().publish_event(&PartnerYieldOutEvt {
            to,
            amount: amount_usdt0,
        });
        Ok(())
    }

    /// Returns the exchange address.
    pub fn exchange(env: Env) -> Address {
        get_address(&env, &DataKey::Exchange)
    }

    /// Returns the funding partner address.
    pub fn funding_partner(env: Env) -> Address {
        get_address(&env, &DataKey::FundingPartner)
    }

    /// Returns the principal token contract address.
    pub fn xlm_token(env: Env) -> Address {
        get_address(&env, &DataKey::XlmToken)
    }

    /// Returns the yield token contract address.
    pub fn yield_token(env: Env) -> Address {
        get_address(&env, &DataKey::YieldToken)
    }

    /// Returns total partner principal tracked by the vault.
    pub fn partner_principal_xlm(env: Env) -> i128 {
        get_i128(&env, &DataKey::PartnerPrincipalXlm)
    }

    /// Returns principal that is not currently reserved.
    pub fn free_principal_xlm(env: Env) -> i128 {
        get_i128(&env, &DataKey::FreePrincipalXlm)
    }

    /// Returns principal currently reserved for exchange collateral.
    pub fn reserved_for_exchange_xlm(env: Env) -> i128 {
        get_i128(&env, &DataKey::ReservedForExchangeXlm)
    }

    /// Returns yield actually paid into the vault.
    pub fn collected_yield_usdt0(env: Env) -> i128 {
        get_i128(&env, &DataKey::CollectedYieldUsdt0)
    }

    /// Returns yield owed by the exchange but not yet paid.
    pub fn yield_debt_usdt0(env: Env) -> i128 {
        get_i128(&env, &DataKey::YieldDebtUsdt0)
    }

    /// Returns the latest recorded yield settlement epoch.
    pub fn latest_settlement_epoch(env: Env) -> u64 {
        get_u64(&env, &DataKey::LatestSettlementEpoch)
    }

    /// Returns the last reserve-set exchange rate.
    pub fn last_set_reserve_exchange_rate(env: Env) -> i128 {
        get_i128(&env, &DataKey::LastSetReserveExchangeRate)
    }

    /// Returns the last reserve-set reference credit.
    pub fn last_set_reserve_credit(env: Env) -> i128 {
        get_i128(&env, &DataKey::LastSetReserveCredit)
    }

    /// Returns the most recent optional reserve audit reference.
    pub fn last_reserve_reference_hash(env: Env) -> Option<BytesN<32>> {
        env.storage()
            .instance()
            .get(&DataKey::LastReserveReferenceHash)
            .unwrap()
    }

    /// Returns the most recent optional yield-settlement audit reference.
    pub fn last_yield_reference_hash(env: Env) -> Option<BytesN<32>> {
        env.storage()
            .instance()
            .get(&DataKey::LastYieldSettlementReferenceHash)
            .unwrap()
    }

    /// Returns the current principal token balance held by the vault.
    pub fn xlm_balance(env: Env) -> i128 {
        xlm_client(&env).balance(&env.current_contract_address())
    }

    /// Returns the current yield token balance held by the vault.
    pub fn yield_balance(env: Env) -> i128 {
        yield_client(&env).balance(&env.current_contract_address())
    }
}

impl UpgradeableInternal for ManagedLiquidityVaultContract {
    fn _require_auth(e: &Env, operator: &Address) {
        operator.require_auth();
        let exchange = get_address(e, &DataKey::Exchange);
        let funding_partner = get_address(e, &DataKey::FundingPartner);
        if *operator != exchange && *operator != funding_partner {
            panic_with_error!(e, ContractError::Unauthorized);
        }
        exchange.require_auth();
        funding_partner.require_auth();
    }
}

/// Extends the TTL of the contract instance (and its instance storage) and the
/// contract Wasm code so the vault stays invocable between (potentially
/// infrequent) settlement calls.
///
/// `Instance::extend_ttl` bumps both the instance and code entries for the
/// current contract in a single call, so upgrades are not needed to keep the
/// Wasm code alive.
fn extend_contract_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Returns the address stored at `key`.
fn get_address(env: &Env, key: &DataKey) -> Address {
    env.storage().instance().get(key).unwrap()
}

/// Returns the `i128` value stored at `key`.
fn get_i128(env: &Env, key: &DataKey) -> i128 {
    env.storage().instance().get(key).unwrap()
}

/// Returns the `u64` value stored at `key`.
fn get_u64(env: &Env, key: &DataKey) -> u64 {
    env.storage().instance().get(key).unwrap()
}

/// Verifies that the posted reserve covers the stated credit.
///
/// # Arguments
///
/// * `_env` - Access to the Soroban environment.
/// * `target_reserved_xlm` - Reserve target in principal-token base units.
/// * `reference_credit_usdt0` - Reference credit amount in yield-token base
///   units.
/// * `exchange_rate` - Fixed-point price of `USDT0 per 1 XLM`, scaled by
///   [`RATE_SCALE`].
///
/// # Errors
///
/// * [`ContractError::ExchangeRateMustBePositive`] - If credit is positive and
///   `exchange_rate <= 0`.
/// * [`ContractError::ReserveBelowRequiredCollateral`] - If the reserve does
///   not cover the reference credit.
/// * [`ContractError::ArithmeticOverflow`] - If multiplication overflows.
fn ensure_reserve_covers_credit(
    _env: &Env,
    target_reserved_xlm: i128,
    reference_credit_usdt0: i128,
    exchange_rate: i128,
) -> Result<(), ContractError> {
    if reference_credit_usdt0 == 0 {
        return Ok(());
    }
    if exchange_rate <= 0 {
        return Err(ContractError::ExchangeRateMustBePositive);
    }

    let covered_credit = checked_mul(target_reserved_xlm, exchange_rate)?;
    let covered_credit = covered_credit / RATE_SCALE;
    if covered_credit < reference_credit_usdt0 {
        return Err(ContractError::ReserveBelowRequiredCollateral);
    }
    Ok(())
}

/// Multiplies two `i128` values and converts overflow into a contract error.
fn checked_mul(lhs: i128, rhs: i128) -> Result<i128, ContractError> {
    lhs.checked_mul(rhs)
        .ok_or(ContractError::ArithmeticOverflow)
}

/// Adds two `i128` values and converts overflow into a contract error.
fn checked_add(lhs: i128, rhs: i128) -> Result<i128, ContractError> {
    lhs.checked_add(rhs)
        .ok_or(ContractError::ArithmeticOverflow)
}

/// Subtracts two `i128` values and converts overflow into a contract error.
fn checked_sub(lhs: i128, rhs: i128) -> Result<i128, ContractError> {
    lhs.checked_sub(rhs)
        .ok_or(ContractError::ArithmeticOverflow)
}

/// Requires authorization from the configured exchange.
fn require_exchange_auth(env: &Env) {
    get_address(env, &DataKey::Exchange).require_auth();
}

/// Requires authorization from the configured funding partner.
fn require_partner_auth(env: &Env) {
    get_address(env, &DataKey::FundingPartner).require_auth();
}

/// Requires authorization from both the exchange and the funding partner.
fn require_exchange_and_partner_auth(env: &Env) {
    require_exchange_auth(env);
    require_partner_auth(env);
}

/// Returns the token client for the principal token.
fn xlm_client<'a>(env: &'a Env) -> soroban_sdk::token::TokenClient<'a> {
    let token = get_address(env, &DataKey::XlmToken);
    soroban_sdk::token::TokenClient::new(env, &token)
}

/// Returns the token client for the yield token.
fn yield_client<'a>(env: &'a Env) -> soroban_sdk::token::TokenClient<'a> {
    let token = get_address(env, &DataKey::YieldToken);
    soroban_sdk::token::TokenClient::new(env, &token)
}
