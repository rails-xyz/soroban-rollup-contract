# Audit Responses

This document contains our responses to the Quantstamp initial report
"Stellar Foundation - Rails" (2026-07-29, audited commit `e86eb5b`).
Each section quotes a finding title and states how we address it.

## RAILS-1 — "Missing Sweep and Reconciliation Paths Strand Surplus or Mis-Sent Tokens"

**Severity:** Low. **Status:** Fixed.

We added `recover_unaccounted_tokens(token, to, amount)`, authorized
by the `Exchange`, and the view `unaccounted_balance(token)`. The view
reports the recoverable amount:

- Configured principal token: `balance − PartnerPrincipalXlm`.
- Configured yield token: `balance − CollectedYieldUsdt0`.
- All other tokens: the full balance.

The surplus has a floor of zero, so tracked principal and collected
yield cannot leave through this path. Each recovery emits
`UnaccountedTokensRecoveredEvt`. The return of recovered funds to
their owner is an off-chain responsibility of the `Exchange`.

The `Exchange` is the single signer for recovery, also when the
recovered balance is a mis-sent configured token. This is intentional:
the off-chain business contracts protect the funding partner, and the
partner has no role in the daily operation of the vault.

## RAILS-2 — "Improve constructor input validation"

**Severity:** Informational. **Status:** Fixed.

We added these checks to the constructor:

- A role address must not be one of the two token contracts
  (`RoleAddressMustNotBeToken`).
- The constructor calls the SEP-41 `decimals` entrypoint on each token
  address. Deployment fails if an address is not a live contract that
  exposes `decimals`. The probe does not prove the full token
  interface; that stays a pre-deployment review item, and a wrong
  token is corrected with a new deployment.
- The two tokens must report the same decimal precision
  (`TokenDecimalsMustMatch`). The collateral coverage check does not
  normalize the token precision, so equal precision is necessary. The
  constructor rejects a 6-decimal `USDT0` at deployment.

## RAILS-3 — "Record excess yield payments"

**Severity:** Informational. **Status:** Fixed.

`pay_yield` now records the payment surplus above the outstanding
debt in `TotalExcessYieldPaidUsdt0`, read with the new view
`total_excess_yield_paid_usdt0()`, and in the new `excess` field of
`YieldPaidEvt`. The full payment is still credited to collected
yield. The excess does not offset the due amount of a later
settlement epoch; that stays an off-chain settlement decision.

Operational note: the `Exchange` must record a settlement epoch
before it pays that epoch. A payment before the record is booked as
excess, and it does not decrease the debt that the record adds later.

## RAILS-4 — "Allow unilateral withdraw of excess XLM funds"

**Severity:** Informational. **Status:** Acknowledged.

We keep the dual signature on `withdraw_partner_principal`:

- `set_reserve` is a periodic posting of off-chain credit, not a live
  measurement. Exposure can change between postings. The co-signature
  lets the `Exchange` post a new reserve before principal leaves the
  vault.
- Outstanding `YieldDebtUsdt0` has no collateral, so the wind-down of
  the position stays a joint action.
- Joint approval gives each party non-repudiable evidence of consent
  (`Repudiate.2` in the STRIDE model).

The accepted cost is the shared liveness dependency tracked as
`DoS.1` / `DoS.2` in the STRIDE model.

## RAILS-5 — "Discrepancies with Documentation"

**Severity:** Informational. **Status:** Fixed.

We updated the design document and the README. The behavior of the
contract does not change.

- The documentation now states that the `Exchange` alone holds the
  upgrade authority and that there is no `Owner` role. This is an
  intentional centralization. The business partnership contract and
  the upgrade review procedures bound the residual risk off-chain
  (`Spoof.3`, `Elevation.1` in the STRIDE model).
- The documentation now states that `pay_yield` is intentionally
  permissionless. The method can only move value into the vault.
  Integrators must not use the payer identity as an authorization
  signal.

## S1 — "Record yield obligation"

**Status:** Acknowledged.

We keep the yield obligation off-chain in this revision. An on-chain
obligation makes the vault the authoritative source of the commercial
agreement, and each term change still arrives as an authenticated
exchange submission. These reconciliation controls apply instead:

- Each settlement is authenticated and epoch-monotonic, and the vault
  emits it as `YieldSettlementEvt`. The full obligation history is
  reconstructible from events.
- `LastYieldSettlementReferenceHash` binds each epoch to a retained
  off-chain settlement package.
- `CollectedYieldUsdt0` increases only with a real token transfer.
  `TotalExcessYieldPaidUsdt0` records payments above the recorded
  debt.

On-chain publication of the obligation parameters stays a candidate
for a future revision.
