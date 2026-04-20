use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, panic_with_error, Address,
    BytesN, Env,
};
use stellar_access::ownable;
use stellar_contract_utils::upgradeable::UpgradeableInternal;
use stellar_macros::Upgradeable;

const RATE_SCALE: i128 = 10_000_000;

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
    RenounceOwnershipDisabled = 14,
    Unauthorized = 15,
    ExchangeRateMustBePositive = 16,
    ReserveBelowRequiredCollateral = 17,
    ArithmeticOverflow = 19,
}

#[derive(Upgradeable)]
#[contract]
pub struct ManagedLiquidityVaultContract;

#[contractevent]
#[derive(Clone)]
pub struct PartnerDepositEvt {
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct PartnerPrincipalOutEvt {
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct ReserveSetEvt {
    pub target_reserved_xlm: i128,
    pub reference_credit_usdt0: i128,
    pub exchange_rate: i128,
    pub reference_hash: Option<BytesN<32>>,
}

#[contractevent]
#[derive(Clone)]
pub struct YieldSettlementEvt {
    pub epoch_id: u64,
    pub yield_due_usdt0: i128,
    pub yield_paid_usdt0: i128,
    pub reference_hash: Option<BytesN<32>>,
}

#[contractevent]
#[derive(Clone)]
pub struct YieldPaidEvt {
    pub amount: i128,
}

#[contractevent]
#[derive(Clone)]
pub struct PartnerYieldOutEvt {
    pub to: Address,
    pub amount: i128,
}

#[contractimpl]
impl ManagedLiquidityVaultContract {
    pub fn __constructor(
        env: Env,
        xlm_token: Address,
        yield_token: Address,
        exchange: Address,
        funding_partner: Address,
        owner: Address,
    ) {
        // `owner` is an upgrade-only governance address. Business operations are
        // deliberately authorized by `exchange` and/or `funding_partner` below.
        ownable::set_owner(&env, &owner);
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
        instance.set(&DataKey::LastReserveReferenceHash, &Option::<BytesN<32>>::None);
        instance.set(
            &DataKey::LastYieldSettlementReferenceHash,
            &Option::<BytesN<32>>::None,
        );
    }

    pub fn deposit_partner(env: Env, amount_xlm: i128) -> Result<(), ContractError> {
        if amount_xlm <= 0 {
            return Err(ContractError::DepositAmountMustBePositive);
        }
        let funding_partner = get_address(&env, &DataKey::FundingPartner);
        funding_partner.require_auth();

        // The vault holds principal custody directly; no share token is minted in
        // this managed design because there is only one funding partner.
        xlm_client(&env).transfer(
            &funding_partner,
            &env.current_contract_address(),
            &amount_xlm,
        );

        let partner_principal = get_i128(&env, &DataKey::PartnerPrincipalXlm);
        let free_principal = get_i128(&env, &DataKey::FreePrincipalXlm);
        let instance = env.storage().instance();
        instance.set(
            &DataKey::PartnerPrincipalXlm,
            &(partner_principal + amount_xlm),
        );
        instance.set(&DataKey::FreePrincipalXlm, &(free_principal + amount_xlm));

        env.events()
            .publish_event(&PartnerDepositEvt { amount: amount_xlm });
        Ok(())
    }

    pub fn withdraw_partner_principal(
        env: Env,
        to: Address,
        amount_xlm: i128,
    ) -> Result<(), ContractError> {
        if amount_xlm <= 0 {
            return Err(ContractError::WithdrawAmountMustBePositive);
        }
        require_exchange_and_partner_auth(&env);

        let free_principal = get_i128(&env, &DataKey::FreePrincipalXlm);
        if amount_xlm > free_principal {
            return Err(ContractError::InsufficientFreePrincipal);
        }

        // Only free principal may leave the vault. Reserved principal remains
        // encumbered behind exchange credit and cannot be withdrawn.
        let partner_principal = get_i128(&env, &DataKey::PartnerPrincipalXlm);
        let instance = env.storage().instance();
        instance.set(
            &DataKey::PartnerPrincipalXlm,
            &(partner_principal - amount_xlm),
        );
        instance.set(&DataKey::FreePrincipalXlm, &(free_principal - amount_xlm));

        xlm_client(&env).transfer(&env.current_contract_address(), &to, &amount_xlm);
        env.events().publish_event(&PartnerPrincipalOutEvt {
            to,
            amount: amount_xlm,
        });
        Ok(())
    }

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

        // Trust boundary:
        // - the exchange computes reserve targets off-chain;
        // - the contract only enforces that the posted reserve does not exceed
        //   total principal and covers the stated credit at 100% mark value.
        // Haircut policy intentionally remains off-chain.
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

        let instance = env.storage().instance();
        instance.set(&DataKey::ReservedForExchangeXlm, &target_reserved_xlm);
        instance.set(
            &DataKey::FreePrincipalXlm,
            &(partner_principal - target_reserved_xlm),
        );
        // `LastSetReserve*` fields are audit metadata only. They are not used for
        // authorization and do not affect balances after validation succeeds.
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

        let latest_epoch = get_u64(&env, &DataKey::LatestSettlementEpoch);
        if epoch_id <= latest_epoch {
            return Err(ContractError::SettlementEpochMustIncrease);
        }

        let current_debt = get_i128(&env, &DataKey::YieldDebtUsdt0);
        let total_obligation = current_debt + yield_due_usdt0;
        if yield_paid_usdt0 > total_obligation {
            return Err(ContractError::YieldPaidExceedsDebtAndCurrentDue);
        }

        // `yield_paid_usdt0` must be backed by an actual token transfer. Unpaid
        // yield remains as debt and is not treated as collectible balance.
        if yield_paid_usdt0 > 0 {
            yield_client(&env).transfer(
                &get_address(&env, &DataKey::Exchange),
                &env.current_contract_address(),
                &yield_paid_usdt0,
            );
        }

        let collected = get_i128(&env, &DataKey::CollectedYieldUsdt0);
        let instance = env.storage().instance();
        instance.set(
            &DataKey::CollectedYieldUsdt0,
            &(collected + yield_paid_usdt0),
        );
        instance.set(
            &DataKey::YieldDebtUsdt0,
            &(total_obligation - yield_paid_usdt0),
        );
        instance.set(&DataKey::LatestSettlementEpoch, &epoch_id);
        instance.set(
            &DataKey::LastYieldSettlementReferenceHash,
            &reference_hash,
        );

        env.events().publish_event(&YieldSettlementEvt {
            epoch_id,
            yield_due_usdt0,
            yield_paid_usdt0,
            reference_hash,
        });
        Ok(())
    }

    pub fn pay_yield(env: Env, amount_usdt0: i128) -> Result<(), ContractError> {
        if amount_usdt0 <= 0 {
            return Err(ContractError::YieldAmountMustBePositive);
        }
        require_exchange_auth(&env);

        // This method is intentionally one-way favorable to the funding partner:
        // it can only reduce debt and/or increase collected yield.
        let exchange = get_address(&env, &DataKey::Exchange);
        yield_client(&env).transfer(&exchange, &env.current_contract_address(), &amount_usdt0);

        let debt = get_i128(&env, &DataKey::YieldDebtUsdt0);
        let collected = get_i128(&env, &DataKey::CollectedYieldUsdt0);
        let new_debt = if amount_usdt0 >= debt {
            0
        } else {
            debt - amount_usdt0
        };

        let instance = env.storage().instance();
        instance.set(&DataKey::YieldDebtUsdt0, &new_debt);
        instance.set(&DataKey::CollectedYieldUsdt0, &(collected + amount_usdt0));

        env.events().publish_event(&YieldPaidEvt {
            amount: amount_usdt0,
        });
        Ok(())
    }

    pub fn withdraw_partner_yield(
        env: Env,
        to: Address,
        amount_usdt0: i128,
    ) -> Result<(), ContractError> {
        if amount_usdt0 <= 0 {
            return Err(ContractError::YieldAmountMustBePositive);
        }
        require_partner_auth(&env);

        let collected = get_i128(&env, &DataKey::CollectedYieldUsdt0);
        if amount_usdt0 > collected {
            return Err(ContractError::InsufficientCollectedYield);
        }

        env.storage()
            .instance()
            .set(&DataKey::CollectedYieldUsdt0, &(collected - amount_usdt0));
        yield_client(&env).transfer(&env.current_contract_address(), &to, &amount_usdt0);

        env.events().publish_event(&PartnerYieldOutEvt {
            to,
            amount: amount_usdt0,
        });
        Ok(())
    }

    pub fn renounce_ownership(_env: Env) -> Result<(), ContractError> {
        Err(ContractError::RenounceOwnershipDisabled)
    }

    pub fn owner(env: Env) -> Option<Address> {
        ownable::get_owner(&env)
    }

    pub fn exchange(env: Env) -> Address {
        get_address(&env, &DataKey::Exchange)
    }

    pub fn funding_partner(env: Env) -> Address {
        get_address(&env, &DataKey::FundingPartner)
    }

    pub fn xlm_token(env: Env) -> Address {
        get_address(&env, &DataKey::XlmToken)
    }

    pub fn yield_token(env: Env) -> Address {
        get_address(&env, &DataKey::YieldToken)
    }

    pub fn partner_principal_xlm(env: Env) -> i128 {
        get_i128(&env, &DataKey::PartnerPrincipalXlm)
    }

    pub fn free_principal_xlm(env: Env) -> i128 {
        get_i128(&env, &DataKey::FreePrincipalXlm)
    }

    pub fn reserved_for_exchange_xlm(env: Env) -> i128 {
        get_i128(&env, &DataKey::ReservedForExchangeXlm)
    }

    pub fn collected_yield_usdt0(env: Env) -> i128 {
        get_i128(&env, &DataKey::CollectedYieldUsdt0)
    }

    pub fn yield_debt_usdt0(env: Env) -> i128 {
        get_i128(&env, &DataKey::YieldDebtUsdt0)
    }

    pub fn latest_settlement_epoch(env: Env) -> u64 {
        get_u64(&env, &DataKey::LatestSettlementEpoch)
    }

    pub fn last_set_reserve_exchange_rate(env: Env) -> i128 {
        get_i128(&env, &DataKey::LastSetReserveExchangeRate)
    }

    pub fn last_set_reserve_credit(env: Env) -> i128 {
        get_i128(&env, &DataKey::LastSetReserveCredit)
    }

    pub fn last_reserve_reference_hash(env: Env) -> Option<BytesN<32>> {
        env.storage()
            .instance()
            .get(&DataKey::LastReserveReferenceHash)
            .unwrap()
    }

    pub fn last_yield_reference_hash(env: Env) -> Option<BytesN<32>> {
        env.storage()
            .instance()
            .get(&DataKey::LastYieldSettlementReferenceHash)
            .unwrap()
    }

    pub fn xlm_balance(env: Env) -> i128 {
        xlm_client(&env).balance(&env.current_contract_address())
    }

    pub fn yield_balance(env: Env) -> i128 {
        yield_client(&env).balance(&env.current_contract_address())
    }
}

impl UpgradeableInternal for ManagedLiquidityVaultContract {
    fn _require_auth(e: &Env, operator: &Address) {
        // Upgrades are intentionally separated from business roles. The contract
        // owner is the sole upgrade authority and should be a governance address.
        operator.require_auth();
        let owner = ownable::get_owner(e).unwrap();
        if *operator != owner {
            panic_with_error!(e, ContractError::Unauthorized);
        }
    }
}

fn get_address(env: &Env, key: &DataKey) -> Address {
    env.storage().instance().get(key).unwrap()
}

fn get_i128(env: &Env, key: &DataKey) -> i128 {
    env.storage().instance().get(key).unwrap()
}

fn get_u64(env: &Env, key: &DataKey) -> u64 {
    env.storage().instance().get(key).unwrap()
}

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

