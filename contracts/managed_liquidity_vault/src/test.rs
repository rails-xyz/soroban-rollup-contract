extern crate std;

use soroban_sdk::{
    testutils::Address as _,
    testutils::{MockAuth, MockAuthInvoke},
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env, IntoVal,
};

use super::contract::{ManagedLiquidityVaultContract, ManagedLiquidityVaultContractClient};

const LEDGER_BUMP: u32 = 1_000_000;

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (TokenClient<'a>, StellarAssetClient<'a>) {
    let address = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    (
        TokenClient::new(env, &address),
        StellarAssetClient::new(env, &address),
    )
}

fn deploy_fixture(
    env: &Env,
) -> (
    Address,
    Address,
    TokenClient<'_>,
    TokenClient<'_>,
    ManagedLiquidityVaultContractClient<'_>,
) {
    env.mock_all_auths();

    let exchange = Address::generate(env);
    let funding_partner = Address::generate(env);

    let xlm_admin = Address::generate(env);
    let yield_admin = Address::generate(env);
    let (xlm_client, xlm_admin_client) = create_token_contract(env, &xlm_admin);
    let (yield_client, yield_admin_client) = create_token_contract(env, &yield_admin);

    let contract_id = env.register(
        ManagedLiquidityVaultContract,
        (
            &xlm_client.address,
            &yield_client.address,
            &exchange,
            &funding_partner,
        ),
    );
    let client = ManagedLiquidityVaultContractClient::new(env, &contract_id);

    xlm_admin_client.mint(&funding_partner, &20_000_0000000);
    yield_admin_client.mint(&exchange, &20_000_0000000);

    (exchange, funding_partner, xlm_client, yield_client, client)
}

fn assert_contract_error<F>(f: F, code: u32)
where
    F: FnOnce(),
{
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_err();
    let message = if let Some(message) = panic.downcast_ref::<std::string::String>() {
        message.clone()
    } else if let Some(message) = panic.downcast_ref::<&'static str>() {
        std::string::String::from(*message)
    } else {
        std::string::String::from("<non-string panic>")
    };

    assert!(
        message.contains(&std::format!("Error(Contract, #{code})")),
        "expected contract error #{code}, got: {message}"
    );
}

fn assert_panics<F>(f: F)
where
    F: FnOnce(),
{
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err());
}

#[test]
fn test_deployment() {
    let env = Env::default();
    let (exchange, funding_partner, xlm_client, yield_client, client) = deploy_fixture(&env);

    assert_eq!(client.exchange(), exchange);
    assert_eq!(client.funding_partner(), funding_partner);
    assert_eq!(client.xlm_token(), xlm_client.address);
    assert_eq!(client.yield_token(), yield_client.address);
    assert_eq!(client.partner_principal_xlm(), 0);
    assert_eq!(client.free_principal_xlm(), 0);
    assert_eq!(client.reserved_for_exchange_xlm(), 0);
    assert_eq!(client.collected_yield_usdt0(), 0);
    assert_eq!(client.yield_debt_usdt0(), 0);
    assert_eq!(client.latest_settlement_epoch(), 0);
    assert_eq!(client.last_reserve_reference_hash(), None);
    assert_eq!(client.last_yield_reference_hash(), None);
}

