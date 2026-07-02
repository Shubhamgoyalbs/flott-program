use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("api_user.owner does not match the supplied owner account")]
    OwnerMismatch,
    
    #[msg("This ApiUser has already been authorized (authority is set)")]
    AlreadyAuthorized,
    
    #[msg("Invalid authorize request, the server key doesn't match the actual server key")]
    InvalidAuthorizeRequest,
    
    #[msg("This ApiUser is already active")]
    AlreadyActive,
    
    #[msg("Vault balance too low — must exceed API_USER_MIN_BALANCE + API_USER_MPC_MIN_BALANCE")]
    InsufficientVaultBalance,
    
    #[msg("Deposit would not bring vault above API_USER_MIN_BALANCE")]
    InsufficientDeposit,
    
    #[msg("Arithmetic overflow")]
    Overflow,
    
    #[msg("Arithmetic underflow")]
    Underflow,
    
    #[msg("No authority is set on this ApiUser — account was never authorized")]
    NotAuthorized,
    
    #[msg("Signer does not match the authority recorded on this ApiUser")]
    AuthorityMismatch,
    
    #[msg("This ApiUser is already inactive")]
    AlreadyNotActive,
    
    #[msg("API user account is inactive")]
    ApiUserInactive,
    
    #[msg("Maximum retries value is invalid")]
    InvalidMaxRetries,
    
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    
    #[msg("Amount must be greater than ne cycle price")]
    InsufficientAmount,
    
    #[msg("Subscriber has already been initialized for this policy")]
    SubscriberAlreadyInitialized,
    
    #[msg("Recipient account does not match the one recorded on the subscription policy")]
    InvalidRecipient,
    
    #[msg("Max cycle count is invalid — must be greater than zero")]
    InvalidMaxCycle,
    
    #[msg("Subscription policy is inactive and cannot accept new subscribers")]
    PolicyInactive,
    
    #[msg("Subscriber has not been initialized — next_charge_at is not set")]
    SubscriberNotInitialized,
    
    #[msg("Arithmetic overflow during calculation")]
    ArithmeticOverflow,
    
    #[msg("Vault balance is insufficient after deposit to meet the minimum required threshold")]
    InsufficientVaultBalanceAfterDeposit,
    
    #[msg("Scheduler request is invalid — charge time has not yet been reached")]
    InvalidSchedulerRequest,
    
    #[msg("Subscriber account does not match the one recorded on the subscriber PDA")]
    SubscriberMismatch,
    
    #[msg("Token transfers are not yet implemented - only native SOL is supported")]
    InvalidTokenMint,
    
    #[msg("Update authority must not be the default pubkey")]
    InvalidUpdateAuthority,
    
    #[msg("Must follow the type safety")]
    MathOverflow,
    
    #[msg("Cliff duration must be greater than zero and must not overflow when added to any unlock time")]
    InvalidCliffDuration,
    
    #[msg("Split unlock time must not be before the policy start time")]
    InvalidUnlockTime,
    
    #[msg("Split percentage must be greater than zero")]
    InvalidSplitPercentage,
    
    #[msg("Duplicate unlock time found across splits")]
    DuplicateUnlockTime,
    
    #[msg("At least one split must be provided")]
    EmptySplits,
    
    #[msg("Active split percentages must sum to exactly 100_000_000")]
    InvalidSplitTotal,
    
    #[msg("")]
    InvalidTokenAccountOwner,
    
    #[msg("")]
    MustFulfillRequirements,
}
