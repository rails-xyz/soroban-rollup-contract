pub const WASM: &[u8] = soroban_sdk::contractfile!(
    file = "./target/wasm32v1-none/release/mock_usdc.wasm", sha256 =
    "909dd43dd2de8d50c03f0eaa4c573cae1513f08a9d0e7a6251cab67b4f3ceef7"
);
#[soroban_sdk::contractargs(name = "Args")]
#[soroban_sdk::contractclient(name = "Client")]
pub trait Contract {
    fn mint(env: soroban_sdk::Env, to: soroban_sdk::Address, amount: i128);
    fn name(env: soroban_sdk::Env) -> soroban_sdk::String;
    fn symbol(env: soroban_sdk::Env) -> soroban_sdk::String;
    fn approve(
        env: soroban_sdk::Env,
        owner: soroban_sdk::Address,
        spender: soroban_sdk::Address,
        amount: i128,
        expiration_ledger: u32,
    );
    fn balance(env: soroban_sdk::Env, account: soroban_sdk::Address) -> i128;
    fn decimals(env: soroban_sdk::Env) -> u32;
    fn transfer(
        env: soroban_sdk::Env,
        from: soroban_sdk::Address,
        to: soroban_sdk::Address,
        amount: i128,
    );
    fn allowance(
        env: soroban_sdk::Env,
        owner: soroban_sdk::Address,
        spender: soroban_sdk::Address,
    ) -> i128;
    fn total_supply(env: soroban_sdk::Env) -> i128;
    fn __constructor(env: soroban_sdk::Env, admin: soroban_sdk::Address);
    fn transfer_from(
        env: soroban_sdk::Env,
        spender: soroban_sdk::Address,
        from: soroban_sdk::Address,
        to: soroban_sdk::Address,
        amount: i128,
    );
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleAccountKey {
    pub index: u32,
    pub role: soroban_sdk::Symbol,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnerTokensKey {
    pub index: u32,
    pub owner: soroban_sdk::Address,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoyaltyInfo {
    pub basis_points: u32,
    pub receiver: soroban_sdk::Address,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Metadata {
    pub base_uri: soroban_sdk::String,
    pub name: soroban_sdk::String,
    pub symbol: soroban_sdk::String,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApprovalData {
    pub approved: soroban_sdk::Address,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Document {
    pub document_hash: soroban_sdk::BytesN<32>,
    pub timestamp: u64,
    pub uri: soroban_sdk::String,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SigningKey {
    pub public_key: soroban_sdk::Bytes,
    pub scheme: u32,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ed25519SignatureData {
    pub public_key: soroban_sdk::BytesN<32>,
    pub signature: soroban_sdk::BytesN<64>,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Secp256k1SignatureData {
    pub public_key: soroban_sdk::BytesN<65>,
    pub recovery_id: u32,
    pub signature: soroban_sdk::BytesN<64>,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Secp256r1SignatureData {
    pub public_key: soroban_sdk::BytesN<65>,
    pub signature: soroban_sdk::BytesN<64>,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Claim {
    pub data: soroban_sdk::Bytes,
    pub issuer: soroban_sdk::Address,
    pub scheme: u32,
    pub signature: soroban_sdk::Bytes,
    pub topic: u32,
    pub uri: soroban_sdk::String,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CountryData {
    pub country: CountryRelation,
    pub metadata: Option<soroban_sdk::Map<soroban_sdk::Symbol, soroban_sdk::String>>,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentityProfile {
    pub countries: soroban_sdk::Vec<CountryData>,
    pub identity_type: IdentityType,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Metadata {
    pub decimals: u32,
    pub name: soroban_sdk::String,
    pub symbol: soroban_sdk::String,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AllowanceKey {
    pub owner: soroban_sdk::Address,
    pub spender: soroban_sdk::Address,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AllowanceData {
    pub amount: i128,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum AccessControlStorageKey {
    RoleAccounts(RoleAccountKey),
    HasRole(soroban_sdk::Address, soroban_sdk::Symbol),
    RoleAccountsCount(soroban_sdk::Symbol),
    RoleAdmin(soroban_sdk::Symbol),
    Admin,
    PendingAdmin,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OwnableStorageKey {
    Owner,
    PendingOwner,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MerkleDistributorStorageKey {
    Root,
    Claimed(u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Rounding {
    Floor,
    Ceil,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PausableStorageKey {
    Paused,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum NFTEnumerableStorageKey {
    TotalSupply,
    OwnerTokens(OwnerTokensKey),
    OwnerTokensIndex(u32),
    GlobalTokens(u32),
    GlobalTokensIndex(u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum NFTConsecutiveStorageKey {
    Approval(u32),
    Owner(u32),
    OwnershipBucket(u32),
    BurnedToken(u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum NFTRoyaltiesStorageKey {
    DefaultRoyalty,
    TokenRoyalty(u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum NFTSequentialStorageKey {
    TokenIdCounter,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum NFTStorageKey {
    Owner(u32),
    Balance(soroban_sdk::Address),
    Approval(u32),
    ApprovalForAll(soroban_sdk::Address, soroban_sdk::Address),
    Metadata,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComplianceHook {
    Transferred,
    Created,
    Destroyed,
    CanTransfer,
    CanCreate,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DataKey {
    HookModules(ComplianceHook),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DocumentStorageKey {
    Index(soroban_sdk::BytesN<32>),
    Bucket(u32),
    Count,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ClaimIssuerStorageKey {
    Topics(u32),
    Pairs(SigningKey),
    RevokedClaim(soroban_sdk::BytesN<32>),
    ClaimNonce(soroban_sdk::Address, u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ClaimsStorageKey {
    Claim(soroban_sdk::BytesN<32>),
    ClaimsByTopic(u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IdentityVerifierStorageKey {
    ClaimTopicsAndIssuers,
    IdentityRegistryStorage,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ClaimTopicsAndIssuersStorageKey {
    ClaimTopics,
    TrustedIssuers,
    IssuerClaimTopics(soroban_sdk::Address),
    ClaimTopicIssuers(u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IdentityType {
    Individual,
    Organization,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IRSStorageKey {
    Identity(soroban_sdk::Address),
    IdentityProfile(soroban_sdk::Address),
    RecoveredTo(soroban_sdk::Address),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CountryRelation {
    Individual(IndividualCountryRelation),
    Organization(OrganizationCountryRelation),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IndividualCountryRelation {
    Residence(u32),
    Citizenship(u32),
    SourceOfFunds(u32),
    TaxResidency(u32),
    Custom(soroban_sdk::Symbol, u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OrganizationCountryRelation {
    Incorporation(u32),
    OperatingJurisdiction(u32),
    TaxJurisdiction(u32),
    SourceOfFunds(u32),
    Custom(soroban_sdk::Symbol, u32),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TokenBinderStorageKey {
    TokenBucket(u32),
    TotalCount,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RWAStorageKey {
    AddressFrozen(soroban_sdk::Address),
    FrozenTokens(soroban_sdk::Address),
    Compliance,
    OnchainId,
    Version,
    IdentityVerifier,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum VaultStorageKey {
    AssetAddress,
    VirtualDecimalsOffset,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum AllowListStorageKey {
    Allowed(soroban_sdk::Address),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BlockListStorageKey {
    Blocked(soroban_sdk::Address),
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SACAdminGenericDataKey {
    Sac,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SACAdminWrapperDataKey {
    Sac,
}
#[soroban_sdk::contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum StorageKey {
    TotalSupply,
    Balance(soroban_sdk::Address),
    Allowance(AllowanceKey),
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RoleTransferError {
    NoPendingTransfer = 2200,
    InvalidLiveUntilLedger = 2201,
    InvalidPendingAccount = 2202,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum AccessControlError {
    Unauthorized = 2000,
    AdminNotSet = 2001,
    IndexOutOfBounds = 2002,
    AdminRoleNotFound = 2003,
    RoleCountIsNotZero = 2004,
    RoleNotFound = 2005,
    AdminAlreadySet = 2006,
    RoleNotHeld = 2007,
    RoleIsEmpty = 2008,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum OwnableError {
    OwnerNotSet = 2100,
    TransferInProgress = 2101,
    OwnerAlreadySet = 2102,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum UpgradeableError {
    MigrationNotAllowed = 1100,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MerkleDistributorError {
    RootNotSet = 1300,
    IndexAlreadyClaimed = 1301,
    InvalidProof = 1302,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SorobanFixedPointError {
    ZeroDenominator = 1500,
    PhantomOverflow = 1501,
    ResultOverflow = 1502,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CryptoError {
    MerkleProofOutOfBounds = 1400,
    MerkleIndexOutOfBounds = 1401,
    HasherEmptyState = 1402,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PausableError {
    EnforcedPause = 1000,
    ExpectedPause = 1001,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum NonFungibleTokenError {
    NonExistentToken = 200,
    IncorrectOwner = 201,
    InsufficientApproval = 202,
    InvalidApprover = 203,
    InvalidLiveUntilLedger = 204,
    MathOverflow = 205,
    TokenIDsAreDepleted = 206,
    InvalidAmount = 207,
    TokenNotFoundInOwnerList = 208,
    TokenNotFoundInGlobalList = 209,
    UnsetMetadata = 210,
    BaseUriMaxLenExceeded = 211,
    InvalidRoyaltyAmount = 212,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComplianceError {
    ModuleAlreadyRegistered = 360,
    ModuleNotRegistered = 361,
    ModuleBoundExceeded = 362,
    TokenNotBound = 363,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DocumentError {
    DocumentNotFound = 380,
    MaxDocumentsReached = 381,
    UriTooLong = 382,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ClaimIssuerError {
    SigDataMismatch = 350,
    KeyIsEmpty = 351,
    KeyAlreadyAllowed = 352,
    KeyNotFound = 353,
    NotAllowed = 354,
    LimitExceeded = 355,
    NoKeysForTopic = 356,
    InvalidClaimDataExpiration = 357,
    Secp256k1RecoveryFailed = 358,
    MathOverflow = 359,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ClaimsError {
    ClaimNotFound = 340,
    ClaimNotValid = 341,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RWAError {
    InsufficientBalance = 300,
    LessThanZero = 301,
    AddressFrozen = 302,
    InsufficientFreeTokens = 303,
    IdentityVerificationFailed = 304,
    TransferNotCompliant = 305,
    MintNotCompliant = 306,
    ComplianceNotSet = 307,
    OnchainIdNotSet = 308,
    VersionNotSet = 309,
    ClaimTopicsAndIssuersNotSet = 310,
    IdentityRegistryStorageNotSet = 311,
    IdentityVerifierNotSet = 312,
    IdentityMismatch = 313,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ClaimTopicsAndIssuersError {
    ClaimTopicDoesNotExist = 370,
    IssuerDoesNotExist = 371,
    ClaimTopicAlreadyExists = 372,
    IssuerAlreadyExists = 373,
    MaxClaimTopicsLimitReached = 374,
    MaxIssuersLimitReached = 375,
    ClaimTopicsSetCannotBeEmpty = 376,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IRSError {
    IdentityOverwrite = 320,
    IdentityNotFound = 321,
    CountryDataNotFound = 322,
    EmptyCountryList = 323,
    MaxCountryEntriesReached = 324,
    AccountRecovered = 325,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TokenBinderError {
    TokenNotFound = 330,
    TokenAlreadyBound = 331,
    MaxTokensReached = 332,
    BindBatchTooLarge = 333,
    BindBatchDuplicates = 334,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum VaultTokenError {
    VaultAssetAddressNotSet = 400,
    VaultAssetAddressAlreadySet = 401,
    VaultVirtualDecimalsOffsetAlreadySet = 402,
    VaultInvalidAssetsAmount = 403,
    VaultInvalidSharesAmount = 404,
    VaultExceededMaxDeposit = 405,
    VaultExceededMaxMint = 406,
    VaultExceededMaxWithdraw = 407,
    VaultExceededMaxRedeem = 408,
    VaultMaxDecimalsOffsetExceeded = 409,
    MathOverflow = 410,
}
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum FungibleTokenError {
    InsufficientBalance = 100,
    InsufficientAllowance = 101,
    InvalidLiveUntilLedger = 102,
    LessThanZero = 103,
    MathOverflow = 104,
    UnsetMetadata = 105,
    ExceededCap = 106,
    InvalidCap = 107,
    CapNotSet = 108,
    SACNotSet = 109,
    SACAddressMismatch = 110,
    SACMissingFnParam = 111,
    SACInvalidFnParam = 112,
    UserNotAllowed = 113,
    UserBlocked = 114,
}
#[soroban_sdk::contractevent(topics = ["role_granted"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleGranted {
    #[topic]
    pub role: soroban_sdk::Symbol,
    #[topic]
    pub account: soroban_sdk::Address,
    pub caller: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["role_revoked"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleRevoked {
    #[topic]
    pub role: soroban_sdk::Symbol,
    #[topic]
    pub account: soroban_sdk::Address,
    pub caller: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["admin_renounced"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AdminRenounced {
    #[topic]
    pub admin: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["role_admin_changed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleAdminChanged {
    #[topic]
    pub role: soroban_sdk::Symbol,
    pub previous_admin_role: soroban_sdk::Symbol,
    pub new_admin_role: soroban_sdk::Symbol,
}
#[soroban_sdk::contractevent(topics = ["admin_transfer_completed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AdminTransferCompleted {
    #[topic]
    pub new_admin: soroban_sdk::Address,
    pub previous_admin: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["admin_transfer_initiated"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AdminTransferInitiated {
    #[topic]
    pub current_admin: soroban_sdk::Address,
    pub new_admin: soroban_sdk::Address,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contractevent(topics = ["ownership_transfer"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnershipTransfer {
    pub old_owner: soroban_sdk::Address,
    pub new_owner: soroban_sdk::Address,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contractevent(topics = ["ownership_renounced"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnershipRenounced {
    pub old_owner: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["ownership_transfer_completed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OwnershipTransferCompleted {
    pub new_owner: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["set_root"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SetRoot {
    pub root: soroban_sdk::Bytes,
}
#[soroban_sdk::contractevent(topics = ["set_claimed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SetClaimed {
    pub index: soroban_sdk::Val,
}
#[soroban_sdk::contractevent(topics = ["paused"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Paused {}
#[soroban_sdk::contractevent(topics = ["unpaused"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Unpaused {}
#[soroban_sdk::contractevent(topics = ["consecutive_mint"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConsecutiveMint {
    #[topic]
    pub to: soroban_sdk::Address,
    pub from_token_id: u32,
    pub to_token_id: u32,
}
#[soroban_sdk::contractevent(topics = ["burn"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Burn {
    #[topic]
    pub from: soroban_sdk::Address,
    pub token_id: u32,
}
#[soroban_sdk::contractevent(topics = ["set_token_royalty"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SetTokenRoyalty {
    #[topic]
    pub receiver: soroban_sdk::Address,
    #[topic]
    pub token_id: u32,
    pub basis_points: u32,
}
#[soroban_sdk::contractevent(topics = ["set_default_royalty"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SetDefaultRoyalty {
    #[topic]
    pub receiver: soroban_sdk::Address,
    pub basis_points: u32,
}
#[soroban_sdk::contractevent(topics = ["remove_token_royalty"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RemoveTokenRoyalty {
    #[topic]
    pub token_id: u32,
}
#[soroban_sdk::contractevent(topics = ["mint"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mint {
    #[topic]
    pub to: soroban_sdk::Address,
    pub token_id: u32,
}
#[soroban_sdk::contractevent(topics = ["approve"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Approve {
    #[topic]
    pub approver: soroban_sdk::Address,
    #[topic]
    pub token_id: u32,
    pub approved: soroban_sdk::Address,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contractevent(topics = ["transfer"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Transfer {
    #[topic]
    pub from: soroban_sdk::Address,
    #[topic]
    pub to: soroban_sdk::Address,
    pub token_id: u32,
}
#[soroban_sdk::contractevent(topics = ["approve_for_all"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApproveForAll {
    #[topic]
    pub owner: soroban_sdk::Address,
    pub operator: soroban_sdk::Address,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contractevent(topics = ["module_added"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ModuleAdded {
    #[topic]
    pub hook: ComplianceHook,
    pub module: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["module_removed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ModuleRemoved {
    #[topic]
    pub hook: ComplianceHook,
    pub module: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["document_removed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DocumentRemoved {
    #[topic]
    pub name: soroban_sdk::BytesN<32>,
}
#[soroban_sdk::contractevent(topics = ["document_updated"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DocumentUpdated {
    #[topic]
    pub name: soroban_sdk::BytesN<32>,
    pub uri: soroban_sdk::String,
    pub document_hash: soroban_sdk::BytesN<32>,
    pub timestamp: u64,
}
#[soroban_sdk::contractevent(topics = ["key_allowed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeyAllowed {
    #[topic]
    pub public_key: soroban_sdk::Bytes,
    pub registry: soroban_sdk::Address,
    pub scheme: u32,
    pub claim_topic: u32,
}
#[soroban_sdk::contractevent(topics = ["key_removed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeyRemoved {
    #[topic]
    pub public_key: soroban_sdk::Bytes,
    pub registry: soroban_sdk::Address,
    pub scheme: u32,
    pub claim_topic: u32,
}
#[soroban_sdk::contractevent(topics = ["claim_revoked"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClaimRevoked {
    #[topic]
    pub identity: soroban_sdk::Address,
    #[topic]
    pub claim_topic: u32,
    #[topic]
    pub revoked: bool,
    pub claim_data: soroban_sdk::Bytes,
}
#[soroban_sdk::contractevent(topics = ["signatures_invalidated"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SignaturesInvalidated {
    #[topic]
    pub identity: soroban_sdk::Address,
    #[topic]
    pub claim_topic: u32,
    pub nonce: u32,
}
#[soroban_sdk::contractevent(topics = ["claim_added"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClaimAdded {
    #[topic]
    pub claim: Claim,
}
#[soroban_sdk::contractevent(topics = ["claim_changed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClaimChanged {
    #[topic]
    pub claim: Claim,
}
#[soroban_sdk::contractevent(topics = ["claim_removed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClaimRemoved {
    #[topic]
    pub claim: Claim,
}
#[soroban_sdk::contractevent(topics = ["burn"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Burn {
    #[topic]
    pub from: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["mint"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mint {
    #[topic]
    pub to: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["claim_topic_added"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClaimTopicAdded {
    #[topic]
    pub claim_topic: u32,
}
#[soroban_sdk::contractevent(topics = ["claim_topic_removed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClaimTopicRemoved {
    #[topic]
    pub claim_topic: u32,
}
#[soroban_sdk::contractevent(topics = ["trusted_issuer_added"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TrustedIssuerAdded {
    #[topic]
    pub trusted_issuer: soroban_sdk::Address,
    pub claim_topics: soroban_sdk::Vec<u32>,
}
#[soroban_sdk::contractevent(topics = ["issuer_topics_updated"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IssuerTopicsUpdated {
    #[topic]
    pub trusted_issuer: soroban_sdk::Address,
    pub claim_topics: soroban_sdk::Vec<u32>,
}
#[soroban_sdk::contractevent(topics = ["trusted_issuer_removed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TrustedIssuerRemoved {
    #[topic]
    pub trusted_issuer: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["identity_stored"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentityStored {
    #[topic]
    pub account: soroban_sdk::Address,
    #[topic]
    pub identity: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["country_data_added"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CountryDataAdded {
    #[topic]
    pub account: soroban_sdk::Address,
    #[topic]
    pub country_data: CountryData,
}
#[soroban_sdk::contractevent(topics = ["identity_modified"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentityModified {
    #[topic]
    pub old_identity: soroban_sdk::Address,
    #[topic]
    pub new_identity: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["identity_unstored"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentityUnstored {
    #[topic]
    pub account: soroban_sdk::Address,
    #[topic]
    pub identity: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["identity_recovered"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentityRecovered {
    #[topic]
    pub old_account: soroban_sdk::Address,
    #[topic]
    pub new_account: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["country_data_removed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CountryDataRemoved {
    #[topic]
    pub account: soroban_sdk::Address,
    #[topic]
    pub country_data: CountryData,
}
#[soroban_sdk::contractevent(topics = ["country_data_modified"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CountryDataModified {
    #[topic]
    pub account: soroban_sdk::Address,
    #[topic]
    pub country_data: CountryData,
}
#[soroban_sdk::contractevent(topics = ["tokens_frozen"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TokensFrozen {
    #[topic]
    pub user_address: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["address_frozen"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AddressFrozen {
    #[topic]
    pub user_address: soroban_sdk::Address,
    #[topic]
    pub is_frozen: bool,
}
#[soroban_sdk::contractevent(topics = ["compliance_set"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ComplianceSet {
    #[topic]
    pub compliance: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["tokens_unfrozen"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TokensUnfrozen {
    #[topic]
    pub user_address: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["recovery_success"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecoverySuccess {
    #[topic]
    pub old_account: soroban_sdk::Address,
    #[topic]
    pub new_account: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["identity_verifier_set"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentityVerifierSet {
    #[topic]
    pub identity_verifier: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["token_onchain_id_updated"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TokenOnchainIdUpdated {
    #[topic]
    pub onchain_id: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["claim_topics_and_issuers_set"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClaimTopicsAndIssuersSet {
    #[topic]
    pub claim_topics_and_issuers: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["token_bound"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TokenBound {
    #[topic]
    pub token: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["token_unbound"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TokenUnbound {
    #[topic]
    pub token: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["deposit"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Deposit {
    #[topic]
    pub operator: soroban_sdk::Address,
    #[topic]
    pub from: soroban_sdk::Address,
    #[topic]
    pub receiver: soroban_sdk::Address,
    pub assets: i128,
    pub shares: i128,
}
#[soroban_sdk::contractevent(topics = ["withdraw"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Withdraw {
    #[topic]
    pub operator: soroban_sdk::Address,
    #[topic]
    pub receiver: soroban_sdk::Address,
    #[topic]
    pub owner: soroban_sdk::Address,
    pub assets: i128,
    pub shares: i128,
}
#[soroban_sdk::contractevent(topics = ["burn"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Burn {
    #[topic]
    pub from: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["user_allowed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserAllowed {
    #[topic]
    pub user: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["user_disallowed"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserDisallowed {
    #[topic]
    pub user: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["user_blocked"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserBlocked {
    #[topic]
    pub user: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["user_unblocked"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserUnblocked {
    #[topic]
    pub user: soroban_sdk::Address,
}
#[soroban_sdk::contractevent(topics = ["mint"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Mint {
    #[topic]
    pub to: soroban_sdk::Address,
    pub amount: i128,
}
#[soroban_sdk::contractevent(topics = ["approve"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Approve {
    #[topic]
    pub owner: soroban_sdk::Address,
    #[topic]
    pub spender: soroban_sdk::Address,
    pub amount: i128,
    pub live_until_ledger: u32,
}
#[soroban_sdk::contractevent(topics = ["transfer"], export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Transfer {
    #[topic]
    pub from: soroban_sdk::Address,
    #[topic]
    pub to: soroban_sdk::Address,
    pub amount: i128,
}