#[test]
fn test_deposit_and_set_reserve() {
    let env = Env::default();
    let (_exchange, funding_partner, xlm_client, _yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    let reserve_target = 2_000_0000000;
    let reference_credit = 160_0000000;
    let exchange_rate = 1_000000;
    let reference_hash = Some(BytesN::from_array(&env, &[1; 32]));

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(
        &reserve_target,
        &reference_credit,
        &exchange_rate,
        &reference_hash,
    );

    assert_eq!(client.partner_principal_xlm(), deposit_amount);
    assert_eq!(client.free_principal_xlm(), deposit_amount - reserve_target);
    assert_eq!(client.reserved_for_exchange_xlm(), reserve_target);
    assert_eq!(client.last_set_reserve_credit(), reference_credit);
    assert_eq!(client.last_set_reserve_exchange_rate(), exchange_rate);
    assert_eq!(client.last_reserve_reference_hash(), reference_hash);
    assert_eq!(client.last_yield_reference_hash(), None);
    assert_eq!(client.xlm_balance(), deposit_amount);
}

#[test]
fn test_record_and_pay_yield_then_withdraw() {
    let env = Env::default();
    let (exchange, funding_partner, _xlm_client, yield_client, client) = deploy_fixture(&env);

    let settlement_paid = 4_0000000;
    let settlement_due = 10_0000000;
    let later_payment = 2_0000000;
    let reference_hash = Some(BytesN::from_array(&env, &[2; 32]));

    yield_client.approve(&exchange, &client.address, &settlement_paid, &LEDGER_BUMP);
    client.record_yield_settlement(&1u64, &settlement_due, &settlement_paid, &reference_hash);

    assert_eq!(client.collected_yield_usdt0(), settlement_paid);
    assert_eq!(client.yield_debt_usdt0(), settlement_due - settlement_paid);
    assert_eq!(client.latest_settlement_epoch(), 1);
    assert_eq!(client.last_yield_reference_hash(), reference_hash);
    assert_eq!(client.last_reserve_reference_hash(), None);

    yield_client.approve(&exchange, &client.address, &later_payment, &LEDGER_BUMP);
    client.pay_yield(&exchange, &later_payment);

    assert_eq!(
        client.collected_yield_usdt0(),
        settlement_paid + later_payment
    );
    assert_eq!(
        client.yield_debt_usdt0(),
        settlement_due - settlement_paid - later_payment
    );

    let partner_initial = yield_client.balance(&funding_partner);
    client.withdraw_partner_yield(&funding_partner, &(settlement_paid + later_payment));
    assert_eq!(client.collected_yield_usdt0(), 0);
    assert_eq!(
        yield_client.balance(&funding_partner),
        partner_initial + settlement_paid + later_payment
    );
}

#[test]
fn test_pay_yield_accepts_alternate_payer() {
    let env = Env::default();
    let (exchange, _funding_partner, _xlm_client, yield_client, client) = deploy_fixture(&env);

    let fee_wallet = Address::generate(&env);
    let settlement_due = 10_0000000;
    let settlement_paid = 4_0000000;
    let later_payment = 2_0000000;

    yield_client.approve(&exchange, &client.address, &settlement_paid, &LEDGER_BUMP);
    client.record_yield_settlement(&1u64, &settlement_due, &settlement_paid, &None);

    yield_client.transfer(&exchange, &fee_wallet, &later_payment);
    yield_client.approve(&fee_wallet, &client.address, &later_payment, &LEDGER_BUMP);
    client.pay_yield(&fee_wallet, &later_payment);

    assert_eq!(
        client.collected_yield_usdt0(),
        settlement_paid + later_payment
    );
    assert_eq!(
        client.yield_debt_usdt0(),
        settlement_due - settlement_paid - later_payment
    );
}

#[test]
fn test_pay_yield_records_payer_not_exchange_auth() {
    let env = Env::default();
    let (exchange, _funding_partner, _xlm_client, yield_client, client) = deploy_fixture(&env);

    let fee_wallet = Address::generate(&env);
    let payment = 2_0000000;

    yield_client.transfer(&exchange, &fee_wallet, &payment);
    yield_client.approve(&fee_wallet, &client.address, &payment, &LEDGER_BUMP);

    let _ = env.auths();
    client.pay_yield(&fee_wallet, &payment);

    let auths = env.auths();
    let addresses: std::vec::Vec<_> = auths.into_iter().map(|(addr, _)| addr).collect();
    assert!(addresses.contains(&fee_wallet));
    assert!(!addresses.contains(&exchange));
}

#[test]
fn test_withdraw_partner_principal_updates_balances() {
    let env = Env::default();
    let (_exchange, funding_partner, xlm_client, _yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    let reserve_target = 2_000_0000000;
    let withdraw_amount = 1_500_0000000;
    let reserve_reference_hash = Some(BytesN::from_array(&env, &[3; 32]));

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(
        &reserve_target,
        &160_0000000,
        &1_000000,
        &reserve_reference_hash,
    );

    let initial_partner_balance = xlm_client.balance(&funding_partner);
    client.withdraw_partner_principal(&funding_partner, &withdraw_amount);

    assert_eq!(
        client.partner_principal_xlm(),
        deposit_amount - withdraw_amount
    );
    assert_eq!(
        client.free_principal_xlm(),
        deposit_amount - reserve_target - withdraw_amount
    );
    assert_eq!(client.reserved_for_exchange_xlm(), reserve_target);
    assert_eq!(client.last_reserve_reference_hash(), reserve_reference_hash);
    assert_eq!(
        xlm_client.balance(&funding_partner),
        initial_partner_balance + withdraw_amount
    );
}

#[test]
fn test_withdraw_partner_principal_records_dual_auth() {
    let env = Env::default();
    let (exchange, funding_partner, xlm_client, _yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    let reserve_target = 2_000_0000000;
    let withdraw_amount = 1_000_0000000;
    let reserve_reference_hash = Some(BytesN::from_array(&env, &[4; 32]));

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(
        &reserve_target,
        &160_0000000,
        &1_000000,
        &reserve_reference_hash,
    );

    let _ = env.auths();
    client.withdraw_partner_principal(&funding_partner, &withdraw_amount);

    let auths = env.auths();
    let addresses: std::vec::Vec<_> = auths.into_iter().map(|(addr, _)| addr).collect();
    assert!(addresses.contains(&exchange));
    assert!(addresses.contains(&funding_partner));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_withdraw_partner_principal_exceeds_free_principal_panics() {
    let env = Env::default();
    let (_exchange, funding_partner, xlm_client, _yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    let reserve_reference_hash = Some(BytesN::from_array(&env, &[5; 32]));

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(
        &2_000_0000000,
        &160_0000000,
        &1_000000,
        &reserve_reference_hash,
    );

    client.withdraw_partner_principal(&funding_partner, &9_000_0000000);
}

#[test]
fn test_set_reserve_can_clear_reference_hash_for_fx_only_update() {
    let env = Env::default();
    let (_exchange, funding_partner, xlm_client, _yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    let initial_reference_hash = Some(BytesN::from_array(&env, &[6; 32]));

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(
        &2_000_0000000,
        &160_0000000,
        &1_000000,
        &initial_reference_hash,
    );
    assert_eq!(client.last_reserve_reference_hash(), initial_reference_hash);

    client.set_reserve(&2_858_0000000, &160_0000000, &700000, &None);

    assert_eq!(client.reserved_for_exchange_xlm(), 2_858_0000000);
    assert_eq!(client.last_set_reserve_credit(), 160_0000000);
    assert_eq!(client.last_set_reserve_exchange_rate(), 700000);
    assert_eq!(client.last_reserve_reference_hash(), None);
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_set_reserve_rejects_under_collateralized_target() {
    let env = Env::default();
    let (_exchange, funding_partner, xlm_client, _yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    let reference_hash = Some(BytesN::from_array(&env, &[7; 32]));

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);

    client.set_reserve(&1_599_9999999, &160_0000000, &1_000000, &reference_hash);
}

#[test]
fn test_validation_errors() {
    let env = Env::default();
    let (_exchange, funding_partner, xlm_client, _yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);

    assert_contract_error(|| client.deposit_partner(&0), 1);
    assert_contract_error(
        || client.withdraw_partner_principal(&funding_partner, &0),
        2,
    );
    assert_contract_error(|| client.pay_yield(&funding_partner, &0), 3);
    assert_contract_error(|| client.withdraw_partner_yield(&funding_partner, &0), 3);
    assert_contract_error(|| client.withdraw_partner_yield(&funding_partner, &1), 7);
}

#[test]
fn test_set_reserve_validation_errors() {
    let env = Env::default();
    let (_exchange, funding_partner, xlm_client, _yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);

    assert_contract_error(|| client.set_reserve(&-1, &0, &0, &None), 4);
    assert_contract_error(|| client.set_reserve(&0, &-1, &0, &None), 13);
    assert_contract_error(
        || client.set_reserve(&(deposit_amount + 1), &0, &0, &None),
        5,
    );
    assert_contract_error(|| client.set_reserve(&1, &1, &0, &None), 16);
}

#[test]
fn test_record_yield_settlement_validation_errors() {
    let env = Env::default();
    let (exchange, _funding_partner, _xlm_client, yield_client, client) = deploy_fixture(&env);

    assert_contract_error(|| client.record_yield_settlement(&1, &-1, &0, &None), 8);

    yield_client.approve(&exchange, &client.address, &1_0000000, &LEDGER_BUMP);
    client.record_yield_settlement(&1, &1_0000000, &1_0000000, &None);

    assert_contract_error(|| client.record_yield_settlement(&1, &0, &0, &None), 10);
    assert_contract_error(
        || client.record_yield_settlement(&2, &1_0000000, &1_0000001, &None),
        9,
    );
}

#[test]
fn test_zero_credit_reserve_and_yield_balance() {
    let env = Env::default();
    let (exchange, funding_partner, xlm_client, yield_client, client) = deploy_fixture(&env);

    let deposit_amount = 10_000_0000000;
    let settlement_paid = 3_0000000;

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(&deposit_amount, &0, &0, &None);

    yield_client.approve(&exchange, &client.address, &settlement_paid, &LEDGER_BUMP);
    client.record_yield_settlement(&1, &settlement_paid, &settlement_paid, &None);

    assert_eq!(client.free_principal_xlm(), 0);
    assert_eq!(client.reserved_for_exchange_xlm(), deposit_amount);
    assert_eq!(client.yield_balance(), settlement_paid);
}

#[test]
fn test_upgrade_requires_exchange_and_funding_partner_auth() {
    let env = Env::default();
    let (exchange, funding_partner, _xlm_client, _yield_client, client) = deploy_fixture(&env);

    let wasm_hash = BytesN::from_array(&env, &[8; 32]);
    let outsider = Address::generate(&env);

    assert_contract_error(|| client.upgrade(&wasm_hash, &outsider), 15);
    assert_panics(|| {
        client
            .mock_auths(&[MockAuth {
                address: &exchange,
                invoke: &MockAuthInvoke {
                    contract: &client.address,
                    fn_name: "upgrade",
                    args: (&wasm_hash, &exchange).into_val(&env),
                    sub_invokes: &[],
                },
            }])
            .upgrade(&wasm_hash, &exchange)
    });
    assert_panics(|| {
        client
            .mock_auths(&[
                MockAuth {
                    address: &exchange,
                    invoke: &MockAuthInvoke {
                        contract: &client.address,
                        fn_name: "upgrade",
                        args: (&wasm_hash, &exchange).into_val(&env),
                        sub_invokes: &[],
                    },
                },
                MockAuth {
                    address: &funding_partner,
                    invoke: &MockAuthInvoke {
                        contract: &client.address,
                        fn_name: "upgrade",
                        args: (&wasm_hash, &exchange).into_val(&env),
                        sub_invokes: &[],
                    },
                },
            ])
            .upgrade(&wasm_hash, &outsider)
    });
    assert_panics(|| {
        client
            .mock_auths(&[
                MockAuth {
                    address: &exchange,
                    invoke: &MockAuthInvoke {
                        contract: &client.address,
                        fn_name: "upgrade",
                        args: (&wasm_hash, &exchange).into_val(&env),
                        sub_invokes: &[],
                    },
                },
                MockAuth {
                    address: &funding_partner,
                    invoke: &MockAuthInvoke {
                        contract: &client.address,
                        fn_name: "upgrade",
                        args: (&wasm_hash, &exchange).into_val(&env),
                        sub_invokes: &[],
                    },
                },
            ])
            .upgrade(&wasm_hash, &exchange)
    });
}
