pub const WASM: &[u8] = soroban_sdk::contractfile!(
    file = "./target/wasm32v1-none/release/rollup_contract.wasm", sha256 =
    "e00676b7bfe9afb3d37f0a0f25537e31683f3332ebc27fe1638acabdacc8fe7e"
);
#[soroban_sdk::contractargs(name = "Args")]
#[soroban_sdk::contractclient(name = "Client")]
pub trait Contract {
    fn fees(env: soroban_sdk::Env) -> i128;
    fn owner(env: soroban_sdk::Env) -> Option<soroban_sdk::Address>;
    fn rollup(
        env: soroban_sdk::Env,
        old_block_hash: soroban_sdk::BytesN<32>,
        new_block_hash: soroban_sdk::BytesN<32>,
        new_withdrawal_addresses: soroban_sdk::Vec<soroban_sdk::Address>,
        new_withdrawal_amounts: soroban_sdk::Vec<i128>,
        new_withdrawal_sum: i128,
        new_fees: i128,
    ) -> Result<(), ContractError>;
    fn deposit(
        env: soroban_sdk::Env,
        user: soroban_sdk::Address,
        amount: i128,
    ) -> Result<(), ContractError>;
    fn recover(
        env: soroban_sdk::Env,
        token_address: soroban_sdk::Address,
        to: soroban_sdk::Address,
        amount: i128,
    ) -> Result<(), ContractError>;
    fn upgrade(
        env: soroban_sdk::Env,
        new_wasm_hash: soroban_sdk::BytesN<32>,
        operator: soroban_sdk::Address,
    );
    fn withdraw(
        env: soroban_sdk::Env,
        user: soroban_sdk::Address,
    ) -> Result<(), ContractError>;
    fn collect_fees(
        env: soroban_sdk::Env,
        to: soroban_sdk::Address,
    ) -> Result<(), ContractError>;
    fn __constructor(
        env: soroban_sdk::Env,
        collateral_token: soroban_sdk::Address,
        owner: soroban_sdk::Address,
    );
    fn accept_ownership(env: soroban_sdk::Env);
    fn latest_block_hash(env: soroban_sdk::Env) -> soroban_sdk::BytesN<32>;
    fn collateral_balance(env: soroban_sdk::Env) -> i128;
    fn renounce_ownership(env: soroban_sdk::Env) -> Result<(), ContractError>;
    fn total_withdrawable(env: soroban_sdk::Env) -> i128;
    fn transfer_ownership(
        env: soroban_sdk::Env,
        new_owner: soroban_sdk::Address,
        live_until_ledger: u32,
    );
    fn withdrawal_allowances(env: soroban_sdk::Env, user: soroban_sdk::Address) -> i128;
}
#[soroban_sdk::contracterror]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ContractError {
    DepositAmountMustBePositive = 1,
    NewBlockHashEmpty = 11,
    OldBlockHashMismatch = 12,
    BlockHashUnchanged = 13,
    ArrayLengthMismatch = 14,
    ArrayLengthExceedsLimit = 15,
    WithdrawalSumMismatch = 16,
    InsufficientBalance = 17,
    WithdrawalAmountMustBeNonNegative = 18,
    FeesMustBeNonNegative = 19,
    NoWithdrawalAllowance = 31,
    NoFeesToCollect = 41,
    CannotRecoverCollateral = 51,
    RecoverAmountMustBePositive = 52,
    RenounceOwnershipDisabled = 61,
    Unauthorized = 62,
    ArithmeticOverflow = 71,
}
#[soroban_sdk::contracterror]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RoleTransferError {
    NoPendingTransfer = 2200,
    InvalidLiveUntilLedger = 2201,
    InvalidPendingAccount = 2202,
    TransferExpired = 2203,
}
#[soroban_sdk::contracterror]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OwnableError {
    OwnerNotSet = 2100,
    TransferInProgress = 2101,
    OwnerAlreadySet = 2102,
}
#[soroban_sdk::contractevent(topics = ["ownership_transfer"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnershipTransfer {
    pub old_owner: soroban_sdk::Address,
    pub new_owner: soroban_sdk::Address,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contractevent(topics = ["ownership_transfer_completed"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnershipTransferCompleted {
    pub new_owner: soroban_sdk::Address,
}

