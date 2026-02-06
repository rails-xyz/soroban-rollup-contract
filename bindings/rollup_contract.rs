pub const WASM: &[u8] = soroban_sdk::contractfile!(
    file = "./target/wasm32v1-none/release/rollup_contract.wasm",
    sha256 = "8270f91c58b6331ae65ea6065100ba1f972d2f3a5dfe067ce7815639f97b77a8"
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
    fn withdraw(env: soroban_sdk::Env, user: soroban_sdk::Address) -> Result<(), ContractError>;
    fn collect_fees(env: soroban_sdk::Env, to: soroban_sdk::Address) -> Result<(), ContractError>;
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
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleAccountKey {
    pub index: u32,
    pub role: soroban_sdk::Symbol,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DataKey {
    LatestBlockHash,
    CollateralToken,
    WithdrawalAllowances(soroban_sdk::Address),
    Fees,
    TotalWithdrawable,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum AccessControlStorageKey {
    RoleAccounts(RoleAccountKey),
    HasRole(soroban_sdk::Address, soroban_sdk::Symbol),
    RoleAccountsCount(soroban_sdk::Symbol),
    RoleAdmin(soroban_sdk::Symbol),
    Admin,
    PendingAdmin,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OwnableStorageKey {
    Owner,
    PendingOwner,
}
#[soroban_sdk::contracterror(export = false)]
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
    NoWithdrawalAllowance = 31,
    NoFeesToCollect = 41,
    CannotRecoverCollateral = 51,
    RecoverAmountMustBePositive = 52,
    RenounceOwnershipDisabled = 61,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RoleTransferError {
    NoPendingTransfer = 2200,
    InvalidLiveUntilLedger = 2201,
    InvalidPendingAccount = 2202,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum AccessControlError {
    Unauthorized = 2000,
    AdminNotSet = 2001,
    IndexOutOfBounds = 2002,
    AdminRoleNotFound = 2003,
    RoleCountIsNotZero = 2004,
    RoleNotFound = 2005,
    AdminAlreadySet = 2006,
    RoleNotHeld = 2007,
    RoleIsEmpty = 2008,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OwnableError {
    OwnerNotSet = 2100,
    TransferInProgress = 2101,
    OwnerAlreadySet = 2102,
}
#[soroban_sdk::contractevent(topics = ["deposit_event"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DepositEvent {
    pub user: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["new_block_event"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct NewBlockEvent {
    pub new_block_hash: soroban_sdk::BytesN<32>,
    pub new_withdrawal_sum: i128,
    pub new_fees: i128,
}
#[soroban_sdk::contractevent(topics = ["withdrawal_event"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct WithdrawalEvent {
    pub user: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["fees_collected_event"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FeesCollectedEvent {
    pub to: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["role_granted"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleGranted {
    #[topic]
    pub role: soroban_sdk::Symbol,
    #[topic]
    pub account: soroban_sdk::Address,
    pub caller: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["role_revoked"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleRevoked {
    #[topic]
    pub role: soroban_sdk::Symbol,
    #[topic]
    pub account: soroban_sdk::Address,
    pub caller: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["admin_renounced"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AdminRenounced {
    #[topic]
    pub admin: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["role_admin_changed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleAdminChanged {
    #[topic]
    pub role: soroban_sdk::Symbol,
    pub previous_admin_role: soroban_sdk::Symbol,
    pub new_admin_role: soroban_sdk::Symbol,
}
#[soroban_sdk::contractevent(topics = ["admin_transfer_completed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AdminTransferCompleted {
    #[topic]
    pub new_admin: soroban_sdk::Address,
    pub previous_admin: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["admin_transfer_initiated"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AdminTransferInitiated {
    #[topic]
    pub current_admin: soroban_sdk::Address,
    pub new_admin: soroban_sdk::Address,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contractevent(topics = ["ownership_transfer"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnershipTransfer {
    pub old_owner: soroban_sdk::Address,
    pub new_owner: soroban_sdk::Address,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contractevent(topics = ["ownership_renounced"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnershipRenounced {
    pub old_owner: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["ownership_transfer_completed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnershipTransferCompleted {
    pub new_owner: soroban_sdk::Address,
}
