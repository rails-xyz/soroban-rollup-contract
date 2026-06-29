//! # Rollup Contract.
//!
//! This contract maintains a single collateral pool that backs:
//! - user deposits into the rollup,
//! - owner-posted withdrawal allowances for users,
//! - and owner-accrued protocol fees.
//!
//! The owner advances rollup state by posting new block hashes together with
//! newly withdrawable balances and fees. Users can then withdraw up to their
//! allowance directly from the collateral pool.

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, Address,
    BytesN, Env, Vec,
};
use stellar_contract_utils::upgradeable::UpgradeableInternal;

use stellar_access::ownable;
use stellar_macros::{only_owner, Upgradeable};

/// Approximate number of ledgers produced in one day (~5s close time).
const DAY_IN_LEDGERS: u32 = 17_280;
/// Target instance TTL set on every extension (~30 days).
const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
/// Re-extend the instance when its remaining TTL drops below this (~29 days).
const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

/// Target persistent-entry TTL set on every extension (~30 days).
const PERSISTENT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
/// Re-extend a persistent entry when its remaining TTL drops below this
/// (~29 days).
const PERSISTENT_LIFETIME_THRESHOLD: u32 = PERSISTENT_BUMP_AMOUNT - DAY_IN_LEDGERS;

/// Storage keys used by the rollup contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    LatestBlockHash,
    CollateralToken,
    WithdrawalAllowances(Address),
    Fees,
    TotalWithdrawable,
}

/// Errors returned by the rollup contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    // Deposit errors: 1-10
    DepositAmountMustBePositive = 1,

    // Rollup errors: 11-30
    NewBlockHashEmpty = 11,
    OldBlockHashMismatch = 12,
    BlockHashUnchanged = 13,
    ArrayLengthMismatch = 14,
    ArrayLengthExceedsLimit = 15,
    WithdrawalSumMismatch = 16,
    InsufficientBalance = 17,
    WithdrawalAmountMustBeNonNegative = 18,
    FeesMustBeNonNegative = 19,

    // Withdrawal errors: 31-40
    NoWithdrawalAllowance = 31,

    // Fee errors: 41-50
    NoFeesToCollect = 41,

    // Recovery errors: 51-60
    CannotRecoverCollateral = 51,
    RecoverAmountMustBePositive = 52,

    // Ownership errors: 61-70
    RenounceOwnershipDisabled = 61,
    Unauthorized = 62,

    // Arithmetic errors: 71-80
    ArithmeticOverflow = 71,
}

/// Rollup contract.
#[derive(Upgradeable)]
#[contract]
pub struct RollupContract;

/// Event emitted when collateral is deposited into the rollup.
#[contractevent]
#[derive(Clone)]
pub struct DepositEvent {
    pub user: Address,
    pub amount: i128,
}

/// Event emitted when a user withdraws their allowance.
#[contractevent]
#[derive(Clone)]
pub struct WithdrawalEvent {
    pub user: Address,
    pub amount: i128,
}

/// Event emitted when the owner posts a new rollup block.
#[contractevent]
#[derive(Clone)]
pub struct NewBlockEvent {
    pub new_block_hash: BytesN<32>,
    pub new_withdrawal_sum: i128,
    pub new_fees: i128,
}

/// Event emitted when accumulated fees are collected.
#[contractevent]
#[derive(Clone)]
pub struct FeesCollectedEvent {
    pub to: Address,
    pub amount: i128,
}

#[contractimpl]
impl RollupContract {
    /// Initializes the rollup contract.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `collateral_token` - Token contract used for deposits, withdrawals,
    ///   and fees.
    /// * `owner` - Upgrade and admin authority for rollup updates and fee
    ///   collection.
    ///
    /// # Notes
    ///
    /// * The latest block hash is initialized to the zero hash.
    /// * Fees and total withdrawable balances are initialized to zero.
    pub fn __constructor(env: Env, collateral_token: Address, owner: Address) {
        ownable::set_owner(&env, &owner);
        env.storage().instance().set(
            &DataKey::LatestBlockHash,
            &BytesN::from_array(&env, &[0; 32]),
        );
        env.storage()
            .instance()
            .set(&DataKey::CollateralToken, &collateral_token);
        env.storage().instance().set(&DataKey::Fees, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalWithdrawable, &0i128);

        extend_contract_ttl(&env);
    }

