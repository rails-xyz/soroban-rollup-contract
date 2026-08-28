# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build all contracts for production deploymeht
stellar contract build
# or
cargo build --target wasm32v1-none --release

# Further optimize the built wasm
stellar contract optimize --wasm target/wasm32v1-none/release/{name_of_production_build}.wasm

# Build with debug assertions enabled (for logging)
cargo build --target wasm32v1-none --release-with-logs

# Run tests
cargo test

# Run a specific test
cargo test test_deposit

# Check code without building
cargo check
```

## Architecture

This is a Soroban smart contract for a rollup system on the Stellar network. The main contract is in `contracts/rollup/`.

### Contract Overview

The `RollupContract` manages deposits, withdrawals, and fee collection for a Layer 1 rollup:

- **Deposits**: Users deposit collateral tokens into the contract
- **Rollup**: Owner-only function that processes batches of withdrawal allowances by verifying block hashes and updating user allowances
- **Withdrawals**: Users claim their allowances after they've been processed in a rollup
- **Fee Collection**: Owner collects accumulated fees

### Key Dependencies

- `stellar-cli`: For building and optimizing Soroban contracts
- `soroban-sdk`: Core Soroban SDK (v26)
- `stellar-access`: Provides `ownable` module for access control
- `stellar-macros`: Procedural macros for contract development, including `#[only_owner]`

### Storage Keys (DataKey enum)

- `LatestBlockHash`: Current rollup state root (BytesN<32>)
- `BlockHeight`: Number of blocks committed so far (u32), incremented by each rollup
- `CollateralToken`: Address of the token used for deposits/withdrawals
- `WithdrawalAllowances(Address)`: Per-user withdrawal allowances (persistent storage)
- `Fees`: Accumulated fees available for collection
- `TotalWithdrawable`: Total amount reserved for withdrawals + fees
