import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




/**
 * Storage keys used by the managed liquidity vault.
 */
export type DataKey = {tag: "XlmToken", values: void} | {tag: "YieldToken", values: void} | {tag: "Exchange", values: void} | {tag: "FundingPartner", values: void} | {tag: "PartnerPrincipalXlm", values: void} | {tag: "FreePrincipalXlm", values: void} | {tag: "ReservedForExchangeXlm", values: void} | {tag: "CollectedYieldUsdt0", values: void} | {tag: "YieldDebtUsdt0", values: void} | {tag: "LatestSettlementEpoch", values: void} | {tag: "LastSetReserveExchangeRate", values: void} | {tag: "LastSetReserveCredit", values: void} | {tag: "LastReserveReferenceHash", values: void} | {tag: "LastYieldSettlementReferenceHash", values: void};


/**
 * Errors returned by the managed liquidity vault.
 */
export const ContractError = {
  1: {message:"DepositAmountMustBePositive"},
  2: {message:"WithdrawAmountMustBePositive"},
  3: {message:"YieldAmountMustBePositive"},
  4: {message:"TargetReserveMustBeNonNegative"},
  5: {message:"ReserveExceedsPrincipal"},
  6: {message:"InsufficientFreePrincipal"},
  7: {message:"InsufficientCollectedYield"},
  8: {message:"YieldSettlementAmountsMustBeNonNegative"},
  9: {message:"YieldPaidExceedsDebtAndCurrentDue"},
  10: {message:"SettlementEpochMustIncrease"},
  13: {message:"AuditValuesMustBeNonNegative"},
  15: {message:"Unauthorized"},
  16: {message:"ExchangeRateMustBePositive"},
  17: {message:"ReserveBelowRequiredCollateral"},
  19: {message:"ArithmeticOverflow"},
  20: {message:"PrincipalAndYieldTokenMustDiffer"},
  21: {message:"ExchangeAndPartnerMustDiffer"}
}






export const UpgradeableError = {
  /**
   * When migration is attempted but not allowed due to upgrade state.
   */
  1100: {message:"MigrationNotAllowed"}
}



export const MerkleDistributorError = {
  /**
   * The merkle root is not set.
   */
  1300: {message:"RootNotSet"},
  /**
   * The provided index was already claimed.
   */
  1301: {message:"IndexAlreadyClaimed"},
  /**
   * The proof is invalid.
   */
  1302: {message:"InvalidProof"}
}

/**
 * Storage keys for the data associated with `MerkleDistributor`
 */
export type MerkleDistributorStorageKey = {tag: "Root", values: void} | {tag: "Claimed", values: readonly [u32]};

export type Rounding = {tag: "Floor", values: void} | {tag: "Ceil", values: void};

export const SorobanFixedPointError = {
  /**
   * The operation failed because the denominator is 0.
   */
  1500: {message:"ZeroDenominator"},
  /**
   * The operation failed because a phantom overflow occurred.
   */
  1501: {message:"PhantomOverflow"},
  /**
   * The operation failed because the result does not fit in Self.
   */
  1502: {message:"ResultOverflow"}
}

export const CryptoError = {
  /**
   * The merkle proof length is out of bounds.
   */
  1400: {message:"MerkleProofOutOfBounds"},
  /**
   * The index of the leaf is out of bounds.
   */
  1401: {message:"MerkleIndexOutOfBounds"},
  /**
   * No data in hasher state.
   */
  1402: {message:"HasherEmptyState"}
}



export const PausableError = {
  /**
   * The operation failed because the contract is paused.
   */
  1000: {message:"EnforcedPause"},
  /**
   * The operation failed because the contract is not paused.
   */
  1001: {message:"ExpectedPause"}
}

/**
 * Storage key for the pausable state
 */
export type PausableStorageKey = {tag: "Paused", values: void};

