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
```

## Dependencies

- `soroban-sdk`: Core Soroban SDK (v23)
- `stellar-access`: Provides `ownable` module and `#[only_owner]` macro for access control
- `stellar-macros`: Procedural macros for contract development
