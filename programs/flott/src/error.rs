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
    
    #[msg("The policy still have some receiver for there vesting.")]
    InvalidRequest,
    
    #[msg("Cancelable duration must be greater than the first split's unlock time")]
    InvalidCancelableDuration,
    
    #[msg("Enrollment must be activated before this action can be performed")]
    EnrollmentMustBeActivated,
    
    #[msg("Enrollment has already been cancelled")]
    EnrollmentAlreadyCancelled,
    
    #[msg("This enrollment is not cancelable")]
    EnrollmentNotCancelable,
    
    #[msg("Provided maker does not match the maker recorded on the vesting policy")]
    InvalidMaker,
    
    #[msg("One or more required signers are invalid")]
    InvalidSigners,
    
    #[msg("The cancel window for this enrollment has expired")]
    CancelWindowExpired,
    
    #[msg("The activation window for this enrollment has expired")]
    EnrollmentWindowExpired,
    
    #[msg("The activation window for this enrollment was not expired yet")]
    EnrollmentWindowNotExpired,
    
    #[msg("This enrollment has already been activated")]
    EnrollmentAlreadyActivated,
    
    #[msg("Token account owner does not match the expected owner")]
    InvalidTokenAccountOwner,
    
    #[msg("All requirements must be fulfilled before this action can be performed")]
    MustFulfillRequirements,
    
    #[msg("The specified time to claim the tranche not reached yet")]
    InvalidClaim,
    
    #[msg("Invalid expiry time - must be within 10 days of creation")]
    InvalidExpiry,
    
    #[msg("Invalid refund configuration - when non_refundable_percentage < 100%, refund_valid_until must be set")]
    InvalidRefundConfig,
    
    #[msg("Extended count must be greater than zero when provided")]
    InvalidExtendedCount,
    
    #[msg("Extend limit has been reached - no more extensions allowed")]
    ExtendLimitReached,
    
    #[msg("Extension too soon - must wait at least 1 hour before extending")]
    ExtendTooSoon,
    
    #[msg("Cannot extend beyond max expiry time")]
    MaxExpiryExceeded,
    
    #[msg("Refund is fully non-refundable (100% non-refundable percentage)")]
    FullyNonRefundable,
    
    #[msg("Refund window is still active - cannot payout before refund_valid_until")]
    RefundWindowActive,
    
    #[msg("Refund vault is missing or not configured")]
    RefundVaultMissing,
    
    #[msg("Nothing to refund - vault balance is zero")]
    NothingToRefund,
    
    #[msg("Order has not been paid yet - refund cannot be claimed")]
    OrderNotPaidYet,
    
    #[msg("Payer does not match the expected payer for this order")]
    PayerMismatch,
    
    #[msg("Invalid vault account provided")]
    InvalidVault,
    
    #[msg("Order has already been paid")]
    OrderAlreadyPaid,
    
    #[msg("Account must be writable to perform this operation")]
    AccountNotWritable,
    
    #[msg("Order has not expired yet - this action is only available after expiry")]
    OrderNotExpiredYet,
    
    #[msg("Token account owner does not match the expected owner")]
    TokenAccountOwnerMismatch,
    
    #[msg("The provided token account is invalid for this operation")]
    InvalidTokenAccount,
    
    #[msg("Split distribution is incomplete - not all splits have been paid out")]
    IncompleteSplitDistribution,
    
    #[msg("Number of accounts provided does not match the expected count")]
    AccountCountMismatch,
    
    #[msg("Provided share receiver does not match the expected recipient for this split")]
    InvalidShareReceiver,
}