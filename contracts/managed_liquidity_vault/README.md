# Managed Liquidity Vault

This directory contains the Soroban smart contract for the **Managed Liquidity Vault**, a single-partner custody and settlement layer established between a **Funding Partner** and the **Exchange** (Rails).

The contract tracks two distinct assets:

1. **Principal/Collateral Asset** (`XLM`): Contributed by the funding partner to back the exchange's market-making activities.
2. **Yield/Fee Asset** (`USDT0`): Paid by the exchange as interest/yield on the reserved collateral, withdrawable only by the funding partner.

---

## Architecture & Integration Scope

The contract serves as an **auditable custody and settlement layer**. It intentionally does not implement trustless oracles, off-chain risk engines, or trading systems on-chain. Instead, it accepts authenticated inputs (such as exchange rates, reference credit levels, and settlement packages) from the configured `Exchange` and validates their mathematical consistency.

```mermaid
flowchart LR
    FP["Funding Partner"]
    EX["Exchange"]
    XP["XLM Token Contract"]
    YP["Yield Token Contract"]
    VAULT["Managed Liquidity Vault Contract"]
    OPS["Off-chain Exchange Operations"]
    PAYER["Approved Yield Payer"]

    subgraph TB1["Authorized External Actors"]
        FP
        EX
        PAYER
    end

    subgraph TB2["External Token Contracts"]
        XP
        YP
    end

    subgraph TB3["On-chain Vault State & Logic"]
        VAULT
    end

    subgraph TB4["Off-chain Exchange Systems"]
        OPS
    end

    FP -->|"deposit_partner(XLM)"| VAULT
    VAULT -->|"transfer_from / transfer XLM"| XP
    EX -->|"set_reserve(...)"| VAULT
    EX -->|"record_yield_settlement(...)"| VAULT
    PAYER -->|"pay_yield(...)"| VAULT
    VAULT -->|"transfer_from / transfer USDT0"| YP
    FP -->|"withdraw_partner_yield(...)"| VAULT
    FP -->|"co-sign withdraw_partner_principal(...)"| VAULT
    EX -->|"co-sign withdraw_partner_principal(...)"| VAULT
    EX -->|"co-sign upgrade(...)"| VAULT
    FP -->|"co-sign upgrade(...)"| VAULT
    EX -.->|"reserve calculations"| OPS
    OPS -.->|"reference packages & hashes"| EX
```

---

## Role and Authorization Matrix

The contract defines two main high-trust identities (`FundingPartner` and `Exchange`), which must be unique addresses initialized at deployment.

| Function                     | Authorization Requirement                     | Description                                                                          |
| :--------------------------- | :-------------------------------------------- | :----------------------------------------------------------------------------------- |
| `__constructor`              | _None_                                        | Initializes contract state, token links, and role addresses.                         |
| `deposit_partner`            | `FundingPartner`                              | Transfers `XLM` from the partner into the vault. Increases total and free principal. |
| `withdraw_partner_principal` | **Dual-Sign** (`Exchange` + `FundingPartner`) | Withdraws unreserved `XLM` principal from the vault.                                 |
| `set_reserve`                | `Exchange`                                    | Sets the amount of `XLM` locked as collateral. Checks collateral coverage.           |
| `record_yield_settlement`    | `Exchange`                                    | Records yield due for a settlement epoch and collects paid yield.                    |
| `pay_yield`                  | `from` (Caller)                               | Transfers `USDT0` yield tokens into the vault (open to any approved payer).          |
| `withdraw_partner_yield`     | `FundingPartner`                              | Withdraws accumulated `USDT0` yield from the vault.                                  |
| `recover_unaccounted_tokens` | `Exchange`                                    | Recovers token balances that the vault ledger does not account for.                  |
| `upgrade`                    | **Dual-Sign** (`Exchange` + `FundingPartner`) | Upgrades the contract's Wasm code hash.                                              |

### Constructor Validation

The role and token addresses are immutable after deployment, so the constructor
rejects any configuration that would produce an unusable or unsound vault:

- `xlm_token` and `yield_token` must differ, and `exchange` and
  `funding_partner` must differ.
- Neither role address may be one of the two token contracts.
- Both token addresses are probed through the SEP-41 `decimals` entrypoint.
  An address that is not a live contract implementing the token interface makes
  deployment fail, instead of yielding a vault whose transfers can never
  succeed.
- Both tokens must report the **same decimal precision**. The collateral
  coverage check cancels `RATE_SCALE` but does not normalize token precision, so
  it is only correct when the principal and yield tokens share the same number
  of decimals. Pinning this at construction turns a pre-deployment review item
  into an enforced on-chain constraint. The intended deployment uses XLM (7
  decimals) and a 7-decimal `USDT0` issuance on Stellar; a 6-decimal `USDT0`
  would be rejected at deployment rather than silently mis-priced.

### Recovery of Unaccounted Tokens

The vault holds real token balances, but only moves them through the principal
and yield flows it tracks. Two situations leave value stranded:

