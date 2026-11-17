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
 * Errors returned by the rollup contract.
 */
export const ContractError = {
  1: {message:"DepositAmountMustBePositive"},
  11: {message:"NewBlockHashEmpty"},
  12: {message:"OldBlockHashMismatch"},
  13: {message:"BlockHashUnchanged"},
  15: {message:"ArrayLengthExceedsLimit"},
  16: {message:"WithdrawalSumMismatch"},
  17: {message:"InsufficientBalance"},
  18: {message:"WithdrawalAmountMustBeNonNegative"},
  19: {message:"FeesMustBeNonNegative"},
  20: {message:"WithdrawalAddressIsContract"},
  31: {message:"NoWithdrawalAllowance"},
  41: {message:"NoFeesToCollect"},
  51: {message:"CannotRecoverCollateral"},
  52: {message:"RecoverAmountMustBePositive"},
  61: {message:"RenounceOwnershipDisabled"},
  62: {message:"Unauthorized"},
  63: {message:"OwnerIsContract"},
  71: {message:"ArithmeticOverflow"}
}


/**
 * Withdrawal allowance credited to one recipient in a rollup block.
 */
export interface WithdrawalCredit {
  amount: i128;
  recipient: string;
}

export const RoleTransferError = {
  2200: {message:"NoPendingTransfer"},
  2201: {message:"InvalidLiveUntilLedger"},
  2202: {message:"InvalidPendingAccount"},
  2203: {message:"TransferExpired"}
}

export const OwnableError = {
  2100: {message:"OwnerNotSet"},
  2101: {message:"TransferInProgress"},
  2102: {message:"OwnerAlreadySet"}
}



