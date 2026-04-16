extern crate std;

use soroban_sdk::{
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env,
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
    Address,
    TokenClient<'_>,
    TokenClient<'_>,
    ManagedLiquidityVaultContractClient<'_>,
) {
    env.mock_all_auths();

    let owner = Address::generate(env);
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
            &owner,
        ),
    );
    let client = ManagedLiquidityVaultContractClient::new(env, &contract_id);

    xlm_admin_client.mint(&funding_partner, &20_0000000);
    yield_admin_client.mint(&exchange, &20_0000000);

    (
        owner,
        exchange,
        funding_partner,
        xlm_client,
        yield_client,
        client,
    )
}

#[test]
fn test_deployment() {
    let env = Env::default();
    let (owner, exchange, funding_partner, xlm_client, yield_client, client) = deploy_fixture(&env);

    assert_eq!(client.owner(), Some(owner));
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
    assert_eq!(
        client.last_settlement_memo_hash(),
        BytesN::from_array(&env, &[0; 32])
    );
}

#[test]
fn test_deposit_and_set_reserve() {
    let env = Env::default();
    let (_owner, _exchange, funding_partner, xlm_client, _yield_client, client) =
        deploy_fixture(&env);

    let deposit_amount = 10_0000000;
    let reserve_target = 2_0000000;
    let reference_credit = 160_0000000;
    let exchange_rate = 10_000000;
    let memo_hash = BytesN::from_array(&env, &[1; 32]);

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
        &memo_hash,
    );

    assert_eq!(client.partner_principal_xlm(), deposit_amount);
    assert_eq!(client.free_principal_xlm(), deposit_amount - reserve_target);
    assert_eq!(client.reserved_for_exchange_xlm(), reserve_target);
    assert_eq!(client.last_set_reserve_credit(), reference_credit);
    assert_eq!(client.last_set_reserve_exchange_rate(), exchange_rate);
    assert_eq!(client.last_settlement_memo_hash(), memo_hash);
    assert_eq!(client.xlm_balance(), deposit_amount);
}

#[test]
fn test_record_and_pay_yield_then_withdraw() {
    let env = Env::default();
    let (_owner, exchange, funding_partner, _xlm_client, yield_client, client) =
        deploy_fixture(&env);

    let settlement_paid = 4_0000000;
    let settlement_due = 10_0000000;
    let later_payment = 2_0000000;
    let memo_hash = BytesN::from_array(&env, &[2; 32]);

    yield_client.approve(&exchange, &client.address, &settlement_paid, &LEDGER_BUMP);
    client.record_yield_settlement(&1u64, &settlement_due, &settlement_paid, &memo_hash);

    assert_eq!(client.collected_yield_usdt0(), settlement_paid);
    assert_eq!(client.yield_debt_usdt0(), settlement_due - settlement_paid);
    assert_eq!(client.latest_settlement_epoch(), 1);

    yield_client.approve(&exchange, &client.address, &later_payment, &LEDGER_BUMP);
    client.pay_yield(&later_payment);

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
fn test_request_loss_payout_updates_balances() {
    let env = Env::default();
    let (_owner, _exchange, funding_partner, xlm_client, _yield_client, client) =
        deploy_fixture(&env);

    let deposit_amount = 10_0000000;
    let reserve_target = 2_0000000;
    let payout_amount = 1_5000000;
    let memo_hash = BytesN::from_array(&env, &[3; 32]);
    let settlement_wallet = Address::generate(&env);

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(&reserve_target, &160_0000000, &10_000000, &memo_hash);

    let initial_wallet_balance = xlm_client.balance(&settlement_wallet);
    client.request_loss_payout(&settlement_wallet, &payout_amount, &memo_hash);

    assert_eq!(
        client.partner_principal_xlm(),
        deposit_amount - payout_amount
    );
    assert_eq!(client.free_principal_xlm(), deposit_amount - reserve_target);
    assert_eq!(
        client.reserved_for_exchange_xlm(),
        reserve_target - payout_amount
    );
    assert_eq!(client.last_settlement_memo_hash(), memo_hash);
    assert_eq!(
        xlm_client.balance(&settlement_wallet),
        initial_wallet_balance + payout_amount
    );
}

#[test]
fn test_request_loss_payout_records_dual_auth() {
    let env = Env::default();
    let (_owner, exchange, funding_partner, xlm_client, _yield_client, client) =
        deploy_fixture(&env);

    let deposit_amount = 10_0000000;
    let reserve_target = 2_0000000;
    let payout_amount = 1_0000000;
    let memo_hash = BytesN::from_array(&env, &[4; 32]);
    let settlement_wallet = Address::generate(&env);

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(&reserve_target, &160_0000000, &10_000000, &memo_hash);

    let _ = env.auths();
    client.request_loss_payout(&settlement_wallet, &payout_amount, &memo_hash);

    let auths = env.auths();
    let addresses: std::vec::Vec<_> = auths.into_iter().map(|(addr, _)| addr).collect();
    assert!(addresses.contains(&exchange));
    assert!(addresses.contains(&funding_partner));
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_request_loss_payout_exceeds_reserve_panics() {
    let env = Env::default();
    let (_owner, _exchange, funding_partner, xlm_client, _yield_client, client) =
        deploy_fixture(&env);

    let deposit_amount = 10_0000000;
    let memo_hash = BytesN::from_array(&env, &[5; 32]);

    xlm_client.approve(
        &funding_partner,
        &client.address,
        &deposit_amount,
        &LEDGER_BUMP,
    );
    client.deposit_partner(&deposit_amount);
    client.set_reserve(&2_0000000, &160_0000000, &10_000000, &memo_hash);

    client.request_loss_payout(&Address::generate(&env), &3_0000000, &memo_hash);
}
