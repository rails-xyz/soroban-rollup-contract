# Soroban Rollup Contract

A Soroban smart contract for managing deposits, withdrawals, and fee collection for a Layer 1 rollup on the Stellar network.

## Overview

The `RollupContract` enables:

- **Deposits**: Users deposit collateral tokens into the contract
- **Rollup**: Owner-only function that processes batches of withdrawal allowances by verifying block hashes and updating user allowances
- **Withdrawals**: Users claim their allowances after they've been processed in a rollup
- **Fee Collection**: Owner collects accumulated fees

## Project Structure

```text
.
├── contracts
│   └── rollup
│       ├── src
│       │   ├── lib.rs    # Main contract implementation
│       │   └── test.rs   # Unit tests
│       └── Cargo.toml
├── Cargo.toml            # Workspace configuration
└── README.md
```

## Build Commands

```bash
# Build all contracts for deployment
stellar contract build
# or
cargo build --target wasm32v1-none --release

# Further optimize the wasm to reduce size
stellar contract build --optimize

# Build with debug assertions enabled (for logging)
cargo build --target wasm32v1-none --release-with-logs

# Run tests
cargo test

# Run a specific test
cargo test test_deposit

# Check code without building
cargo check

# Generate bindings
stellar contract bindings rust --wasm ./target/wasm32v1-none/release/rollup_contract.wasm --out ./bindings/rollup_contract.rs

```

## Certora

Simple Sunbeam scaffolding for `managed_liquidity_vault` lives under `certora/` and `contracts/managed_liquidity_vault/src/certora/`.

```bash
# Activate the local Python environment so certoraSorobanProver is on PATH
source .venv/bin/activate

# Run a local compilation-only check
cd certora
certoraSorobanProver managed_liquidity_vault.conf --compilation_steps_only --short_output

# Run the full prover job after setting CERTORAKEY
certoraSorobanProver managed_liquidity_vault.conf
```

Notes:

- The build script targets the Soroban artifact produced at `target/wasm32v1-none/release/managed_liquidity_vault.wasm`.
- A workspace-local Cargo config in `.cargo/config.toml` disables a global GitHub HTTPS-to-SSH rewrite so public Certora dependencies can be fetched reliably.
- If you prefer not to activate the virtualenv, run `../.venv/bin/certoraSorobanProver` from the `certora/` directory instead.

## Environment Setup

We have a TESTNET Stellar Node live on staging. You can access this node with stellar cli by adding a network configuration:

```
stellar network add staging --url https://xdr.fungible.xyz --passphrase "Test SDF Network ; September 2022"
```

Then you can use the `--network staging` flag with stellar cli commands to interact with the staging network:

```
stellar contract deploy --network staging
```

## Dependencies

- `soroban-sdk`: Core Soroban SDK (v23)
- `stellar-access`: Provides `ownable` module and `#[only_owner]` macro for access control
- `stellar-macros`: Procedural macros for contract development
