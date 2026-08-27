# Rollup Contract

A Soroban smart contract that holds the collateral pool for a rollup on the Stellar network. The pool backs user deposits, owner-posted withdrawal allowances, and owner-accrued protocol fees.

The owner advances the rollup state by posting a new block hash together with the newly withdrawable balances and fees. Users then withdraw their allowance directly from the collateral pool.

## Overview

The `RollupContract` provides:

- **Deposits**: Users transfer collateral tokens into the contract.
- **Rollup**: An owner-only function that commits a new block hash and credits a batch of withdrawal allowances plus fees.
- **Withdrawals**: Users claim the full allowance credited to them by earlier rollup blocks.
- **Fee collection**: The owner transfers the accrued fees to a chosen recipient.
- **Recovery**: The owner recovers tokens that are not the collateral token.
- **Ownership and upgrades**: Two-step ownership transfer and owner-authorized Wasm upgrades. Ownership renunciation is disabled.

## Contract Interface

| Function                                     | Authorization | Description                                                                                    |
| :------------------------------------------- | :------------ | :--------------------------------------------------------------------------------------------- |
| `__constructor(collateral_token, owner)`     | _None_        | Sets the collateral token and owner. Initializes the block hash to zero, fees and totals to 0. |
| `deposit(user, amount)`                      | `user`        | Transfers `amount` of collateral token from `user` into the contract.                          |
| `rollup(old_block_hash, new_block_hash, …)`  | Owner         | Commits a new block hash and adds withdrawal allowances and fees.                              |
| `withdraw(user)`                             | `user`        | Transfers the full stored allowance to `user` and clears the entry.                            |
| `collect_fees(to)`                           | Owner         | Transfers all accrued fees to `to`.                                                            |
| `recover(token_address, to, amount)`         | Owner         | Transfers a non-collateral token out of the contract.                                          |
| `upgrade(new_wasm_hash, operator)`           | Owner         | Replaces the contract Wasm.                                                                    |
| `transfer_ownership(new_owner, live_until)`  | Owner         | Proposes a new owner. The proposal expires at `live_until`.                                    |
| `accept_ownership()`                         | Proposed owner | Completes the two-step ownership transfer.                                                    |
| `renounce_ownership()`                       | _None_        | Always fails with `RenounceOwnershipDisabled`.                                                 |

Read-only functions: `owner`, `latest_block_hash`, `withdrawal_allowances(user)`, `fees`, `total_withdrawable`, and `collateral_balance`.

### Rollup Constraints

`rollup` rejects a batch unless all of the following hold:

- `new_block_hash` is not the zero hash and differs from `old_block_hash`.
- `old_block_hash` matches the currently stored block hash.
- The address and amount arrays have equal length, and hold at most 100 entries.
- Every posted amount is non-negative, and `new_fees` is non-negative.
- The posted amounts add up to `new_withdrawal_sum`.
- The unreserved collateral balance covers `new_withdrawal_sum + new_fees`.

Existing user allowances are incremented, not replaced. All balance arithmetic is checked and fails with `ArithmeticOverflow` instead of wrapping.

## State

Instance storage:

- `LatestBlockHash`: Current rollup state root (`BytesN<32>`).
- `CollateralToken`: Address of the token used for deposits, withdrawals, and fees.
- `Fees`: Accrued fees available for collection.
- `TotalWithdrawable`: Total reserved for user withdrawals plus accrued fees.

Persistent storage:

- `WithdrawalAllowances(Address)`: Per-user withdrawal allowance. The entry is removed when the user withdraws.

Every state-changing call extends the instance TTL (and the Wasm code entry) to about 30 days, so the contract stays invocable between infrequent owner updates. Each allowance entry it touches gets the same extension.

## Project Structure

```text
contracts/rollup
├── src
│   ├── lib.rs      # Module declarations
│   ├── contract.rs # Contract implementation
│   └── test.rs     # Unit tests
├── test_snapshots  # Generated test ledger snapshots
├── Cargo.toml
├── Makefile
└── README.md
```

The crate is a member of the workspace at the repository root, next to the `managed_liquidity_vault` contract.

## Build Commands

Run these from the repository root.

```bash
# Build every contract in the workspace. Optimization is on by default.
stellar contract build

# Build only this contract
stellar contract build --package rollup-contract

# Build without the optimization pass
stellar contract build --optimize=false

# Build with cargo directly (no optimization pass)
cargo build --target wasm32v1-none --release

# Build with debug assertions enabled (for logging)
cargo build --target wasm32v1-none --profile release-with-logs

# Run the tests for this contract
cargo test --package rollup-contract

# Run a specific test
cargo test --package rollup-contract test_deposit

# Check code without building
cargo check

# Generate Rust bindings
stellar contract bindings rust \
  --wasm ./target/wasm32v1-none/release/rollup_contract.wasm \
  --out ./bindings/rollup_contract.rs
```

`contracts/rollup/Makefile` wraps the common targets: `make build`, `make test`, `make fmt`, and `make clean`.

## Environment Setup

A TESTNET Stellar node is live on staging. Add it as a network configuration for the stellar CLI:

```bash
stellar network add staging --url https://xdr.fungible.xyz --passphrase "Test SDF Network ; September 2022"
```

Then pass `--network staging` to interact with it:

```bash
stellar contract deploy \
  --network staging \
  --source-account <account> \
  --wasm ./target/wasm32v1-none/release/rollup_contract.wasm \
  -- \
  --collateral_token <token-address> \
  --owner <owner-address>
```

## Dependencies

- `soroban-sdk`: Core Soroban SDK (v26).
- `stellar-access`: Provides the `ownable` module for access control.
- `stellar-contract-utils`: Provides the `Upgradeable` trait and the `upgrade` helper that `upgrade` calls.
- `stellar-macros`: Procedural macros, including the `#[only_owner]` macro.

Collateral transfers use `soroban_sdk::token::TokenClient`. The `stellar-tokens` entry in `Cargo.toml` is not referenced by the contract source.