    /// Deposits collateral into the rollup.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `user` - Address providing the deposit.
    /// * `amount` - Amount of collateral token to deposit.
    ///
    /// # Errors
    ///
    /// * [`ContractError::DepositAmountMustBePositive`] - If `amount <= 0`.
    ///
    /// # Notes
    ///
    /// * Authorization from `user` is required.
    pub fn deposit(env: Env, user: Address, amount: i128) -> Result<(), ContractError> {
        user.require_auth();
        if amount <= 0 {
            return Err(ContractError::DepositAmountMustBePositive);
        }
        extend_contract_ttl(&env);
        let collateral_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();
        let token_client = soroban_sdk::token::TokenClient::new(&env, &collateral_token);
        token_client.transfer(&user, &env.current_contract_address(), &amount);
        env.events().publish_event(&DepositEvent { user, amount });
        Ok(())
    }

    /// Advances the rollup state to a new block hash and posts new withdrawable
    /// balances plus fees.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `old_block_hash` - Expected current block hash.
    /// * `new_block_hash` - New block hash to commit.
    /// * `new_withdrawal_addresses` - Addresses receiving newly posted
    ///   withdrawal allowances.
    /// * `new_withdrawal_amounts` - Amounts paired by index with
    ///   `new_withdrawal_addresses`.
    /// * `new_withdrawal_sum` - Sum of all new withdrawal amounts.
    /// * `new_fees` - Fees accrued in the new rollup block.
    ///
    /// # Errors
    ///
    /// * [`ContractError::NewBlockHashEmpty`] - If `new_block_hash` is zero.
    /// * [`ContractError::OldBlockHashMismatch`] - If `old_block_hash` does not
    ///   match the currently stored block hash.
    /// * [`ContractError::BlockHashUnchanged`] - If the new block hash is equal
    ///   to the old block hash.
    /// * [`ContractError::ArrayLengthMismatch`] - If the address and amount
    ///   arrays have different lengths.
    /// * [`ContractError::ArrayLengthExceedsLimit`] - If more than 100
    ///   withdrawal entries are posted.
    /// * [`ContractError::WithdrawalSumMismatch`] - If the provided
    ///   `new_withdrawal_sum` does not match the calculated total.
    /// * [`ContractError::InsufficientBalance`] - If the contract balance
    ///   cannot support the new withdrawable amount plus fees.
    /// * [`ContractError::FeesMustBeNonNegative`] - If `new_fees < 0`.
    /// * [`ContractError::WithdrawalAmountMustBeNonNegative`] - If any entry in
    ///   `new_withdrawal_amounts` is negative.
    /// * [`ContractError::ArithmeticOverflow`] - If any balance, fee, or
    ///   allowance accumulation overflows.
    ///
    /// # Notes
    ///
    /// * Owner authorization is required.
    /// * Existing user allowances are incremented, not replaced.
    #[only_owner]
    pub fn rollup(
        env: Env,
        old_block_hash: BytesN<32>,
        new_block_hash: BytesN<32>,
        new_withdrawal_addresses: Vec<Address>,
        new_withdrawal_amounts: Vec<i128>,
        new_withdrawal_sum: i128,
        new_fees: i128,
    ) -> Result<(), ContractError> {
        extend_contract_ttl(&env);
        if new_fees < 0 {
            return Err(ContractError::FeesMustBeNonNegative);
        }
        if new_block_hash == BytesN::from_array(&env, &[0; 32]) {
            return Err(ContractError::NewBlockHashEmpty);
        }
        let latest_block_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::LatestBlockHash)
            .unwrap();
        if old_block_hash != latest_block_hash {
            return Err(ContractError::OldBlockHashMismatch);
        }
        if new_block_hash == old_block_hash {
            return Err(ContractError::BlockHashUnchanged);
        }
        if new_withdrawal_addresses.len() != new_withdrawal_amounts.len() {
            return Err(ContractError::ArrayLengthMismatch);
        }
        if new_withdrawal_addresses.len() > 100 {
            return Err(ContractError::ArrayLengthExceedsLimit);
        }

