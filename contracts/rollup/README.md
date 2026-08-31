# Rollup Contract

A Soroban smart contract that holds the collateral pool for a rollup on the Stellar network. The pool backs user deposits, owner-posted withdrawal allowances, and owner-accrued fees.

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

Read-only functions: `owner`, `latest_block_hash`, `block_height`, `withdrawal_allowances(user)`, `fees`, `total_withdrawable`, and `collateral_balance`.

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
- `BlockHeight`: Number of blocks committed so far (`u32`). Each `rollup` increments it, and `NewBlockEvent` publishes it.
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
│   ├── certora     # Formal specification rules (feature `certora`)
│   │   ├── mod.rs
│   │   └── spec.rs
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
# `>|` overwrites the existing file also when the shell sets `noclobber`.
stellar contract bindings rust \
  --wasm ./target/wasm32v1-none/release/rollup_contract.wasm \
>| bindings/rollup_contract.rs
```

`contracts/rollup/Makefile` wraps the common targets: `make build`, `make test`, `make fmt`, and `make clean`.

## Formal Verification (Certora)

The crate carries formal specification rules for the Certora Prover behind the `certora` feature flag. The rules live in [spec.rs](./src/certora/spec.rs). The prover verifies all of them:

| Rule | Property |
| :--- | :--- |
| `sanity` | The contract state is reachable. |
| `deposit_rejects_non_positive_amount` | A deposit of a non-positive amount returns `DepositAmountMustBePositive`. |
| `rollup_rejects_negative_fees` | A rollup with negative fees returns `FeesMustBeNonNegative`. |
| `withdraw_clears_allowance` | A successful withdrawal leaves no allowance for the user. |
| `collect_fees_clears_balance` | A successful fee collection leaves no accrued fees. |
| `recover_rejects_non_positive_amount` | A recovery of a non-positive amount never transfers. |
| `renounce_ownership_always_rejected` | Ownership renunciation always returns `RenounceOwnershipDisabled`. |

Two limits shape that set, and [spec.rs](./src/certora/spec.rs) documents both. The prover treats the comparison of two `Address` or `BytesN` objects as uninterpreted, so the block-hash guards of `rollup` and the collateral-token guard of `recover` stay covered by the unit tests. A collateral transfer is a call into the token contract, and the prover does not converge on a rule that puts one on the path to the property, so each rule asserts a property that a single call establishes.

The Sunbeam scaffolding lives at the repository root: the build script [certora_build_rollup_contract.py](../../certora_build_rollup_contract.py) and the job configuration [certora/rollup_contract.conf](../../certora/rollup_contract.conf). Run the prover from the repository root:

```bash
# Activate the local Python environment so certoraSorobanProver is on PATH
source .venv/bin/activate

# Run a local compilation-only check
cd certora
certoraSorobanProver rollup_contract.conf --compilation_steps_only --short_output

# Run the full prover job after setting CERTORAKEY
certoraSorobanProver rollup_contract.conf
```

To compile the instrumented contract without the prover:

```bash
SOROBAN_SDK_BUILD_SYSTEM_SUPPORTS_SPEC_SHAKING_V2=1 \
RUSTFLAGS='-C link-arg=--allow-undefined' \
cargo build --target wasm32v1-none --release --package rollup-contract --features certora
```

Notes:

- The build script targets the Soroban artifact at `target/wasm32v1-none/release/rollup_contract.wasm`. Each rule is exported from that Wasm under its own name, and `certora/rollup_contract.conf` lists the rules to verify.
- The configuration sets `optimistic_loop`. Storage access converts a `DataKey` through `Val`, and the prover reads the length of the vector behind that conversion as a symbolic value, so no finite `loop_iter` closes the loop. No rule reasons about loop behaviour: the rules post empty withdrawal vectors, which leaves the withdrawal loop of `rollup` unreachable.
- The build script declares `SOROBAN_SDK_BUILD_SYSTEM_SUPPORTS_SPEC_SHAKING_V2`, because soroban-sdk 26 blocks a plain `cargo build` without it. The variable only affects contract-spec metadata, not the verified code.
- A workspace-local Cargo config in `.cargo/config.toml` disables a global GitHub HTTPS-to-SSH rewrite so public Certora dependencies can be fetched reliably.
- To skip the virtualenv activation, run `../.venv/bin/certoraSorobanProver` from the `certora/` directory instead.

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
- `cvlr`, `cvlr-soroban`, `cvlr-soroban-derive`: Certora verification harness. Optional, enabled by the `certora` feature.

Collateral transfers use `soroban_sdk::token::TokenClient`. The `stellar-tokens` entry in `Cargo.toml` is not referenced by the contract source.