    // `exchange_rate` is fixed-point USDT0 per 1 XLM, scaled by `RATE_SCALE`.
    // The check enforces 100% mark-value coverage only:
    // reserved_xlm * price >= reference_credit_usdt0.
    let covered_credit = checked_mul(target_reserved_xlm, exchange_rate)?;
    let covered_credit = covered_credit / RATE_SCALE;
    if covered_credit < reference_credit_usdt0 {
        return Err(ContractError::ReserveBelowRequiredCollateral);
    }
    Ok(())
}

fn checked_mul(lhs: i128, rhs: i128) -> Result<i128, ContractError> {
    lhs.checked_mul(rhs)
        .ok_or(ContractError::ArithmeticOverflow)
}

fn require_exchange_auth(env: &Env) {
    get_address(env, &DataKey::Exchange).require_auth();
}

fn require_partner_auth(env: &Env) {
    get_address(env, &DataKey::FundingPartner).require_auth();
}

fn require_exchange_and_partner_auth(env: &Env) {
    require_exchange_auth(env);
    require_partner_auth(env);
}

fn xlm_client<'a>(env: &'a Env) -> soroban_sdk::token::TokenClient<'a> {
    let token = get_address(env, &DataKey::XlmToken);
    soroban_sdk::token::TokenClient::new(env, &token)
}

fn yield_client<'a>(env: &'a Env) -> soroban_sdk::token::TokenClient<'a> {
    let token = get_address(env, &DataKey::YieldToken);
    soroban_sdk::token::TokenClient::new(env, &token)
}
