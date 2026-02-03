use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env, Vec,
};

use stellar_access::ownable;
use stellar_macros::only_owner;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    LatestBlockHash,
    CollateralToken,
    WithdrawalAllowances(Address),
    Fees,
    TotalWithdrawable,
}

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

    // Withdrawal errors: 31-40
    NoWithdrawalAllowance = 31,

    // Fee errors: 41-50
    NoFeesToCollect = 41,

    // Recovery errors: 51-60
    CannotRecoverCollateral = 51,
    RecoverAmountMustBePositive = 52,

    // Ownership errors: 61-70
    RenounceOwnershipDisabled = 61,
}

#[contract]
pub struct RollupContract;

#[contractevent]
#[derive(Clone)]
pub struct DepositEvent {
    pub user: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct WithdrawalEvent {
    pub user: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct NewBlockEvent {
    pub new_block_hash: BytesN<32>,
    pub new_withdrawal_sum: i128,
    pub new_fees: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct FeesCollectedEvent {
    pub to: Address,
    pub amount: i128,
}

#[contractimpl]
impl RollupContract {
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
    }

    pub fn deposit(env: Env, user: Address, amount: i128) -> Result<(), ContractError> {
        user.require_auth();
        if amount <= 0 {
            return Err(ContractError::DepositAmountMustBePositive);
        }
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
            let key = DataKey::WithdrawalAllowances(user.clone());
            let current = env.storage().persistent().get(&key).unwrap_or(0i128);
            env.storage().persistent().set(&key, &(current + allowance));
            calculated_withdrawal_sum += allowance;
        }
        if calculated_withdrawal_sum != new_withdrawal_sum {
            return Err(ContractError::WithdrawalSumMismatch);
        }

        let net_new_withdrawable = new_withdrawal_sum + new_fees;
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
        if net_new_withdrawable > balance - total_withdrawable {
            return Err(ContractError::InsufficientBalance);
        }

        let current_total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWithdrawable)
            .unwrap();
        env.storage().instance().set(
            &DataKey::TotalWithdrawable,
            &(current_total + net_new_withdrawable),
        );
        let current_fees: i128 = env.storage().instance().get(&DataKey::Fees).unwrap();
        env.storage()
            .instance()
            .set(&DataKey::Fees, &(current_fees + new_fees));
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

    pub fn withdraw(env: Env, user: Address) -> Result<(), ContractError> {
        user.require_auth();

        let key = DataKey::WithdrawalAllowances(user.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if amount <= 0 {
            return Err(ContractError::NoWithdrawalAllowance);
        }

        env.storage().persistent().set(&key, &0i128);
        let current_total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalWithdrawable)
            .unwrap();
        env.storage()
            .instance()
            .set(&DataKey::TotalWithdrawable, &(current_total - amount));
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

    #[only_owner]
    pub fn collect_fees(env: Env, to: Address) -> Result<(), ContractError> {
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
        env.storage()
            .instance()
            .set(&DataKey::TotalWithdrawable, &(current_total - fees));
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
        let token_client = soroban_sdk::token::TokenClient::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &to, &amount);
        Ok(())
    }

    pub fn renounce_ownership(_env: Env) -> Result<(), ContractError> {
        Err(ContractError::RenounceOwnershipDisabled)
    }

    /// Initiates a 2-step ownership transfer. The new owner must call `accept_ownership` to complete.
    /// Note: Auth is enforced internally by the ownable library.
    pub fn transfer_ownership(env: Env, new_owner: Address, live_until_ledger: u32) {
        ownable::transfer_ownership(&env, &new_owner, live_until_ledger);
    }

    pub fn accept_ownership(env: Env) {
        ownable::accept_ownership(&env);
    }

    // View functions
    pub fn owner(env: Env) -> Option<Address> {
        ownable::get_owner(&env)
    }

    pub fn latest_block_hash(env: Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::LatestBlockHash)
            .unwrap()
    }

    pub fn withdrawal_allowances(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::WithdrawalAllowances(user))
            .unwrap_or(0)
    }

    pub fn fees(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Fees).unwrap()
    }

    pub fn total_withdrawable(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalWithdrawable)
            .unwrap()
    }

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
