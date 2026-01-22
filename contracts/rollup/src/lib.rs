#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, contracttype, Address, BytesN, Env, Vec};

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
    ReentrancyGuard,
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
        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &false);
    }

    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic!("Deposit amount must be greater than 0");
        }
        let collateral_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();
        let token_client = soroban_sdk::token::TokenClient::new(&env, &collateral_token);
        token_client.transfer(&user, &env.current_contract_address(), &amount);
        env.events().publish_event(&DepositEvent { user, amount });
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
    ) {
        if new_block_hash == BytesN::from_array(&env, &[0; 32]) {
            panic!("New block hash cannot be empty");
        }
        let latest_block_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::LatestBlockHash)
            .unwrap();
        if old_block_hash != latest_block_hash {
            panic!("Old block hash does not match the latest block hash");
        }
        if new_block_hash == old_block_hash {
            panic!("New block hash cannot be the same as the old block hash");
        }
        if new_withdrawal_addresses.len() != new_withdrawal_amounts.len() {
            panic!("Arrays must have the same length");
        }
        if new_withdrawal_addresses.len() > 100 {
            panic!("Array length exceeds gas limit safety bounds");
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
            panic!("Calculated withdrawal sum does not match the provided sum");
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
            panic!("Insufficient balance");
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
    }

    pub fn withdraw(env: Env, user: Address) {
        // Require authorization from the user to withdraw their funds
        user.require_auth();

        // Reentrancy guard
        let guard: bool = env
            .storage()
            .instance()
            .get(&DataKey::ReentrancyGuard)
            .unwrap();
        if guard {
            panic!("Reentrant call");
        }
        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &true);

        let key = DataKey::WithdrawalAllowances(user.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if amount <= 0 {
            panic!("No withdrawal allowance available");
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

        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &false);
    }

    #[only_owner]
    pub fn collect_fees(env: Env, to: Address) {
        // Reentrancy guard
        let guard: bool = env
            .storage()
            .instance()
            .get(&DataKey::ReentrancyGuard)
            .unwrap();
        if guard {
            panic!("Reentrant call");
        }
        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &true);

        let fees: i128 = env.storage().instance().get(&DataKey::Fees).unwrap();
        if fees <= 0 {
            panic!("No fees to collect");
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

        env.storage()
            .instance()
            .set(&DataKey::ReentrancyGuard, &false);
    }

    #[only_owner]
    pub fn recover(env: Env, token_address: Address, to: Address, amount: i128) {
        let collateral_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::CollateralToken)
            .unwrap();
        if token_address == collateral_token {
            panic!("Cannot recover collateral token");
        }
        if amount <= 0 {
            panic!("Amount must be greater than 0");
        }
        let token_client = soroban_sdk::token::TokenClient::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &to, &amount);
    }

    pub fn renounce_ownership(_env: Env) {
        panic!("Renouncing ownership is disabled");
    }

    // View functions - owner() is provided by OpenZeppelin Ownable trait
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
}

mod test;
