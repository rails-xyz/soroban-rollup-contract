pub const WASM: &[u8] = soroban_sdk::contractfile!(
    file = "./target/wasm32v1-none/release/managed_liquidity_vault.wasm", sha256 =
    "b8adb410091b7440cae657712a6930765bec97df57aab5de25ea1ed35b3b4b09"
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
    fn unaccounted_balance(
        env: soroban_sdk::Env,
        token: soroban_sdk::Address,
    ) -> Result<i128, ContractError>;
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
    fn recover_unaccounted_tokens(
        env: soroban_sdk::Env,
        token: soroban_sdk::Address,
        to: soroban_sdk::Address,
        amount: i128,
    ) -> Result<(), ContractError>;
    fn withdraw_partner_principal(
        env: soroban_sdk::Env,
        to: soroban_sdk::Address,
        amount_xlm: i128,
    ) -> Result<(), ContractError>;
    fn last_reserve_reference_hash(
        env: soroban_sdk::Env,
    ) -> Option<soroban_sdk::BytesN<32>>;
    fn total_excess_yield_paid_usdt0(env: soroban_sdk::Env) -> i128;
    fn last_set_reserve_exchange_rate(env: soroban_sdk::Env) -> i128;
}
#[soroban_sdk::contracterror]
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
    RecoverAmountMustBePositive = 22,
    InsufficientUnaccountedBalance = 23,
    RoleAddressMustNotBeToken = 24,
    TokenDecimalsMustMatch = 25,
}

