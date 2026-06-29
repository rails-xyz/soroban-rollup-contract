pub const WASM: &[u8] = soroban_sdk::contractfile!(
    file = "./target/wasm32v1-none/release/managed_liquidity_vault.wasm", sha256 =
    "9c89a1c372ae7745928fe933c13757829665b0a69c3eba47d6cd97c0d6e79de2"
);
#[soroban_sdk::contractargs(name = "Args")]
#[soroban_sdk::contractclient(name = "Client")]
pub trait Contract {
    fn upgrade(
        env: soroban_sdk::Env,
        new_wasm_hash: soroban_sdk::BytesN<32>,
        operator: soroban_sdk::Address,
    );
    fn exchange(env: soroban_sdk::Env) -> soroban_sdk::Address;
    fn pay_yield(
        env: soroban_sdk::Env,
        from: soroban_sdk::Address,
        amount_usdt0: i128,
    ) -> Result<(), ContractError>;
    fn xlm_token(env: soroban_sdk::Env) -> soroban_sdk::Address;
    fn set_reserve(
        env: soroban_sdk::Env,
        target_reserved_xlm: i128,
        reference_credit_usdt0: i128,
        exchange_rate: i128,
        reference_hash: Option<soroban_sdk::BytesN<32>>,
    ) -> Result<(), ContractError>;
    fn xlm_balance(env: soroban_sdk::Env) -> i128;
    fn yield_token(env: soroban_sdk::Env) -> soroban_sdk::Address;
    fn __constructor(
        env: soroban_sdk::Env,
        xlm_token: soroban_sdk::Address,
        yield_token: soroban_sdk::Address,
        exchange: soroban_sdk::Address,
        funding_partner: soroban_sdk::Address,
    );
    fn yield_balance(env: soroban_sdk::Env) -> i128;
    fn deposit_partner(
        env: soroban_sdk::Env,
        amount_xlm: i128,
    ) -> Result<(), ContractError>;
    fn funding_partner(env: soroban_sdk::Env) -> soroban_sdk::Address;
    fn yield_debt_usdt0(env: soroban_sdk::Env) -> i128;
    fn free_principal_xlm(env: soroban_sdk::Env) -> i128;
    fn collected_yield_usdt0(env: soroban_sdk::Env) -> i128;
    fn partner_principal_xlm(env: soroban_sdk::Env) -> i128;
    fn withdraw_partner_yield(
        env: soroban_sdk::Env,
        to: soroban_sdk::Address,
        amount_usdt0: i128,
    ) -> Result<(), ContractError>;
    fn last_set_reserve_credit(env: soroban_sdk::Env) -> i128;
    fn latest_settlement_epoch(env: soroban_sdk::Env) -> u64;
    fn record_yield_settlement(
        env: soroban_sdk::Env,
        epoch_id: u64,
        yield_due_usdt0: i128,
        yield_paid_usdt0: i128,
        reference_hash: Option<soroban_sdk::BytesN<32>>,
    ) -> Result<(), ContractError>;
    fn last_yield_reference_hash(
        env: soroban_sdk::Env,
    ) -> Option<soroban_sdk::BytesN<32>>;
    fn reserved_for_exchange_xlm(env: soroban_sdk::Env) -> i128;
    fn withdraw_partner_principal(
        env: soroban_sdk::Env,
        to: soroban_sdk::Address,
        amount_xlm: i128,
    ) -> Result<(), ContractError>;
    fn last_reserve_reference_hash(
        env: soroban_sdk::Env,
    ) -> Option<soroban_sdk::BytesN<32>>;
    fn last_set_reserve_exchange_rate(env: soroban_sdk::Env) -> i128;
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MerkleDistributorStorageKey {
    Root,
    Claimed(u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Rounding {
    Floor,
    Ceil,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PausableStorageKey {
    Paused,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum UpgradeableError {
    MigrationNotAllowed = 1100,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MerkleDistributorError {
    RootNotSet = 1300,
    IndexAlreadyClaimed = 1301,
    InvalidProof = 1302,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SorobanFixedPointError {
    ZeroDenominator = 1500,
    PhantomOverflow = 1501,
    ResultOverflow = 1502,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CryptoError {
    MerkleProofOutOfBounds = 1400,
    MerkleIndexOutOfBounds = 1401,
    HasherEmptyState = 1402,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PausableError {
    EnforcedPause = 1000,
    ExpectedPause = 1001,
}
#[soroban_sdk::contractevent(export = false, topics = ["yield_paid_evt"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct YieldPaidEvt {
    pub amount: i128,
}
#[soroban_sdk::contractevent(export = false, topics = ["reserve_set_evt"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReserveSetEvt {
    pub target_reserved_xlm: i128,
    pub reference_credit_usdt0: i128,
    pub exchange_rate: i128,
    pub reference_hash: Option<soroban_sdk::BytesN<32>>,
}
#[soroban_sdk::contractevent(export = false, topics = ["partner_deposit_evt"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartnerDepositEvt {
    pub amount: i128,
}
#[soroban_sdk::contractevent(export = false, topics = ["partner_yield_out_evt"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartnerYieldOutEvt {
    pub to: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(export = false, topics = ["yield_settlement_evt"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct YieldSettlementEvt {
    pub epoch_id: u64,
    pub yield_due_usdt0: i128,
    pub yield_paid_usdt0: i128,
    pub reference_hash: Option<soroban_sdk::BytesN<32>>,
}
#[soroban_sdk::contractevent(export = false, topics = ["partner_principal_out_evt"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartnerPrincipalOutEvt {
    pub to: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(export = false, topics = ["set_root"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SetRoot {
    pub root: soroban_sdk::Bytes,
}
#[soroban_sdk::contractevent(export = false, topics = ["set_claimed"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SetClaimed {
    pub index: soroban_sdk::Val,
}
#[soroban_sdk::contractevent(export = false, topics = ["paused"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Paused {}
#[soroban_sdk::contractevent(export = false, topics = ["unpaused"])]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Unpaused {}

