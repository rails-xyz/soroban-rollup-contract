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