        let mut calculated_withdrawal_sum = 0i128;
        for i in 0..new_withdrawal_addresses.len() {
            let user = &new_withdrawal_addresses.get(i).unwrap();
            let allowance = new_withdrawal_amounts.get(i).unwrap();
            if allowance < 0 {
                return Err(ContractError::WithdrawalAmountMustBeNonNegative);
            }
            let key = DataKey::WithdrawalAllowances(user.clone());
            let current = env.storage().persistent().get(&key).unwrap_or(0i128);
            env.storage()
                .persistent()
                .set(&key, &checked_add(current, allowance)?);
            env.storage().persistent().extend_ttl(
                &key,
                PERSISTENT_LIFETIME_THRESHOLD,
                PERSISTENT_BUMP_AMOUNT,
            );
            calculated_withdrawal_sum = checked_add(calculated_withdrawal_sum, allowance)?;
        }
        if calculated_withdrawal_sum != new_withdrawal_sum {
            return Err(ContractError::WithdrawalSumMismatch);
        }

        let net_new_withdrawable = checked_add(new_withdrawal_sum, new_fees)?;
        let collateral_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();
        let token_client = soroban_sdk::token::TokenClient::new(&env, &collateral_token);
        let balance = token_client.balance(&env.current_contract_address());
        let total_withdrawable: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWithdrawable)
            .unwrap();
        if net_new_withdrawable > checked_sub(balance, total_withdrawable)? {
            return Err(ContractError::InsufficientBalance);
        }

        env.storage().instance().set(
            &DataKey::TotalWithdrawable,
            &checked_add(total_withdrawable, net_new_withdrawable)?,
        );
        let current_fees: i128 = env.storage().instance().get(&DataKey::Fees).unwrap();
        env.storage()
            .instance()
            .set(&DataKey::Fees, &checked_add(current_fees, new_fees)?);
        env.storage()
            .instance()
            .set(&DataKey::LatestBlockHash, &new_block_hash);
        env.events().publish_event(&NewBlockEvent {
            new_block_hash,
            new_withdrawal_sum,
            new_fees,
        });
        Ok(())
    }

    /// Withdraws the full posted allowance for `user`.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `user` - Address withdrawing its allowance.
    ///
    /// # Errors
    ///
    /// * [`ContractError::NoWithdrawalAllowance`] - If no positive allowance is
    ///   available for `user`.
    ///
    /// # Notes
    ///
    /// * Authorization from `user` is required.
    /// * This method withdraws the full stored allowance and resets it to zero.
    pub fn withdraw(env: Env, user: Address) -> Result<(), ContractError> {
        user.require_auth();
        extend_contract_ttl(&env);

        let key = DataKey::WithdrawalAllowances(user.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if amount <= 0 {
            return Err(ContractError::NoWithdrawalAllowance);
        }

        // The full allowance is withdrawn, so the entry is now zero. Remove it
        // rather than keeping a zero-valued entry alive on rent; a future read
        // falls back to `unwrap_or(0)`.
        env.storage().persistent().remove(&key);
        let current_total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWithdrawable)
            .unwrap();
        env.storage().instance().set(
            &DataKey::TotalWithdrawable,
            &checked_sub(current_total, amount)?,
        );
        let collateral_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();
        let token_client = soroban_sdk::token::TokenClient::new(&env, &collateral_token);
        token_client.transfer(&env.current_contract_address(), &user, &amount);
        env.events()
            .publish_event(&WithdrawalEvent { user, amount });
        Ok(())
    }

    /// Collects all currently accrued protocol fees.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `to` - Recipient of collected fees.
    ///
    /// # Errors
    ///
    /// * [`ContractError::NoFeesToCollect`] - If no positive fee balance is
    ///   available.
    ///
    /// # Notes
    ///
    /// * Owner authorization is required.
    /// * Collected fees are removed from both `Fees` and `TotalWithdrawable`.
    #[only_owner]
    pub fn collect_fees(env: Env, to: Address) -> Result<(), ContractError> {
        extend_contract_ttl(&env);
        let fees: i128 = env.storage().instance().get(&DataKey::Fees).unwrap();
        if fees <= 0 {
            return Err(ContractError::NoFeesToCollect);
        }

        env.storage().instance().set(&DataKey::Fees, &0i128);
        let current_total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWithdrawable)
            .unwrap();
        env.storage().instance().set(
            &DataKey::TotalWithdrawable,
            &checked_sub(current_total, fees)?,
        );
        let collateral_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();
        let token_client = soroban_sdk::token::TokenClient::new(&env, &collateral_token);
        token_client.transfer(&env.current_contract_address(), &to, &fees);
        env.events()
            .publish_event(&FeesCollectedEvent { to, amount: fees });
        Ok(())
    }

    /// Recovers non-collateral tokens that were sent to the contract.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `token_address` - Token contract to recover.
    /// * `to` - Recipient of the recovered tokens.
    /// * `amount` - Amount to recover.
    ///
    /// # Errors
    ///
    /// * [`ContractError::CannotRecoverCollateral`] - If `token_address` is the
    ///   configured collateral token.
    /// * [`ContractError::RecoverAmountMustBePositive`] - If `amount <= 0`.
    ///
    /// # Notes
    ///
    /// * Owner authorization is required.
    #[only_owner]
    pub fn recover(
        env: Env,
        token_address: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        let collateral_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();
        if token_address == collateral_token {
            return Err(ContractError::CannotRecoverCollateral);
        }
        if amount <= 0 {
            return Err(ContractError::RecoverAmountMustBePositive);
        }
        extend_contract_ttl(&env);
        let token_client = soroban_sdk::token::TokenClient::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &to, &amount);
        Ok(())
    }

    /// Rejects ownership renunciation.
    ///
    /// # Errors
    ///
    /// * [`ContractError::RenounceOwnershipDisabled`] - Always returned.
    pub fn renounce_ownership(_env: Env) -> Result<(), ContractError> {
        Err(ContractError::RenounceOwnershipDisabled)
    }

    /// Initiates a 2-step ownership transfer.
    ///
    /// The proposed new owner must later call [`Self::accept_ownership`] to
    /// complete the transfer.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    /// * `new_owner` - Proposed new owner.
    /// * `live_until_ledger` - Ledger until which the pending transfer can be
    ///   accepted.
    ///
    /// # Notes
    ///
    /// * Authorization is enforced internally by the ownable library.
    pub fn transfer_ownership(env: Env, new_owner: Address, live_until_ledger: u32) {
        ownable::transfer_ownership(&env, &new_owner, live_until_ledger);
    }

    /// Accepts a pending ownership transfer.
    ///
    /// # Arguments
    ///
    /// * `env` - Access to the Soroban environment.
    pub fn accept_ownership(env: Env) {
        ownable::accept_ownership(&env);
    }

    /// Returns the current owner.
    pub fn owner(env: Env) -> Option<Address> {
        ownable::get_owner(&env)
    }

    /// Returns the latest committed rollup block hash.
    pub fn latest_block_hash(env: Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::LatestBlockHash)
            .unwrap()
    }

    /// Returns the current withdrawal allowance for `user`.
    pub fn withdrawal_allowances(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::WithdrawalAllowances(user))
            .unwrap_or(0)
    }

    /// Returns currently accrued protocol fees.
    pub fn fees(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Fees).unwrap()
    }

    /// Returns the total amount currently reserved for user withdrawals plus
    /// accrued fees.
    pub fn total_withdrawable(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalWithdrawable)
            .unwrap()
    }

    /// Returns the current collateral-token balance held by the contract.
    pub fn collateral_balance(env: Env) -> i128 {
        let collateral_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();
        let token_client = soroban_sdk::token::TokenClient::new(&env, &collateral_token);
        token_client.balance(&env.current_contract_address())
    }
}

impl UpgradeableInternal for RollupContract {
    /// Requires authorization from the upgrade owner.
    fn _require_auth(e: &Env, operator: &Address) {
        operator.require_auth();
        let owner = ownable::get_owner(e).unwrap();
        if *operator != owner {
            panic_with_error!(e, ContractError::Unauthorized);
        }
    }
}

/// Extends the TTL of the contract instance (and its instance storage) and the
/// contract Wasm code so the rollup stays invocable between (potentially
/// infrequent) owner updates.
///
/// `Instance::extend_ttl` bumps both the instance and code entries for the
/// current contract in a single call, so upgrades are not needed to keep the
/// Wasm code alive.
fn extend_contract_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
