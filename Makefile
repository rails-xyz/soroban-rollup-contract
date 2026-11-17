WASM_DIR := ./target/wasm32v1-none/release

.PHONY: build bindings bindings-rust bindings-ts

build:
	stellar contract build

# Regenerate all client bindings. Run after a change to a contract interface.
bindings: bindings-rust bindings-ts

bindings-rust: build
	stellar contract bindings rust --wasm $(WASM_DIR)/rollup_contract.wasm > bindings/rollup_contract.rs.tmp
	mv bindings/rollup_contract.rs.tmp bindings/rollup_contract.rs
	stellar contract bindings rust --wasm $(WASM_DIR)/managed_liquidity_vault.wasm > bindings/managed_liquidity_vault.rs.tmp
	mv bindings/managed_liquidity_vault.rs.tmp bindings/managed_liquidity_vault.rs

# The js-stellar-sdk generator (`npx @stellar/stellar-sdk generate`) replaces this
# deprecated command, but its output does not compile for contracts that return
# `Result<_, ContractError>`: it emits error enums as values only.
bindings-ts: build
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/rollup_contract.wasm \
		--output-dir bindings/ts/rollup \
		--overwrite
