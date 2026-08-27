# Soroban Contracts

Rails Soroban smart contracts for the Stellar network.

## Contracts

| Directory                                                            | Package                    | Description                                                                     |
| :------------------------------------------------------------------- | :------------------------- | :------------------------------------------------------------------------------ |
| [contracts/rollup](./contracts/rollup)                               | `rollup-contract`          | Collateral pool for a rollup: deposits, withdrawal allowances, and fees.        |
| [contracts/managed_liquidity_vault](./contracts/managed_liquidity_vault) | `managed-liquidity-vault`  | Custody and settlement layer between a funding partner and the exchange.        |
| [contracts/mock_usdt](./contracts/mock_usdt)                         | `mock-usdt`                | SEP-41 test token used in local and staging environments.                       |

Each contract directory holds its own README with the build, test, and deployment details.

## Common Commands

```bash
# Build every contract in the workspace
stellar contract build

# Build one contract
stellar contract build --package <package>

# Run all tests
cargo test

# Check code without building
cargo check
```

Generated client bindings live in [bindings](./bindings).