1. A third-party token is sent to the vault address. The vault has no ledger
   entry for it and no other entrypoint can move it.
2. The configured principal or yield token is transferred to the vault directly,
   bypassing `deposit_partner`, `pay_yield`, or `record_yield_settlement`. The
   real balance grows but `PartnerPrincipalXlm` / `CollectedYieldUsdt0` do not,
   so the surplus sits above what any withdrawal can reach.

`recover_unaccounted_tokens(token, to, amount)` recovers both cases. (The audit
refers to this as a sweep path; the entrypoint is named after the finding's
recommendation of a "carefully-authorized recovery path".) The recoverable
amount is reported by the `unaccounted_balance(token)` view:

- For the configured principal token: `balance − PartnerPrincipalXlm`.
- For the configured yield token: `balance − CollectedYieldUsdt0`.
- For any other token: the full balance.

The surplus is floored at zero, so tracked principal and collected yield can
never leave through this path — recovery cannot be used to drain partner funds
or to bypass the dual-signed principal withdrawal. Recovery is authorized by the
`Exchange` alone and emits `UnaccountedTokensRecoveredEvt`; returning recovered
funds to their rightful owner is an off-chain operational responsibility of the
`Exchange`.

---

## State Model & Core Invariants

### Storage Schema (`DataKey`)

All contract state is stored in the contract instance storage (`Storage::instance`), which is regularly bumped in TTL inside `extend_contract_ttl` on every state-changing invocation:

- `XlmToken`: Address of the principal token contract.
- `YieldToken`: Address of the yield token contract.
- `Exchange`: Address of the authorized exchange operator.
- `FundingPartner`: Address of the authorized funding partner.
- `PartnerPrincipalXlm`: Total principal contributed by the partner.
- `FreePrincipalXlm`: Portion of principal that is unreserved and withdrawable.
- `ReservedForExchangeXlm`: Portion of principal locked as exchange collateral.
- `CollectedYieldUsdt0`: Yield tokens paid in and available for withdrawal.
- `YieldDebtUsdt0`: Yield obligation recorded but not yet paid.
- `LatestSettlementEpoch`: ID of the most recently settled epoch.
- `LastSetReserveExchangeRate` / `LastSetReserveCredit`: Last audited exchange rate and off-chain credit level.
- `LastReserveReferenceHash` / `LastYieldSettlementReferenceHash`: Optional 32-byte hash link to off-chain audit packages.

### Enforced Invariants

1.  **Principal Balance Conservation**:
    $$\text{FreePrincipalXlm} + \text{ReservedForExchangeXlm} = \text{PartnerPrincipalXlm}$$
2.  **Yield Collection Backing**:
    `CollectedYieldUsdt0` only increases when `USDT0` is successfully transferred into the contract. Unpaid obligations remain in `YieldDebtUsdt0`.
3.  **Monotonic Epochs**:
    `epoch_id` must strictly increase in `record_yield_settlement`.
4.  **Collateral Coverage constraint**:
    If `reference_credit_usdt0 > 0`, then:
    $$\frac{\text{target\\_reserved\\_xlm} \times \text{exchange\\_rate}}{\text{RATE\\_SCALE}} \ge \text{reference\\_credit\\_usdt0}$$
    Where $\text{RATE\\_SCALE} = 10,000,000$

---

## Security Audit & Threat Model Highlights (STRIDE)

A STRIDE threat analysis is maintained in [stride-threat-model.md](./src/stride-threat-model.md). Key security design decisions include:

- **Authentication & Gating**: Every state-changing function requires explicit `.require_auth()` verification of the actor. The upgrade method and principal withdrawals require a joint multi-sig pattern where both parties must submit authorization.
- **Off-chain Input Tampering**: The contract does not verify the accuracy of the exchange rate or credit level. Off-chain reconciliation workflows must monitor `set_reserve` and `record_yield_settlement` events against trusted internal risk engine logs using the `reference_hash`.
- **Liveness Dependency**: If either the `Exchange` or `FundingPartner` key is lost or compromised, principal withdrawals and code upgrades are blocked. Strong multi-sig custody models must be implemented off-chain for both keys.
- **Instance Expiry (TTL)**: To prevent contract entries or Wasm bytecode from expiring due to low activity, `extend_contract_ttl` bumps instance TTL to $\sim 30$ days whenever state changes.

---

## Verification & Testing

### Formal Verification (Certora)

The codebase includes formal specification rules for the Certora Prover under the `certora` feature flag. The rules are located in [spec.rs](./src/certora/spec.rs) and cover validation of boundary cases, rejection of negative amounts, and authorization constraints.

To compile the contract with Certora instrumentation:

```bash
cargo build --target wasm32-unknown-unknown --release --features certora
```

### Unit Tests

A comprehensive test suite simulating deposits, withdrawals, reserve changes, and multi-sig operations is implemented in [test.rs](./src/test.rs).

To run the Rust unit tests:

```bash
cargo test
```