export interface Client {
  /**
   * Construct and simulate a fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns currently accrued protocol fees.
   */
  fees: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a owner transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current owner.
   */
  owner: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a rollup transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Advances the rollup state to a new block hash and posts new withdrawable
   * balances plus fees.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `old_block_hash` - Expected current block hash.
   * * `new_block_hash` - New block hash to commit.
   * * `new_withdrawal_credits` - Recipients and amounts of the newly posted
   * withdrawal allowances.
   * * `new_withdrawal_sum` - Sum of all new withdrawal amounts.
   * * `new_fees` - Fees accrued in the new rollup block.
   * 
   * # Errors
   * 
   * * [`ContractError::NewBlockHashEmpty`] - If `new_block_hash` is zero.
   * * [`ContractError::OldBlockHashMismatch`] - If `old_block_hash` does not
   * match the currently stored block hash.
   * * [`ContractError::BlockHashUnchanged`] - If the new block hash is equal
   * to the old block hash.
   * * [`ContractError::ArrayLengthExceedsLimit`] - If more than 100
   * withdrawal credits are posted.
   * * [`ContractError::WithdrawalSumMismatch`] - If the provided
   * `new_withdrawal_sum` does not match the calculated total.
   * * [`ContractError::InsufficientBalance`] - If the contract balan
   */
  rollup: ({old_block_hash, new_block_hash, new_withdrawal_credits, new_withdrawal_sum, new_fees}: {old_block_hash: Buffer, new_block_hash: Buffer, new_withdrawal_credits: Array<WithdrawalCredit>, new_withdrawal_sum: i128, new_fees: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits collateral into the rollup.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `user` - Address providing the deposit.
   * * `amount` - Amount of collateral token to deposit.
   * 
   * # Errors
   * 
   * * [`ContractError::DepositAmountMustBePositive`] - If `amount <= 0`.
   * 
   * # Notes
   * 
   * * Authorization from `user` is required.
   */
  deposit: ({user, amount}: {user: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a recover transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Recovers non-collateral tokens that were sent to the contract.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `token_address` - Token contract to recover.
   * * `to` - Recipient of the recovered tokens.
   * * `amount` - Amount to recover.
   * 
   * # Errors
   * 
   * * [`ContractError::CannotRecoverCollateral`] - If `token_address` is the
   * configured collateral token.
   * * [`ContractError::RecoverAmountMustBePositive`] - If `amount <= 0`.
   * 
   * # Notes
   * 
   * * Owner authorization is required.
   */
  recover: ({token_address, to, amount}: {token_address: string, to: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Replaces the contract Wasm with `new_wasm_hash`. Requires authorization
   * from the owner.
   */
  upgrade: ({new_wasm_hash, operator}: {new_wasm_hash: Buffer, operator: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraws the full posted allowance for `user`.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `user` - Address withdrawing its allowance.
   * 
   * # Errors
   * 
   * * [`ContractError::NoWithdrawalAllowance`] - If no positive allowance is
   * available for `user`.
   * 
   * # Notes
   * 
   * * Authorization from `user` is required.
   * * This method withdraws the full stored allowance and resets it to zero.
   */
  withdraw: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a block_height transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the height of the latest committed rollup block.
   * 
   * The constructor sets this to zero, and every [`Self::rollup`] increments
   * it by one, so the value counts the blocks committed so far.
   */
  block_height: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a collect_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Collects all currently accrued protocol fees.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `to` - Recipient of collected fees.
   * 
   * # Errors
   * 
   * * [`ContractError::NoFeesToCollect`] - If no positive fee balance is
   * available.
   * 
   * # Notes
   * 
   * * Owner authorization is required.
   * * Collected fees are removed from both `Fees` and `TotalWithdrawable`.
   */
  collect_fees: ({to}: {to: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accept_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Accepts a pending ownership transfer.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   */
  accept_ownership: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a latest_block_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the latest committed rollup block hash.
   */
  latest_block_hash: (options?: MethodOptions) => Promise<AssembledTransaction<Buffer>>

  /**
   * Construct and simulate a collateral_balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current collateral-token balance held by the contract.
   */
  collateral_balance: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a renounce_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Rejects ownership renunciation.
   * 
   * # Errors
   * 
   * * [`ContractError::RenounceOwnershipDisabled`] - Always returned.
   */
  renounce_ownership: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a total_withdrawable transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the total amount currently reserved for user withdrawals plus
   * accrued fees.
   */
  total_withdrawable: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initiates a 2-step ownership transfer.
   * 
   * The proposed new owner must later call [`Self::accept_ownership`] to
   * complete the transfer.
   * 
   * # Arguments
   * 
   * * `env` - Access to the Soroban environment.
   * * `new_owner` - Proposed new owner.
   * * `live_until_ledger` - Ledger until which the pending transfer can be
   * accepted.
   * 
   * # Notes
   * 
   * * Authorization is enforced internally by the ownable library.
   */
  transfer_ownership: ({new_owner, live_until_ledger}: {new_owner: string, live_until_ledger: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a withdrawal_allowances transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the current withdrawal allowance for `user`.
   */
  withdrawal_allowances: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {collateral_token, owner}: {collateral_token: string, owner: string},
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
    return ContractClient.deploy({collateral_token, owner}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAChSZXR1cm5zIGN1cnJlbnRseSBhY2NydWVkIHByb3RvY29sIGZlZXMuAAAABGZlZXMAAAAAAAAAAQAAAAs=",
        "AAAAAAAAABpSZXR1cm5zIHRoZSBjdXJyZW50IG93bmVyLgAAAAAABW93bmVyAAAAAAAAAAAAAAEAAAPoAAAAEw==",
        "AAAABAAAACdFcnJvcnMgcmV0dXJuZWQgYnkgdGhlIHJvbGx1cCBjb250cmFjdC4AAAAAAAAAAA1Db250cmFjdEVycm9yAAAAAAAAEgAAAAAAAAAbRGVwb3NpdEFtb3VudE11c3RCZVBvc2l0aXZlAAAAAAEAAAAAAAAAEU5ld0Jsb2NrSGFzaEVtcHR5AAAAAAAACwAAAAAAAAAUT2xkQmxvY2tIYXNoTWlzbWF0Y2gAAAAMAAAAAAAAABJCbG9ja0hhc2hVbmNoYW5nZWQAAAAAAA0AAAAAAAAAF0FycmF5TGVuZ3RoRXhjZWVkc0xpbWl0AAAAAA8AAAAAAAAAFVdpdGhkcmF3YWxTdW1NaXNtYXRjaAAAAAAAABAAAAAAAAAAE0luc3VmZmljaWVudEJhbGFuY2UAAAAAEQAAAAAAAAAhV2l0aGRyYXdhbEFtb3VudE11c3RCZU5vbk5lZ2F0aXZlAAAAAAAAEgAAAAAAAAAVRmVlc011c3RCZU5vbk5lZ2F0aXZlAAAAAAAAEwAAAAAAAAAbV2l0aGRyYXdhbEFkZHJlc3NJc0NvbnRyYWN0AAAAABQAAAAAAAAAFU5vV2l0aGRyYXdhbEFsbG93YW5jZQAAAAAAAB8AAAAAAAAAD05vRmVlc1RvQ29sbGVjdAAAAAApAAAAAAAAABdDYW5ub3RSZWNvdmVyQ29sbGF0ZXJhbAAAAAAzAAAAAAAAABtSZWNvdmVyQW1vdW50TXVzdEJlUG9zaXRpdmUAAAAANAAAAAAAAAAZUmVub3VuY2VPd25lcnNoaXBEaXNhYmxlZAAAAAAAAD0AAAAAAAAADFVuYXV0aG9yaXplZAAAAD4AAAAAAAAAD093bmVySXNDb250cmFjdAAAAAA/AAAAAAAAABJBcml0aG1ldGljT3ZlcmZsb3cAAAAAAEc=",
        "AAAAAAAABABBZHZhbmNlcyB0aGUgcm9sbHVwIHN0YXRlIHRvIGEgbmV3IGJsb2NrIGhhc2ggYW5kIHBvc3RzIG5ldyB3aXRoZHJhd2FibGUKYmFsYW5jZXMgcGx1cyBmZWVzLgoKIyBBcmd1bWVudHMKCiogYGVudmAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCiogYG9sZF9ibG9ja19oYXNoYCAtIEV4cGVjdGVkIGN1cnJlbnQgYmxvY2sgaGFzaC4KKiBgbmV3X2Jsb2NrX2hhc2hgIC0gTmV3IGJsb2NrIGhhc2ggdG8gY29tbWl0LgoqIGBuZXdfd2l0aGRyYXdhbF9jcmVkaXRzYCAtIFJlY2lwaWVudHMgYW5kIGFtb3VudHMgb2YgdGhlIG5ld2x5IHBvc3RlZAp3aXRoZHJhd2FsIGFsbG93YW5jZXMuCiogYG5ld193aXRoZHJhd2FsX3N1bWAgLSBTdW0gb2YgYWxsIG5ldyB3aXRoZHJhd2FsIGFtb3VudHMuCiogYG5ld19mZWVzYCAtIEZlZXMgYWNjcnVlZCBpbiB0aGUgbmV3IHJvbGx1cCBibG9jay4KCiMgRXJyb3JzCgoqIFtgQ29udHJhY3RFcnJvcjo6TmV3QmxvY2tIYXNoRW1wdHlgXSAtIElmIGBuZXdfYmxvY2tfaGFzaGAgaXMgemVyby4KKiBbYENvbnRyYWN0RXJyb3I6Ok9sZEJsb2NrSGFzaE1pc21hdGNoYF0gLSBJZiBgb2xkX2Jsb2NrX2hhc2hgIGRvZXMgbm90Cm1hdGNoIHRoZSBjdXJyZW50bHkgc3RvcmVkIGJsb2NrIGhhc2guCiogW2BDb250cmFjdEVycm9yOjpCbG9ja0hhc2hVbmNoYW5nZWRgXSAtIElmIHRoZSBuZXcgYmxvY2sgaGFzaCBpcyBlcXVhbAp0byB0aGUgb2xkIGJsb2NrIGhhc2guCiogW2BDb250cmFjdEVycm9yOjpBcnJheUxlbmd0aEV4Y2VlZHNMaW1pdGBdIC0gSWYgbW9yZSB0aGFuIDEwMAp3aXRoZHJhd2FsIGNyZWRpdHMgYXJlIHBvc3RlZC4KKiBbYENvbnRyYWN0RXJyb3I6OldpdGhkcmF3YWxTdW1NaXNtYXRjaGBdIC0gSWYgdGhlIHByb3ZpZGVkCmBuZXdfd2l0aGRyYXdhbF9zdW1gIGRvZXMgbm90IG1hdGNoIHRoZSBjYWxjdWxhdGVkIHRvdGFsLgoqIFtgQ29udHJhY3RFcnJvcjo6SW5zdWZmaWNpZW50QmFsYW5jZWBdIC0gSWYgdGhlIGNvbnRyYWN0IGJhbGFuAAAABnJvbGx1cAAAAAAABQAAAAAAAAAOb2xkX2Jsb2NrX2hhc2gAAAAAA+4AAAAgAAAAAAAAAA5uZXdfYmxvY2tfaGFzaAAAAAAD7gAAACAAAAAAAAAAFm5ld193aXRoZHJhd2FsX2NyZWRpdHMAAAAAA+oAAAfQAAAAEFdpdGhkcmF3YWxDcmVkaXQAAAAAAAAAEm5ld193aXRoZHJhd2FsX3N1bQAAAAAACwAAAAAAAAAIbmV3X2ZlZXMAAAALAAAAAQAAA+kAAAACAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAUBEZXBvc2l0cyBjb2xsYXRlcmFsIGludG8gdGhlIHJvbGx1cC4KCiMgQXJndW1lbnRzCgoqIGBlbnZgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGB1c2VyYCAtIEFkZHJlc3MgcHJvdmlkaW5nIHRoZSBkZXBvc2l0LgoqIGBhbW91bnRgIC0gQW1vdW50IG9mIGNvbGxhdGVyYWwgdG9rZW4gdG8gZGVwb3NpdC4KCiMgRXJyb3JzCgoqIFtgQ29udHJhY3RFcnJvcjo6RGVwb3NpdEFtb3VudE11c3RCZVBvc2l0aXZlYF0gLSBJZiBgYW1vdW50IDw9IDBgLgoKIyBOb3RlcwoKKiBBdXRob3JpemF0aW9uIGZyb20gYHVzZXJgIGlzIHJlcXVpcmVkLgAAAAdkZXBvc2l0AAAAAAIAAAAAAAAABHVzZXIAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAAAIAAAfQAAAADUNvbnRyYWN0RXJyb3IAAAA=",
        "AAAAAAAAAddSZWNvdmVycyBub24tY29sbGF0ZXJhbCB0b2tlbnMgdGhhdCB3ZXJlIHNlbnQgdG8gdGhlIGNvbnRyYWN0LgoKIyBBcmd1bWVudHMKCiogYGVudmAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCiogYHRva2VuX2FkZHJlc3NgIC0gVG9rZW4gY29udHJhY3QgdG8gcmVjb3Zlci4KKiBgdG9gIC0gUmVjaXBpZW50IG9mIHRoZSByZWNvdmVyZWQgdG9rZW5zLgoqIGBhbW91bnRgIC0gQW1vdW50IHRvIHJlY292ZXIuCgojIEVycm9ycwoKKiBbYENvbnRyYWN0RXJyb3I6OkNhbm5vdFJlY292ZXJDb2xsYXRlcmFsYF0gLSBJZiBgdG9rZW5fYWRkcmVzc2AgaXMgdGhlCmNvbmZpZ3VyZWQgY29sbGF0ZXJhbCB0b2tlbi4KKiBbYENvbnRyYWN0RXJyb3I6OlJlY292ZXJBbW91bnRNdXN0QmVQb3NpdGl2ZWBdIC0gSWYgYGFtb3VudCA8PSAwYC4KCiMgTm90ZXMKCiogT3duZXIgYXV0aG9yaXphdGlvbiBpcyByZXF1aXJlZC4AAAAAB3JlY292ZXIAAAAAAwAAAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAAAIAAAfQAAAADUNvbnRyYWN0RXJyb3IAAAA=",
        "AAAAAAAAAFdSZXBsYWNlcyB0aGUgY29udHJhY3QgV2FzbSB3aXRoIGBuZXdfd2FzbV9oYXNoYC4gUmVxdWlyZXMgYXV0aG9yaXphdGlvbgpmcm9tIHRoZSBvd25lci4AAAAAB3VwZ3JhZGUAAAAAAgAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAAAAAAhvcGVyYXRvcgAAABMAAAAA",
        "AAAAAAAAAX5XaXRoZHJhd3MgdGhlIGZ1bGwgcG9zdGVkIGFsbG93YW5jZSBmb3IgYHVzZXJgLgoKIyBBcmd1bWVudHMKCiogYGVudmAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCiogYHVzZXJgIC0gQWRkcmVzcyB3aXRoZHJhd2luZyBpdHMgYWxsb3dhbmNlLgoKIyBFcnJvcnMKCiogW2BDb250cmFjdEVycm9yOjpOb1dpdGhkcmF3YWxBbGxvd2FuY2VgXSAtIElmIG5vIHBvc2l0aXZlIGFsbG93YW5jZSBpcwphdmFpbGFibGUgZm9yIGB1c2VyYC4KCiMgTm90ZXMKCiogQXV0aG9yaXphdGlvbiBmcm9tIGB1c2VyYCBpcyByZXF1aXJlZC4KKiBUaGlzIG1ldGhvZCB3aXRoZHJhd3MgdGhlIGZ1bGwgc3RvcmVkIGFsbG93YW5jZSBhbmQgcmVzZXRzIGl0IHRvIHplcm8uAAAAAAAId2l0aGRyYXcAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAAAgAAB9AAAAANQ29udHJhY3RFcnJvcgAAAA==",
        "AAAAAQAAAEFXaXRoZHJhd2FsIGFsbG93YW5jZSBjcmVkaXRlZCB0byBvbmUgcmVjaXBpZW50IGluIGEgcm9sbHVwIGJsb2NrLgAAAAAAAAAAAAAQV2l0aGRyYXdhbENyZWRpdAAAAAIAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAJcmVjaXBpZW50AAAAAAAAEw==",
        "AAAAAAAAAL5SZXR1cm5zIHRoZSBoZWlnaHQgb2YgdGhlIGxhdGVzdCBjb21taXR0ZWQgcm9sbHVwIGJsb2NrLgoKVGhlIGNvbnN0cnVjdG9yIHNldHMgdGhpcyB0byB6ZXJvLCBhbmQgZXZlcnkgW2BTZWxmOjpyb2xsdXBgXSBpbmNyZW1lbnRzCml0IGJ5IG9uZSwgc28gdGhlIHZhbHVlIGNvdW50cyB0aGUgYmxvY2tzIGNvbW1pdHRlZCBzbyBmYXIuAAAAAAAMYmxvY2tfaGVpZ2h0AAAAAAAAAAEAAAAE",
        "AAAAAAAAAV1Db2xsZWN0cyBhbGwgY3VycmVudGx5IGFjY3J1ZWQgcHJvdG9jb2wgZmVlcy4KCiMgQXJndW1lbnRzCgoqIGBlbnZgIC0gQWNjZXNzIHRvIHRoZSBTb3JvYmFuIGVudmlyb25tZW50LgoqIGB0b2AgLSBSZWNpcGllbnQgb2YgY29sbGVjdGVkIGZlZXMuCgojIEVycm9ycwoKKiBbYENvbnRyYWN0RXJyb3I6Ok5vRmVlc1RvQ29sbGVjdGBdIC0gSWYgbm8gcG9zaXRpdmUgZmVlIGJhbGFuY2UgaXMKYXZhaWxhYmxlLgoKIyBOb3RlcwoKKiBPd25lciBhdXRob3JpemF0aW9uIGlzIHJlcXVpcmVkLgoqIENvbGxlY3RlZCBmZWVzIGFyZSByZW1vdmVkIGZyb20gYm90aCBgRmVlc2AgYW5kIGBUb3RhbFdpdGhkcmF3YWJsZWAuAAAAAAAADGNvbGxlY3RfZmVlcwAAAAEAAAAAAAAAAnRvAAAAAAATAAAAAQAAA+kAAAACAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAgZJbml0aWFsaXplcyB0aGUgcm9sbHVwIGNvbnRyYWN0LgoKIyBBcmd1bWVudHMKCiogYGVudmAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCiogYGNvbGxhdGVyYWxfdG9rZW5gIC0gVG9rZW4gY29udHJhY3QgdXNlZCBmb3IgZGVwb3NpdHMsIHdpdGhkcmF3YWxzLAphbmQgZmVlcy4KKiBgb3duZXJgIC0gVXBncmFkZSBhbmQgYWRtaW4gYXV0aG9yaXR5IGZvciByb2xsdXAgdXBkYXRlcyBhbmQgZmVlCmNvbGxlY3Rpb24uCgojIEVycm9ycwoKKiBbYENvbnRyYWN0RXJyb3I6Ok93bmVySXNDb250cmFjdGBdIC0gSWYgYG93bmVyYCBpcyB0aGUgYWRkcmVzcyBvZgp0aGlzIGNvbnRyYWN0LgoKIyBOb3RlcwoKKiBUaGUgbGF0ZXN0IGJsb2NrIGhhc2ggaXMgaW5pdGlhbGl6ZWQgdG8gdGhlIHplcm8gaGFzaC4KKiBUaGUgYmxvY2sgaGVpZ2h0IGlzIGluaXRpYWxpemVkIHRvIHplcm8uCiogRmVlcyBhbmQgdG90YWwgd2l0aGRyYXdhYmxlIGJhbGFuY2VzIGFyZSBpbml0aWFsaXplZCB0byB6ZXJvLgAAAAAADV9fY29uc3RydWN0b3IAAAAAAAACAAAAAAAAABBjb2xsYXRlcmFsX3Rva2VuAAAAEwAAAAAAAAAFb3duZXIAAAAAAAATAAAAAA==",
        "AAAAAAAAAGBBY2NlcHRzIGEgcGVuZGluZyBvd25lcnNoaXAgdHJhbnNmZXIuCgojIEFyZ3VtZW50cwoKKiBgZW52YCAtIEFjY2VzcyB0byB0aGUgU29yb2JhbiBlbnZpcm9ubWVudC4AAAAQYWNjZXB0X293bmVyc2hpcAAAAAAAAAAA",
        "AAAAAAAAAC9SZXR1cm5zIHRoZSBsYXRlc3QgY29tbWl0dGVkIHJvbGx1cCBibG9jayBoYXNoLgAAAAARbGF0ZXN0X2Jsb2NrX2hhc2gAAAAAAAAAAAAAAQAAA+4AAAAg",
        "AAAAAAAAAEJSZXR1cm5zIHRoZSBjdXJyZW50IGNvbGxhdGVyYWwtdG9rZW4gYmFsYW5jZSBoZWxkIGJ5IHRoZSBjb250cmFjdC4AAAAAABJjb2xsYXRlcmFsX2JhbGFuY2UAAAAAAAAAAAABAAAACw==",
        "AAAAAAAAAGxSZWplY3RzIG93bmVyc2hpcCByZW51bmNpYXRpb24uCgojIEVycm9ycwoKKiBbYENvbnRyYWN0RXJyb3I6OlJlbm91bmNlT3duZXJzaGlwRGlzYWJsZWRgXSAtIEFsd2F5cyByZXR1cm5lZC4AAAAScmVub3VuY2Vfb3duZXJzaGlwAAAAAAAAAAAAAQAAA+kAAAACAAAH0AAAAA1Db250cmFjdEVycm9yAAAA",
        "AAAAAAAAAFNSZXR1cm5zIHRoZSB0b3RhbCBhbW91bnQgY3VycmVudGx5IHJlc2VydmVkIGZvciB1c2VyIHdpdGhkcmF3YWxzIHBsdXMKYWNjcnVlZCBmZWVzLgAAAAASdG90YWxfd2l0aGRyYXdhYmxlAAAAAAAAAAAAAQAAAAs=",
        "AAAAAAAAAXxJbml0aWF0ZXMgYSAyLXN0ZXAgb3duZXJzaGlwIHRyYW5zZmVyLgoKVGhlIHByb3Bvc2VkIG5ldyBvd25lciBtdXN0IGxhdGVyIGNhbGwgW2BTZWxmOjphY2NlcHRfb3duZXJzaGlwYF0gdG8KY29tcGxldGUgdGhlIHRyYW5zZmVyLgoKIyBBcmd1bWVudHMKCiogYGVudmAgLSBBY2Nlc3MgdG8gdGhlIFNvcm9iYW4gZW52aXJvbm1lbnQuCiogYG5ld19vd25lcmAgLSBQcm9wb3NlZCBuZXcgb3duZXIuCiogYGxpdmVfdW50aWxfbGVkZ2VyYCAtIExlZGdlciB1bnRpbCB3aGljaCB0aGUgcGVuZGluZyB0cmFuc2ZlciBjYW4gYmUKYWNjZXB0ZWQuCgojIE5vdGVzCgoqIEF1dGhvcml6YXRpb24gaXMgZW5mb3JjZWQgaW50ZXJuYWxseSBieSB0aGUgb3duYWJsZSBsaWJyYXJ5LgAAABJ0cmFuc2Zlcl9vd25lcnNoaXAAAAAAAAIAAAAAAAAACW5ld19vd25lcgAAAAAAABMAAAAAAAAAEWxpdmVfdW50aWxfbGVkZ2VyAAAAAAAABAAAAAA=",
        "AAAAAAAAADRSZXR1cm5zIHRoZSBjdXJyZW50IHdpdGhkcmF3YWwgYWxsb3dhbmNlIGZvciBgdXNlcmAuAAAAFXdpdGhkcmF3YWxfYWxsb3dhbmNlcwAAAAAAAAEAAAAAAAAABHVzZXIAAAATAAAAAQAAAAs=",
        "AAAABAAAAAAAAAAAAAAAEVJvbGVUcmFuc2ZlckVycm9yAAAAAAAABAAAAAAAAAARTm9QZW5kaW5nVHJhbnNmZXIAAAAAAAiYAAAAAAAAABZJbnZhbGlkTGl2ZVVudGlsTGVkZ2VyAAAAAAiZAAAAAAAAABVJbnZhbGlkUGVuZGluZ0FjY291bnQAAAAAAAiaAAAAAAAAAA9UcmFuc2ZlckV4cGlyZWQAAAAImw==",
        "AAAABAAAAAAAAAAAAAAADE93bmFibGVFcnJvcgAAAAMAAAAAAAAAC093bmVyTm90U2V0AAAACDQAAAAAAAAAElRyYW5zZmVySW5Qcm9ncmVzcwAAAAAINQAAAAAAAAAPT3duZXJBbHJlYWR5U2V0AAAACDY=",
        "AAAABQAAADZFdmVudCBlbWl0dGVkIHdoZW4gYW4gb3duZXJzaGlwIHRyYW5zZmVyIGlzIGluaXRpYXRlZC4AAAAAAAAAAAART3duZXJzaGlwVHJhbnNmZXIAAAAAAAABAAAAEm93bmVyc2hpcF90cmFuc2ZlcgAAAAAAAwAAAAAAAAAJb2xkX293bmVyAAAAAAAAEwAAAAAAAAAAAAAACW5ld19vd25lcgAAAAAAABMAAAAAAAAAAAAAABFsaXZlX3VudGlsX2xlZGdlcgAAAAAAAAQAAAAAAAAAAg==",
        "AAAABQAAADZFdmVudCBlbWl0dGVkIHdoZW4gYW4gb3duZXJzaGlwIHRyYW5zZmVyIGlzIGNvbXBsZXRlZC4AAAAAAAAAAAAaT3duZXJzaGlwVHJhbnNmZXJDb21wbGV0ZWQAAAAAAAEAAAAcb3duZXJzaGlwX3RyYW5zZmVyX2NvbXBsZXRlZAAAAAEAAAAAAAAACW5ld19vd25lcgAAAAAAABMAAAAAAAAAAg==" ]),
      options
    )
  }
  public readonly fromJSON = {
    fees: this.txFromJSON<i128>,
        owner: this.txFromJSON<Option<string>>,
        rollup: this.txFromJSON<Result<void>>,
        deposit: this.txFromJSON<Result<void>>,
        recover: this.txFromJSON<Result<void>>,
        upgrade: this.txFromJSON<null>,
        withdraw: this.txFromJSON<Result<void>>,
        block_height: this.txFromJSON<u32>,
        collect_fees: this.txFromJSON<Result<void>>,
        accept_ownership: this.txFromJSON<null>,
        latest_block_hash: this.txFromJSON<Buffer>,
        collateral_balance: this.txFromJSON<i128>,
        renounce_ownership: this.txFromJSON<Result<void>>,
        total_withdrawable: this.txFromJSON<i128>,
        transfer_ownership: this.txFromJSON<null>,
        withdrawal_allowances: this.txFromJSON<i128>
  }
}