export interface Client {
  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash, operator}: {new_wasm_hash: Buffer, operator: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a exchange transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the exchange address.
   */
  exchange: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a pay_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Pays yield into the vault outside epoch settlement.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `from` - Address supplying the yield token transfer.
   * * `amount_usdt0` - Amount of yield token to transfer into the vault.
   * 
   * # Errors
   * 
   * * [`ContractError::YieldAmountMustBePositive`] - If `amount_usdt0 <= 0`.
   * 
   * # Notes
   * 
   * * Authorization from `from` is required at the root invocation.
   * * This method can only improve the funding partner position by reducing
   * debt and/or increasing collected yield.
   */
  pay_yield: ({from, amount_usdt0}: {from: string, amount_usdt0: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a xlm_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the principal token contract address.
   */
  xlm_token: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a set_reserve transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sets the exchange reserve target.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `target_reserved_xlm` - Total principal to reserve as exchange
   * collateral after this call.
   * * `reference_credit_usdt0` - Off-chain exchange credit amount that the
   * reserve is expected to cover.
   * * `exchange_rate` - Fixed-point price of `USDT0 per 1 XLM`, scaled by
   * [`RATE_SCALE`].
   * * `reference_hash` - Optional audit reference for the off-chain reserve
   * calculation or credit movement.
   * 
   * # Errors
   * 
   * * [`ContractError::TargetReserveMustBeNonNegative`] - If
   * `target_reserved_xlm < 0`.
   * * [`ContractError::AuditValuesMustBeNonNegative`] - If
   * `reference_credit_usdt0 < 0` or `exchange_rate < 0`.
   * * [`ContractError::ReserveExceedsPrincipal`] - If the reserve target is
   * larger than total partner principal.
   * * [`ContractError::ExchangeRateMustBePositive`] - If a positive credit
   * amount is posted with a zero or negative exchange rate.
   * * [`ContractError::ReserveBelowRequiredCollateral`] - If the posted
   * reserve does not cover the stated credit
   */
  set_reserve: ({target_reserved_xlm, reference_credit_usdt0, exchange_rate, reference_hash}: {target_reserved_xlm: i128, reference_credit_usdt0: i128, exchange_rate: i128, reference_hash: Option<Buffer>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a xlm_balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current principal token balance held by the vault.
   */
  xlm_balance: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a yield_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the yield token contract address.
   */
  yield_token: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a yield_balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current yield token balance held by the vault.
   */
  yield_balance: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a deposit_partner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits principal into the vault.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `amount_xlm` - Amount of principal token to deposit.
   * 
   * # Errors
   * 
   * * [`ContractError::DepositAmountMustBePositive`] - If `amount_xlm <= 0`.
   * 
   * # Notes
   * 
   * * Authorization from `FundingPartner` is required.
   * * No share token is minted because this design supports a single funding
   * partner only.
   */
  deposit_partner: ({amount_xlm}: {amount_xlm: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a funding_partner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the funding partner address.
   */
  funding_partner: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a yield_debt_usdt0 transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns yield owed by the exchange but not yet paid.
   */
  yield_debt_usdt0: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a free_principal_xlm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns principal that is not currently reserved.
   */
  free_principal_xlm: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a collected_yield_usdt0 transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns yield actually paid into the vault.
   */
  collected_yield_usdt0: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a partner_principal_xlm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns total partner principal tracked by the vault.
   */
  partner_principal_xlm: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a withdraw_partner_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraws collected yield from the vault.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `to` - Recipient of withdrawn yield.
   * * `amount_usdt0` - Amount of collected yield to withdraw.
   * 
   * # Errors
   * 
   * * [`ContractError::YieldAmountMustBePositive`] - If `amount_usdt0 <= 0`.
   * * [`ContractError::InsufficientCollectedYield`] - If `amount_usdt0`
   * exceeds collected yield balance.
   * 
   * # Notes
   * 
   * * Authorization from `FundingPartner` is required.
   */
  withdraw_partner_yield: ({to, amount_usdt0}: {to: string, amount_usdt0: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a last_set_reserve_credit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the last reserve-set reference credit.
   */
  last_set_reserve_credit: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a latest_settlement_epoch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the latest recorded yield settlement epoch.
   */
  latest_settlement_epoch: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a record_yield_settlement transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Records yield debt and any concurrent yield payment for a settlement
   * epoch.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `epoch_id` - Monotonically increasing settlement epoch identifier.
   * * `yield_due_usdt0` - Additional yield obligation created by this epoch.
   * * `yield_paid_usdt0` - Portion of total outstanding yield paid now.
   * * `reference_hash` - Optional audit reference for the off-chain
   * settlement package.
   * 
   * # Errors
   * 
   * * [`ContractError::YieldSettlementAmountsMustBeNonNegative`] - If either
   * yield amount is negative.
   * * [`ContractError::SettlementEpochMustIncrease`] - If `epoch_id` is not
   * greater than the previously recorded epoch.
   * * [`ContractError::YieldPaidExceedsDebtAndCurrentDue`] - If the payment
   * exceeds prior debt plus current epoch due.
   * 
   * # Notes
   * 
   * * Authorization from `Exchange` is required.
   * * `yield_paid_usdt0` must be backed by an actual token transfer.
   * * Unpaid yield remains debt and is not added to collected yield.
   */
  record_yield_settlement: ({epoch_id, yield_due_usdt0, yield_paid_usdt0, reference_hash}: {epoch_id: u64, yield_due_usdt0: i128, yield_paid_usdt0: i128, reference_hash: Option<Buffer>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a last_yield_reference_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the most recent optional yield-settlement audit reference.
   */
  last_yield_reference_hash: (options?: MethodOptions) => Promise<AssembledTransaction<Option<Buffer>>>

  /**
   * Construct and simulate a reserved_for_exchange_xlm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns principal currently reserved for exchange collateral.
   */
  reserved_for_exchange_xlm: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a withdraw_partner_principal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraws free principal from the vault.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `to` - Recipient of the withdrawn principal.
   * * `amount_xlm` - Amount of free principal to withdraw.
   * 
   * # Errors
   * 
   * * [`ContractError::WithdrawAmountMustBePositive`] - If `amount_xlm <= 0`.
   * * [`ContractError::InsufficientFreePrincipal`] - If `amount_xlm`
   * exceeds the unreserved principal balance.
   * 
   * # Notes
   * 
   * * Authorization from both `Exchange` and `FundingPartner` is required.
   * * Reserved principal remains encumbered behind exchange credit and is
   * never withdrawable through this method.
   */
  withdraw_partner_principal: ({to, amount_xlm}: {to: string, amount_xlm: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a last_reserve_reference_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the most recent optional reserve audit reference.
   */
  last_reserve_reference_hash: (options?: MethodOptions) => Promise<AssembledTransaction<Option<Buffer>>>

  /**
   * Construct and simulate a last_set_reserve_exchange_rate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the last reserve-set exchange rate.
   */
  last_set_reserve_exchange_rate: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {xlm_token, yield_token, exchange, funding_partner}: {xlm_token: string, yield_token: string, exchange: string, funding_partner: string},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({xlm_token, yield_token, exchange, funding_partner}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAgAAADFTdG9yYWdlIGtleXMgdXNlZCBieSB0aGUgbWFuYWdlZCBsaXF1aWRpdHkgdmF1bHQuAAAAAAAAAAAAAAdEYXRhS2V5AAAAAA4AAAAAAAAAAAAAAAhYbG1Ub2tlbgAAAAAAAAAAAAAACllpZWxkVG9rZW4AAAAAAAAAAAAAAAAACEV4Y2hhbmdlAAAAAAAAAAAAAAAORnVuZGluZ1BhcnRuZXIAAAAAAAAAAAAAAAAAE1BhcnRuZXJQcmluY2lwYWxYbG0AAAAAAAAAAAAAAAAQRnJlZVByaW5jaXBhbFhsbQAAAAAAAAAAAAAAFlJlc2VydmVkRm9yRXhjaGFuZ2VYbG0AAAAAAAAAAAAAAAAAE0NvbGxlY3RlZFlpZWxkVXNkdDAAAAAAAAAAAAAAAAAOWWllbGREZWJ0VXNkdDAAAAAAAAAAAAAAAAAAFUxhdGVzdFNldHRsZW1lbnRFcG9jaAAAAAAAAAAAAAAAAAAAGkxhc3RTZXRSZXNlcnZlRXhjaGFuZ2VSYXRlAAAAAAAAAAAAAAAAABRMYXN0U2V0UmVzZXJ2ZUNyZWRpdAAAAAAAAAAAAAAAGExhc3RSZXNlcnZlUmVmZXJlbmNlSGFzaAAAAAAAAAAAAAAAIExhc3RZaWVsZFNldHRsZW1lbnRSZWZlcmVuY2VIYXNo",
        "AAAABQAAAERFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGV4Y2hhbmdlIHBheXMgeWllbGQgb3V0c2lkZSBlcG9jaCBzZXR0bGVtZW50LgAAAAAAAAAMWWllbGRQYWlkRXZ0AAAAAQAAAA55aWVsZF9wYWlkX2V2dAAAAAAAAQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAI=",
        "AAAABAAAAC9FcnJvcnMgcmV0dXJuZWQgYnkgdGhlIG1hbmFnZWQgbGlxdWlkaXR5IHZhdWx0LgAAAAAAAAAADUNvbnRyYWN0RXJyb3IAAAAAAAARAAAAAAAAABtEZXBvc2l0QW1vdW50TXVzdEJlUG9zaXRpdmUAAAAAAQAAAAAAAAAcV2l0aGRyYXdBbW91bnRNdXN0QmVQb3NpdGl2ZQAAAAIAAAAAAAAAGVlpZWxkQW1vdW50TXVzdEJlUG9zaXRpdmUAAAAAAAADAAAAAAAAAB5UYXJnZXRSZXNlcnZlTXVzdEJlTm9uTmVnYXRpdmUAAAAAAAQAAAAAAAAAF1Jlc2VydmVFeGNlZWRzUHJpbmNpcGFsAAAAAAUAAAAAAAAAGUluc3VmZmljaWVudEZyZWVQcmluY2lwYWwAAAAAAAAGAAAAAAAAABpJbnN1ZmZpY2llbnRDb2xsZWN0ZWRZaWVsZAAAAAAABwAAAAAAAAAnWWllbGRTZXR0bGVtZW50QW1vdW50c011c3RCZU5vbk5lZ2F0aXZlAAAAAAgAAAAAAAAAIVlpZWxkUGFpZEV4Y2VlZHNEZWJ0QW5kQ3VycmVudER1ZQAAAAAAAAkAAAAAAAAAG1NldHRsZW1lbnRFcG9jaE11c3RJbmNyZWFzZQAAAAAKAAAAAAAAABxBdWRpdFZhbHVlc011c3RCZU5vbk5lZ2F0aXZlAAAADQAAAAAAAAAMVW5hdXRob3JpemVkAAAADwAAAAAAAAAaRXhjaGFuZ2VSYXRlTXVzdEJlUG9zaXRpdmUAAAAAABAAAAAAAAAAHlJlc2VydmVCZWxvd1JlcXVpcmVkQ29sbGF0ZXJhbAAAAAAAEQAAAAAAAAASQXJpdGhtZXRpY092ZXJmbG93AAAAAAATAAAAAAAAACBQcmluY2lwYWxBbmRZaWVsZFRva2VuTXVzdERpZmZlcgAAABQAAAAAAAAAHEV4Y2hhbmdlQW5kUGFydG5lck11c3REaWZmZXIAAAAV",
        "AAAABQAAAEBFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGV4Y2hhbmdlIHNldHMgdGhlIGN1cnJlbnQgcmVzZXJ2ZSB0YXJnZXQuAAAAAAAAAA1SZXNlcnZlU2V0RXZ0AAAAAAAAAQAAAA9yZXNlcnZlX3NldF9ldnQAAAAABAAAAAAAAAATdGFyZ2V0X3Jlc2VydmVkX3hsbQAAAAALAAAAAAAAAAAAAAAWcmVmZXJlbmNlX2NyZWRpdF91c2R0MAAAAAAACwAAAAAAAAAAAAAADWV4Y2hhbmdlX3JhdGUAAAAAAAALAAAAAAAAAAAAAAAOcmVmZXJlbmNlX2hhc2gAAAAAA+gAAAPuAAAAIAAAAAAAAAAC",
        "AAAABQAAAElFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGZ1bmRpbmcgcGFydG5lciBkZXBvc2l0cyBwcmluY2lwYWwgaW50byB0aGUgdmF1bHQuAAAAAAAAAAAAABFQYXJ0bmVyRGVwb3NpdEV2dAAAAAAAAAEAAAATcGFydG5lcl9kZXBvc2l0X2V2dAAAAAABAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAEFFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGZ1bmRpbmcgcGFydG5lciB3aXRoZHJhd3MgY29sbGVjdGVkIHlpZWxkLgAAAAAAAAAAAAASUGFydG5lcllpZWxkT3V0RXZ0AAAAAAABAAAAFXBhcnRuZXJfeWllbGRfb3V0X2V2dAAAAAAAAAIAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAI=",
        "AAAABQAAADdFdmVudCBlbWl0dGVkIHdoZW4geWllbGQgZGVidCBmb3IgYW4gZXBvY2ggaXMgcmVjb3JkZWQuAAAAAAAAAAASWWllbGRTZXR0bGVtZW50RXZ0AAAAAAABAAAAFHlpZWxkX3NldHRsZW1lbnRfZXZ0AAAABAAAAAAAAAAIZXBvY2hfaWQAAAAGAAAAAAAAAAAAAAAPeWllbGRfZHVlX3VzZHQwAAAAAAsAAAAAAAAAAAAAABB5aWVsZF9wYWlkX3VzZHQwAAAACwAAAAAAAAAAAAAADnJlZmVyZW5jZV9oYXNoAAAAAAPoAAAD7gAAACAAAAAAAAAAAg==",
        "AAAABQAAADNFdmVudCBlbWl0dGVkIHdoZW4gZnJlZSBwcmluY2lwYWwgbGVhdmVzIHRoZSB2YXVsdC4AAAAAAAAAABZQYXJ0bmVyUHJpbmNpcGFsT3V0RXZ0AAAAAAABAAAAGXBhcnRuZXJfcHJpbmNpcGFsX291dF9ldnQAAAAAAAACAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAC",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAACAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAAAAAACG9wZXJhdG9yAAAAEwAAAAA=",
        "AAAAAAAAAB1SZXR1cm5zIHRoZSBleGNoYW5nZSBhZGRyZXNzLgAAAAAAAAhleGNoYW5nZQAAAAAAAAABAAAAEw==",
        "AAAAAAAAAfhQYXlzIHlpZWxkIGludG8gdGhlIHZhdWx0IG91dHNpZGUgZXBvY2ggc2V0dGxlbWVudC4KCiMgQXJndW1lbnRzCgoqIGBlbnZgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGBmcm9tYCAtIEFkZHJlc3Mgc3VwcGx5aW5nIHRoZSB5aWVsZCB0b2tlbiB0cmFuc2Zlci4KKiBgYW1vdW50X3VzZHQwYCAtIEFtb3VudCBvZiB5aWVsZCB0b2tlbiB0byB0cmFuc2ZlciBpbnRvIHRoZSB2YXVsdC4KCiMgRXJyb3JzCgoqIFtgQ29udHJhY3RFcnJvcjo6WWllbGRBbW91bnRNdXN0QmVQb3NpdGl2ZWBdIC0gSWYgYGFtb3VudF91c2R0MCA8PSAwYC4KCiMgTm90ZXMKCiogQXV0aG9yaXphdGlvbiBmcm9tIGBmcm9tYCBpcyByZXF1aXJlZCBhdCB0aGUgcm9vdCBpbnZvY2F0aW9uLgoqIFRoaXMgbWV0aG9kIGNhbiBvbmx5IGltcHJvdmUgdGhlIGZ1bmRpbmcgcGFydG5lciBwb3NpdGlvbiBieSByZWR1Y2luZwpkZWJ0IGFuZC9vciBpbmNyZWFzaW5nIGNvbGxlY3RlZCB5aWVsZC4AAAAJcGF5X3lpZWxkAAAAAAAAAgAAAAAAAAAEZnJvbQAAABMAAAAAAAAADGFtb3VudF91c2R0MAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAC1SZXR1cm5zIHRoZSBwcmluY2lwYWwgdG9rZW4gY29udHJhY3QgYWRkcmVzcy4AAAAAAAAJeGxtX3Rva2VuAAAAAAAAAAAAAAEAAAAT",
        "AAAAAAAABABTZXRzIHRoZSBleGNoYW5nZSByZXNlcnZlIHRhcmdldC4KCiMgQXJndW1lbnRzCgoqIGBlbnZgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGB0YXJnZXRfcmVzZXJ2ZWRfeGxtYCAtIFRvdGFsIHByaW5jaXBhbCB0byByZXNlcnZlIGFzIGV4Y2hhbmdlCmNvbGxhdGVyYWwgYWZ0ZXIgdGhpcyBjYWxsLgoqIGByZWZlcmVuY2VfY3JlZGl0X3VzZHQwYCAtIE9mZi1jaGFpbiBleGNoYW5nZSBjcmVkaXQgYW1vdW50IHRoYXQgdGhlCnJlc2VydmUgaXMgZXhwZWN0ZWQgdG8gY292ZXIuCiogYGV4Y2hhbmdlX3JhdGVgIC0gRml4ZWQtcG9pbnQgcHJpY2Ugb2YgYFVTRFQwIHBlciAxIFhMTWAsIHNjYWxlZCBieQpbYFJBVEVfU0NBTEVgXS4KKiBgcmVmZXJlbmNlX2hhc2hgIC0gT3B0aW9uYWwgYXVkaXQgcmVmZXJlbmNlIGZvciB0aGUgb2ZmLWNoYWluIHJlc2VydmUKY2FsY3VsYXRpb24gb3IgY3JlZGl0IG1vdmVtZW50LgoKIyBFcnJvcnMKCiogW2BDb250cmFjdEVycm9yOjpUYXJnZXRSZXNlcnZlTXVzdEJlTm9uTmVnYXRpdmVgXSAtIElmCmB0YXJnZXRfcmVzZXJ2ZWRfeGxtIDwgMGAuCiogW2BDb250cmFjdEVycm9yOjpBdWRpdFZhbHVlc011c3RCZU5vbk5lZ2F0aXZlYF0gLSBJZgpgcmVmZXJlbmNlX2NyZWRpdF91c2R0MCA8IDBgIG9yIGBleGNoYW5nZV9yYXRlIDwgMGAuCiogW2BDb250cmFjdEVycm9yOjpSZXNlcnZlRXhjZWVkc1ByaW5jaXBhbGBdIC0gSWYgdGhlIHJlc2VydmUgdGFyZ2V0IGlzCmxhcmdlciB0aGFuIHRvdGFsIHBhcnRuZXIgcHJpbmNpcGFsLgoqIFtgQ29udHJhY3RFcnJvcjo6RXhjaGFuZ2VSYXRlTXVzdEJlUG9zaXRpdmVgXSAtIElmIGEgcG9zaXRpdmUgY3JlZGl0CmFtb3VudCBpcyBwb3N0ZWQgd2l0aCBhIHplcm8gb3IgbmVnYXRpdmUgZXhjaGFuZ2UgcmF0ZS4KKiBbYENvbnRyYWN0RXJyb3I6OlJlc2VydmVCZWxvd1JlcXVpcmVkQ29sbGF0ZXJhbGBdIC0gSWYgdGhlIHBvc3RlZApyZXNlcnZlIGRvZXMgbm90IGNvdmVyIHRoZSBzdGF0ZWQgY3JlZGl0AAAAC3NldF9yZXNlcnZlAAAAAAQAAAAAAAAAE3RhcmdldF9yZXNlcnZlZF94bG0AAAAACwAAAAAAAAAWcmVmZXJlbmNlX2NyZWRpdF91c2R0MAAAAAAACwAAAAAAAAANZXhjaGFuZ2VfcmF0ZQAAAAAAAAsAAAAAAAAADnJlZmVyZW5jZV9oYXNoAAAAAAPoAAAD7gAAACAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAD5SZXR1cm5zIHRoZSBjdXJyZW50IHByaW5jaXBhbCB0b2tlbiBiYWxhbmNlIGhlbGQgYnkgdGhlIHZhdWx0LgAAAAAAC3hsbV9iYWxhbmNlAAAAAAAAAAABAAAACw==",
        "AAAAAAAAAClSZXR1cm5zIHRoZSB5aWVsZCB0b2tlbiBjb250cmFjdCBhZGRyZXNzLgAAAAAAAAt5aWVsZF90b2tlbgAAAAAAAAAAAQAAABM=",
        "AAAAAAAAAglJbml0aWFsaXplcyB0aGUgdmF1bHQuCgojIEFyZ3VtZW50cwoKKiBgZW52YCAtIEFjY2VzcyB0byB0aGUgU29yb2JhbiBlbnZpcm9ubWVudC4KKiBgeGxtX3Rva2VuYCAtIFRva2VuIGNvbnRyYWN0IHVzZWQgYXMgdmF1bHQgcHJpbmNpcGFsIGFuZCBjb2xsYXRlcmFsLgoqIGB5aWVsZF90b2tlbmAgLSBUb2tlbiBjb250cmFjdCB1c2VkIGZvciBleHRlcm5hbGx5IHBhaWQgeWllbGQuCiogYGV4Y2hhbmdlYCAtIEFkZHJlc3MgYXV0aG9yaXplZCB0byBtYW5hZ2UgcmVzZXJ2ZSBhbmQgeWllbGQKc2V0dGxlbWVudC4KKiBgZnVuZGluZ19wYXJ0bmVyYCAtIEFkZHJlc3MgYXV0aG9yaXplZCB0byBkZXBvc2l0IHByaW5jaXBhbCBhbmQKd2l0aGRyYXcgY29sbGVjdGVkIHlpZWxkLgoKIyBOb3RlcwoKKiBVcGdyYWRlcyByZXF1aXJlIGJvdGggdGhlIGNvbmZpZ3VyZWQgYEV4Y2hhbmdlYCBhbmQKYEZ1bmRpbmdQYXJ0bmVyYC4KKiBQcmluY2lwYWwgYW5kIHlpZWxkIGJhbGFuY2VzIGFyZSBpbml0aWFsaXplZCB0byB6ZXJvLgAAAAAAAA1fX2NvbnN0cnVjdG9yAAAAAAAABAAAAAAAAAAJeGxtX3Rva2VuAAAAAAAAEwAAAAAAAAALeWllbGRfdG9rZW4AAAAAEwAAAAAAAAAIZXhjaGFuZ2UAAAATAAAAAAAAAA9mdW5kaW5nX3BhcnRuZXIAAAAAEwAAAAA=",
        "AAAAAAAAADpSZXR1cm5zIHRoZSBjdXJyZW50IHlpZWxkIHRva2VuIGJhbGFuY2UgaGVsZCBieSB0aGUgdmF1bHQuAAAAAAANeWllbGRfYmFsYW5jZQAAAAAAAAAAAAABAAAACw==",
        "AAAAAAAAAXxEZXBvc2l0cyBwcmluY2lwYWwgaW50byB0aGUgdmF1bHQuCgojIEFyZ3VtZW50cwoKKiBgZW52YCAtIEFjY2VzcyB0byB0aGUgU29yb2JhbiBlbnZpcm9ubWVudC4KKiBgYW1vdW50X3hsbWAgLSBBbW91bnQgb2YgcHJpbmNpcGFsIHRva2VuIHRvIGRlcG9zaXQuCgojIEVycm9ycwoKKiBbYENvbnRyYWN0RXJyb3I6OkRlcG9zaXRBbW91bnRNdXN0QmVQb3NpdGl2ZWBdIC0gSWYgYGFtb3VudF94bG0gPD0gMGAuCgojIE5vdGVzCgoqIEF1dGhvcml6YXRpb24gZnJvbSBgRnVuZGluZ1BhcnRuZXJgIGlzIHJlcXVpcmVkLgoqIE5vIHNoYXJlIHRva2VuIGlzIG1pbnRlZCBiZWNhdXNlIHRoaXMgZGVzaWduIHN1cHBvcnRzIGEgc2luZ2xlIGZ1bmRpbmcKcGFydG5lciBvbmx5LgAAAA9kZXBvc2l0X3BhcnRuZXIAAAAAAQAAAAAAAAAKYW1vdW50X3hsbQAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUNvbnRyYWN0RXJyb3IAAAA=",
        "AAAAAAAAACRSZXR1cm5zIHRoZSBmdW5kaW5nIHBhcnRuZXIgYWRkcmVzcy4AAAAPZnVuZGluZ19wYXJ0bmVyAAAAAAAAAAABAAAAEw==",
        "AAAAAAAAADRSZXR1cm5zIHlpZWxkIG93ZWQgYnkgdGhlIGV4Y2hhbmdlIGJ1dCBub3QgeWV0IHBhaWQuAAAAEHlpZWxkX2RlYnRfdXNkdDAAAAAAAAAAAQAAAAs=",
        "AAAAAAAAADFSZXR1cm5zIHByaW5jaXBhbCB0aGF0IGlzIG5vdCBjdXJyZW50bHkgcmVzZXJ2ZWQuAAAAAAAAEmZyZWVfcHJpbmNpcGFsX3hsbQAAAAAAAAAAAAEAAAAL",
        "AAAAAAAAACtSZXR1cm5zIHlpZWxkIGFjdHVhbGx5IHBhaWQgaW50byB0aGUgdmF1bHQuAAAAABVjb2xsZWN0ZWRfeWllbGRfdXNkdDAAAAAAAAAAAAAAAQAAAAs=",
        "AAAAAAAAADVSZXR1cm5zIHRvdGFsIHBhcnRuZXIgcHJpbmNpcGFsIHRyYWNrZWQgYnkgdGhlIHZhdWx0LgAAAAAAABVwYXJ0bmVyX3ByaW5jaXBhbF94bG0AAAAAAAAAAAAAAQAAAAs=",
        "AAAAAAAAAbtXaXRoZHJhd3MgY29sbGVjdGVkIHlpZWxkIGZyb20gdGhlIHZhdWx0LgoKIyBBcmd1bWVudHMKCiogYGVudmAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCiogYHRvYCAtIFJlY2lwaWVudCBvZiB3aXRoZHJhd24geWllbGQuCiogYGFtb3VudF91c2R0MGAgLSBBbW91bnQgb2YgY29sbGVjdGVkIHlpZWxkIHRvIHdpdGhkcmF3LgoKIyBFcnJvcnMKCiogW2BDb250cmFjdEVycm9yOjpZaWVsZEFtb3VudE11c3RCZVBvc2l0aXZlYF0gLSBJZiBgYW1vdW50X3VzZHQwIDw9IDBgLgoqIFtgQ29udHJhY3RFcnJvcjo6SW5zdWZmaWNpZW50Q29sbGVjdGVkWWllbGRgXSAtIElmIGBhbW91bnRfdXNkdDBgCmV4Y2VlZHMgY29sbGVjdGVkIHlpZWxkIGJhbGFuY2UuCgojIE5vdGVzCgoqIEF1dGhvcml6YXRpb24gZnJvbSBgRnVuZGluZ1BhcnRuZXJgIGlzIHJlcXVpcmVkLgAAAAAWd2l0aGRyYXdfcGFydG5lcl95aWVsZAAAAAAAAgAAAAAAAAACdG8AAAAAABMAAAAAAAAADGFtb3VudF91c2R0MAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAC5SZXR1cm5zIHRoZSBsYXN0IHJlc2VydmUtc2V0IHJlZmVyZW5jZSBjcmVkaXQuAAAAAAAXbGFzdF9zZXRfcmVzZXJ2ZV9jcmVkaXQAAAAAAAAAAAEAAAAL",
        "AAAAAAAAADNSZXR1cm5zIHRoZSBsYXRlc3QgcmVjb3JkZWQgeWllbGQgc2V0dGxlbWVudCBlcG9jaC4AAAAAF2xhdGVzdF9zZXR0bGVtZW50X2Vwb2NoAAAAAAAAAAABAAAABg==",
        "AAAAAAAAA7pSZWNvcmRzIHlpZWxkIGRlYnQgYW5kIGFueSBjb25jdXJyZW50IHlpZWxkIHBheW1lbnQgZm9yIGEgc2V0dGxlbWVudAplcG9jaC4KCiMgQXJndW1lbnRzCgoqIGBlbnZgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGBlcG9jaF9pZGAgLSBNb25vdG9uaWNhbGx5IGluY3JlYXNpbmcgc2V0dGxlbWVudCBlcG9jaCBpZGVudGlmaWVyLgoqIGB5aWVsZF9kdWVfdXNkdDBgIC0gQWRkaXRpb25hbCB5aWVsZCBvYmxpZ2F0aW9uIGNyZWF0ZWQgYnkgdGhpcyBlcG9jaC4KKiBgeWllbGRfcGFpZF91c2R0MGAgLSBQb3J0aW9uIG9mIHRvdGFsIG91dHN0YW5kaW5nIHlpZWxkIHBhaWQgbm93LgoqIGByZWZlcmVuY2VfaGFzaGAgLSBPcHRpb25hbCBhdWRpdCByZWZlcmVuY2UgZm9yIHRoZSBvZmYtY2hhaW4Kc2V0dGxlbWVudCBwYWNrYWdlLgoKIyBFcnJvcnMKCiogW2BDb250cmFjdEVycm9yOjpZaWVsZFNldHRsZW1lbnRBbW91bnRzTXVzdEJlTm9uTmVnYXRpdmVgXSAtIElmIGVpdGhlcgp5aWVsZCBhbW91bnQgaXMgbmVnYXRpdmUuCiogW2BDb250cmFjdEVycm9yOjpTZXR0bGVtZW50RXBvY2hNdXN0SW5jcmVhc2VgXSAtIElmIGBlcG9jaF9pZGAgaXMgbm90CmdyZWF0ZXIgdGhhbiB0aGUgcHJldmlvdXNseSByZWNvcmRlZCBlcG9jaC4KKiBbYENvbnRyYWN0RXJyb3I6OllpZWxkUGFpZEV4Y2VlZHNEZWJ0QW5kQ3VycmVudER1ZWBdIC0gSWYgdGhlIHBheW1lbnQKZXhjZWVkcyBwcmlvciBkZWJ0IHBsdXMgY3VycmVudCBlcG9jaCBkdWUuCgojIE5vdGVzCgoqIEF1dGhvcml6YXRpb24gZnJvbSBgRXhjaGFuZ2VgIGlzIHJlcXVpcmVkLgoqIGB5aWVsZF9wYWlkX3VzZHQwYCBtdXN0IGJlIGJhY2tlZCBieSBhbiBhY3R1YWwgdG9rZW4gdHJhbnNmZXIuCiogVW5wYWlkIHlpZWxkIHJlbWFpbnMgZGVidCBhbmQgaXMgbm90IGFkZGVkIHRvIGNvbGxlY3RlZCB5aWVsZC4AAAAAABdyZWNvcmRfeWllbGRfc2V0dGxlbWVudAAAAAAEAAAAAAAAAAhlcG9jaF9pZAAAAAYAAAAAAAAAD3lpZWxkX2R1ZV91c2R0MAAAAAALAAAAAAAAABB5aWVsZF9wYWlkX3VzZHQwAAAACwAAAAAAAAAOcmVmZXJlbmNlX2hhc2gAAAAAA+gAAAPuAAAAIAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAADUNvbnRyYWN0RXJyb3IAAAA=",
        "AAAAAAAAAEJSZXR1cm5zIHRoZSBtb3N0IHJlY2VudCBvcHRpb25hbCB5aWVsZC1zZXR0bGVtZW50IGF1ZGl0IHJlZmVyZW5jZS4AAAAAABlsYXN0X3lpZWxkX3JlZmVyZW5jZV9oYXNoAAAAAAAAAAAAAAEAAAPoAAAD7gAAACA=",
        "AAAAAAAAAD1SZXR1cm5zIHByaW5jaXBhbCBjdXJyZW50bHkgcmVzZXJ2ZWQgZm9yIGV4Y2hhbmdlIGNvbGxhdGVyYWwuAAAAAAAAGXJlc2VydmVkX2Zvcl9leGNoYW5nZV94bG0AAAAAAAAAAAAAAQAAAAs=",
        "AAAAAAAAAkhXaXRoZHJhd3MgZnJlZSBwcmluY2lwYWwgZnJvbSB0aGUgdmF1bHQuCgojIEFyZ3VtZW50cwoKKiBgZW52YCAtIEFjY2VzcyB0byB0aGUgU29yb2JhbiBlbnZpcm9ubWVudC4KKiBgdG9gIC0gUmVjaXBpZW50IG9mIHRoZSB3aXRoZHJhd24gcHJpbmNpcGFsLgoqIGBhbW91bnRfeGxtYCAtIEFtb3VudCBvZiBmcmVlIHByaW5jaXBhbCB0byB3aXRoZHJhdy4KCiMgRXJyb3JzCgoqIFtgQ29udHJhY3RFcnJvcjo6V2l0aGRyYXdBbW91bnRNdXN0QmVQb3NpdGl2ZWBdIC0gSWYgYGFtb3VudF94bG0gPD0gMGAuCiogW2BDb250cmFjdEVycm9yOjpJbnN1ZmZpY2llbnRGcmVlUHJpbmNpcGFsYF0gLSBJZiBgYW1vdW50X3hsbWAKZXhjZWVkcyB0aGUgdW5yZXNlcnZlZCBwcmluY2lwYWwgYmFsYW5jZS4KCiMgTm90ZXMKCiogQXV0aG9yaXphdGlvbiBmcm9tIGJvdGggYEV4Y2hhbmdlYCBhbmQgYEZ1bmRpbmdQYXJ0bmVyYCBpcyByZXF1aXJlZC4KKiBSZXNlcnZlZCBwcmluY2lwYWwgcmVtYWlucyBlbmN1bWJlcmVkIGJlaGluZCBleGNoYW5nZSBjcmVkaXQgYW5kIGlzCm5ldmVyIHdpdGhkcmF3YWJsZSB0aHJvdWdoIHRoaXMgbWV0aG9kLgAAABp3aXRoZHJhd19wYXJ0bmVyX3ByaW5jaXBhbAAAAAAAAgAAAAAAAAACdG8AAAAAABMAAAAAAAAACmFtb3VudF94bG0AAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAADlSZXR1cm5zIHRoZSBtb3N0IHJlY2VudCBvcHRpb25hbCByZXNlcnZlIGF1ZGl0IHJlZmVyZW5jZS4AAAAAAAAbbGFzdF9yZXNlcnZlX3JlZmVyZW5jZV9oYXNoAAAAAAAAAAABAAAD6AAAA+4AAAAg",
        "AAAAAAAAACtSZXR1cm5zIHRoZSBsYXN0IHJlc2VydmUtc2V0IGV4Y2hhbmdlIHJhdGUuAAAAAB5sYXN0X3NldF9yZXNlcnZlX2V4Y2hhbmdlX3JhdGUAAAAAAAAAAAABAAAACw==",
        "AAAABAAAAAAAAAAAAAAAEFVwZ3JhZGVhYmxlRXJyb3IAAAABAAAAQVdoZW4gbWlncmF0aW9uIGlzIGF0dGVtcHRlZCBidXQgbm90IGFsbG93ZWQgZHVlIHRvIHVwZ3JhZGUgc3RhdGUuAAAAAAAAE01pZ3JhdGlvbk5vdEFsbG93ZWQAAAAETA==",
        "AAAABQAAACpFdmVudCBlbWl0dGVkIHdoZW4gdGhlIG1lcmtsZSByb290IGlzIHNldC4AAAAAAAAAAAAHU2V0Um9vdAAAAAABAAAACHNldF9yb290AAAAAQAAAAAAAAAEcm9vdAAAAA4AAAAAAAAAAg==",
        "AAAABQAAACdFdmVudCBlbWl0dGVkIHdoZW4gYW4gaW5kZXggaXMgY2xhaW1lZC4AAAAAAAAAAApTZXRDbGFpbWVkAAAAAAABAAAAC3NldF9jbGFpbWVkAAAAAAEAAAAAAAAABWluZGV4AAAAAAAAAAAAAAAAAAAC",
        "AAAABAAAAAAAAAAAAAAAFk1lcmtsZURpc3RyaWJ1dG9yRXJyb3IAAAAAAAMAAAAbVGhlIG1lcmtsZSByb290IGlzIG5vdCBzZXQuAAAAAApSb290Tm90U2V0AAAAAAUUAAAAJ1RoZSBwcm92aWRlZCBpbmRleCB3YXMgYWxyZWFkeSBjbGFpbWVkLgAAAAATSW5kZXhBbHJlYWR5Q2xhaW1lZAAAAAUVAAAAFVRoZSBwcm9vZiBpcyBpbnZhbGlkLgAAAAAAAAxJbnZhbGlkUHJvb2YAAAUW",
        "AAAAAgAAAD1TdG9yYWdlIGtleXMgZm9yIHRoZSBkYXRhIGFzc29jaWF0ZWQgd2l0aCBgTWVya2xlRGlzdHJpYnV0b3JgAAAAAAAAAAAAABtNZXJrbGVEaXN0cmlidXRvclN0b3JhZ2VLZXkAAAAAAgAAAAAAAAAoVGhlIE1lcmtsZSByb290IG9mIHRoZSBkaXN0cmlidXRpb24gdHJlZQAAAARSb290AAAAAQAAACNNYXBzIGFuIGluZGV4IHRvIGl0cyBjbGFpbWVkIHN0YXR1cwAAAAAHQ2xhaW1lZAAAAAABAAAABA==",
        "AAAAAgAAAAAAAAAAAAAACFJvdW5kaW5nAAAAAgAAAAAAAAAAAAAABUZsb29yAAAAAAAAAAAAAAAAAAAEQ2VpbA==",
        "AAAABAAAAAAAAAAAAAAAFlNvcm9iYW5GaXhlZFBvaW50RXJyb3IAAAAAAAMAAAAyVGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgZGVub21pbmF0b3IgaXMgMC4AAAAAAA9aZXJvRGVub21pbmF0b3IAAAAF3AAAADlUaGUgb3BlcmF0aW9uIGZhaWxlZCBiZWNhdXNlIGEgcGhhbnRvbSBvdmVyZmxvdyBvY2N1cnJlZC4AAAAAAAAPUGhhbnRvbU92ZXJmbG93AAAABd0AAAA9VGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgcmVzdWx0IGRvZXMgbm90IGZpdCBpbiBTZWxmLgAAAAAAAA5SZXN1bHRPdmVyZmxvdwAAAAAF3g==",
        "AAAABAAAAAAAAAAAAAAAC0NyeXB0b0Vycm9yAAAAAAMAAAApVGhlIG1lcmtsZSBwcm9vZiBsZW5ndGggaXMgb3V0IG9mIGJvdW5kcy4AAAAAAAAWTWVya2xlUHJvb2ZPdXRPZkJvdW5kcwAAAAAFeAAAACdUaGUgaW5kZXggb2YgdGhlIGxlYWYgaXMgb3V0IG9mIGJvdW5kcy4AAAAAFk1lcmtsZUluZGV4T3V0T2ZCb3VuZHMAAAAABXkAAAAYTm8gZGF0YSBpbiBoYXNoZXIgc3RhdGUuAAAAEEhhc2hlckVtcHR5U3RhdGUAAAV6",
        "AAAABQAAACpFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGNvbnRyYWN0IGlzIHBhdXNlZC4AAAAAAAAAAAAGUGF1c2VkAAAAAAABAAAABnBhdXNlZAAAAAAAAAAAAAI=",
        "AAAABQAAACxFdmVudCBlbWl0dGVkIHdoZW4gdGhlIGNvbnRyYWN0IGlzIHVucGF1c2VkLgAAAAAAAAAIVW5wYXVzZWQAAAABAAAACHVucGF1c2VkAAAAAAAAAAI=",
        "AAAABAAAAAAAAAAAAAAADVBhdXNhYmxlRXJyb3IAAAAAAAACAAAANFRoZSBvcGVyYXRpb24gZmFpbGVkIGJlY2F1c2UgdGhlIGNvbnRyYWN0IGlzIHBhdXNlZC4AAAANRW5mb3JjZWRQYXVzZQAAAAAAA+gAAAA4VGhlIG9wZXJhdGlvbiBmYWlsZWQgYmVjYXVzZSB0aGUgY29udHJhY3QgaXMgbm90IHBhdXNlZC4AAAANRXhwZWN0ZWRQYXVzZQAAAAAAA+k=",
        "AAAAAgAAACJTdG9yYWdlIGtleSBmb3IgdGhlIHBhdXNhYmxlIHN0YXRlAAAAAAAAAAAAElBhdXNhYmxlU3RvcmFnZUtleQAAAAAAAQAAAAAAAAAySW5kaWNhdGVzIHdoZXRoZXIgdGhlIGNvbnRyYWN0IGlzIGluIHBhdXNlZCBzdGF0ZS4AAAAAAAZQYXVzZWQAAA==" ]),
      options
    )
  }
  public readonly fromJSON = {
    upgrade: this.txFromJSON<null>,
        exchange: this.txFromJSON<string>,
        pay_yield: this.txFromJSON<Result<void>>,
        xlm_token: this.txFromJSON<string>,
        set_reserve: this.txFromJSON<Result<void>>,
        xlm_balance: this.txFromJSON<i128>,
        yield_token: this.txFromJSON<string>,
        yield_balance: this.txFromJSON<i128>,
        deposit_partner: this.txFromJSON<Result<void>>,
        funding_partner: this.txFromJSON<string>,
        yield_debt_usdt0: this.txFromJSON<i128>,
        free_principal_xlm: this.txFromJSON<i128>,
        collected_yield_usdt0: this.txFromJSON<i128>,
        partner_principal_xlm: this.txFromJSON<i128>,
        withdraw_partner_yield: this.txFromJSON<Result<void>>,
        last_set_reserve_credit: this.txFromJSON<i128>,
        latest_settlement_epoch: this.txFromJSON<u64>,
        record_yield_settlement: this.txFromJSON<Result<void>>,
        last_yield_reference_hash: this.txFromJSON<Option<Buffer>>,
        reserved_for_exchange_xlm: this.txFromJSON<i128>,
        withdraw_partner_principal: this.txFromJSON<Result<void>>,
        last_reserve_reference_hash: this.txFromJSON<Option<Buffer>>,
        last_set_reserve_exchange_rate: this.txFromJSON<i128>
  }
}