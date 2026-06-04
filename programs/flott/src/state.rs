use anchor_lang::prelude::*;

use crate::error::ErrorCode;

/// Captures payment details once an order has been successfully paid.
/// Stored inside `Refund.order_payment` as `Option<OrderPayment>`.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct OrderPayment {
  /// The wallet that paid for the order.
  pub payer: Pubkey,
  
  /// Unix timestamp of when the payment was made.
  pub paid_at: i64,
}

/// Represents a single recipient's share in a split payment policy.
/// Used as entries in `Split.shares` to define how a payment
/// is distributed across multiple wallets.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct SplitShare {
  /// The wallet address that will receive this portion of the payment.
  pub account: Pubkey,
  
  /// The fraction of the total payment allocated to `account`, in 6-decimal precision.
  /// Formula: `percentage / 100_000_000 * 100 = %`
  /// Examples: 1_000_000 = 1% | 50_000_000 = 50% | 100_000_000 = 100%
  /// All active shares in `Split.shares` must sum to exactly `100_000_000`.
  pub percentage: u32,
}

/// Represents a single tranche in a vesting schedule.
/// Each split defines when and how much of `total_amount` unlocks.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct VestingSplit {
  /// Unix timestamp when this tranche becomes claimable.
  pub unlock_at: i64,
  
  /// Percentage of `total_amount` released at `unlock_at`, in 6-decimal precision.
  /// Formula: `percentage / 100_000_000 * 100 = %`
  /// Examples: 1_000_000 = 1% | 25_000_000 = 25% | 100_000_000 = 100%
  /// All active splits across `VestingPolicy.splits` must sum to exactly `100_000_000`.
  pub percentage: u32,
}

/// Represents a merchant or integrator using the platform via API.
/// Each `ApiUser` has its own fee configuration and an associated vault
/// that holds funds collected through its orders.
#[account]
#[derive(InitSpace)]
pub struct ApiUser {
  /// The authority keypair that controls this account.
  /// Generated and managed by an MPC provider (e.g. Para / Turnkey).
  pub authority: Option<Pubkey>,
  
  /// The owner of this api key and the owner of the funds generated
  pub owner: Pubkey,
  
  /// PDA bump seed for this `ApiUser` account.
  pub bump: u8,
  
  /// The vault account that holds funds collected by this `ApiUser`.
  /// This is a system-owned account derived from this `ApiUser` PDA.
  pub vault: Pubkey,
  
  /// PDA bump seed for the `vault` account.
  pub vault_bump: u8,
  
  /// Whether this `ApiUser` account is currently active.
  /// Inactive accounts cannot create or process orders.
  /// Default to inactive, activated only after server authorized key signs the key.
  pub is_active: bool,
  
  /// Fee charged on each transaction, in 6-decimal precision.
  /// Formula: `fee_percentage / 100_000_000 * 100 = fee%`
  /// Examples: 1_000_000 = 1% | 50_000_000 = 50% | 100_000_000 = 100%
  /// Must be in range [0, 100_000_000].
  pub fee_percentage: u32,
  
  /// Unix timestamp of when this account was created.
  pub created_at: i64,
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 32],
}

/// Represents a payment order created by an `ApiUser`.
/// An order defines how much to collect, in which token,
/// from whom, and where the funds should go.
#[account]
#[derive(InitSpace)]
pub struct Order {
  /// A reference ID linking this on-chain order to a Web2 record.
  /// Accepts any 32-byte identifier: ULID, UUID, CUID, or custom.
  pub metadata: [u8; 32],
  
  /// The exact amount the payer must send, denominated in the
  /// smallest unit of `token` (e.g. lamports for SOL, micro-USDC for USDC).
  /// Does NOT include the platform fee or `ApiUser` fee.
  pub total_amount: u64,
  
  /// The SPL token mint used for payment.
  /// Use `So11111111111111111111111111111111111111112` for native SOL.
  pub token: Pubkey,
  
  /// The `ApiUser` account that created this order.
  pub api_user: Pubkey,
  
  /// Restricts who can pay this order.
  /// `None`  — anyone can pay.
  /// `Some`  — only the specified wallet can pay.
  pub payer: Option<Pubkey>,
  
  /// Determines where funds are sent after payment.
  /// `None`  — a split policy governs distribution (see `Split` account).
  /// `Some`  — the entire amount goes to this single recipient.
  pub recipient: Option<Pubkey>,
  
  /// Unix timestamp of when this order was created.
  pub created_at: i64,
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 16],
}

/// Represents a refund policy attached to a specific `Order`.
/// Holds the vault for escrowed funds and tracks whether the
/// order has been paid and is ready for refund claim.
///
/// State is derived from fields — no explicit status enum needed:
/// - `order_payment: None`   → order not yet paid
/// - `order_payment: Some`   → paid, refund claimable until `refund_valid_until`
/// - account deleted         → refund was claimed or expired
#[account]
#[derive(InitSpace)]
pub struct Refund {
  /// The `Order` account this refund is associated with.
  pub order: Pubkey,
  
  /// The portion of `total_amount` that is non-refundable, in 6-decimal precision.
  /// Formula: `non_refundable_percentage / 100_000_000 * 100 = %`
  /// Examples: 1_000_000 = 1% | 50_000_000 = 50% | 100_000_000 = 100%
  /// Must be in range [0, 100_000_000].
  pub non_refundable_percentage: u32,
  
  /// PDA bump seed for this `Refund` account.
  pub bump: u8,
  
  /// Unix timestamp after which this refund is no longer claimable.
  /// Once `Clock::get().unix_timestamp > refund_valid_until`,
  /// the refund is considered expired and this account should be closed.
  pub refund_valid_until: i64,
  
  /// Populated once the associated order is paid.
  /// `None`  — order has not been paid yet.
  /// `Some`  — order is paid; refund can be claimed by the recorded payer.
  pub order_payment: Option<OrderPayment>,
  
  /// The escrow vault that holds the payer's funds until
  /// the refund is claimed or the window expires.
  pub vault: Pubkey,
  
  /// PDA bump seed for the `vault` account.
  pub vault_bump: u8,
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 16],
}

/// Marks an order as having a deadline for payment.
/// If the order is not paid by `expires_at`, this account
/// should be closed and the associated `Order` account deleted.
#[account]
#[derive(InitSpace)]
pub struct Expiry {
  /// PDA bump seed for this `Expiry` account.
  pub bump: u8,
  
  /// Unix timestamp after which the order is considered expired.
  /// If `Clock::get().unix_timestamp > expires_at` and the order
  /// is unpaid, the order and this account should be closed.
  /// Must be within 10 days of `created_at`. Defaults to 24 hours
  /// from order creation if not explicitly set.
  pub expires_at: i64,
  
  /// Optional wallet authorized to extend the order's expiry deadline.
  /// `None`  — expiry cannot be extended; order expires at `expires_at`.
  /// `Some`  — the specified wallet may call the extend instruction,
  ///           which requires payment of an additional extension fee.
  pub extend_authority: Option<Pubkey>,
  
  /// Optional key for bounding the number of time we can extend
  /// `None`  — if extend policy not exists
  /// `None`  —  by one on extension,
  ///           valid till 0if extend policy not exists
  pub extended_count: Option<u8>,
  
  /// hard ceiling — cannot extend beyond this timestamp
  pub max_expires_at: i64,
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 16],
}

/// Defines a split payment policy for an `Order`.
/// When `Order.recipient` is `None`, this account governs
/// how the payment is distributed across multiple recipients.
#[account]
#[derive(InitSpace)]
pub struct Split {
  /// PDA bump seed for this `Split` account.
  pub bump: u8,
  
  /// Up to 7 recipients that share the payment distribution.
  /// `None` slots are ignored; active slots must sum to exactly
  /// `100_000_000` (100% in 6-decimal precision).
  /// At least one share must be populated for a valid split policy.
  pub shares: [Option<SplitShare>; 7],
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 16],
}

/// Defines a vesting schedule for releasing funds over time.
/// Controls when and how much of the locked amount can be unlocked
/// across up to 8 time-based tranches via `splits`.
///
/// State is derived from fields — no explicit status enum needed:
/// - `cancelled_at: None`  + `starts_at` in future  → pending; no tranches claimable yet
/// - `cancelled_at: None`  + `starts_at` passed     → active; eligible tranches are claimable
/// - `cancelled_at: Some`                            → cancelled; remaining funds returned to maker
/// - `total_claimed == total_amount`                 → completed; all tranches claimed
/// - account deleted                                 → fully closed
#[account]
#[derive(InitSpace)]
pub struct VestingPolicy {
  /// The wallet that created and funded this vesting policy.
  pub maker: Pubkey,
  
  /// The `ApiUser` account this vesting policy belongs to.
  pub api_user: Pubkey,
  
  /// The SPL token mint used for vesting payouts.
  /// Use `So11111111111111111111111111111111111111112` for native SOL.
  pub token: Pubkey,
  
  /// The escrow vault that holds the total locked funds for this policy.
  /// Funds are distributed to individual `VestingReceiver` vaults as
  /// receivers are enrolled and tranches become claimable.
  pub vault: Pubkey,
  
  /// PDA bump seed for the `vault` account.
  pub vault_bump: u8,
  
  /// Total amount locked in this vesting policy, denominated in
  /// the smallest unit of `token` (e.g. lamports, micro-USDC).
  /// Must equal the sum of all tranche amounts derived from `splits`.
  pub total_amount: u64,
  
  /// Up to 8 time-based tranches defining the unlock schedule.
  /// `None` slots are ignored; active slots must sum to exactly
  /// `100_000_000` (100% in 6-decimal precision).
  pub splits: [Option<VestingSplit>; 8],
  
  /// Optional cliff duration in seconds from `starts_at`.
  /// `None`  — no cliff; tranches unlock purely based on `unlock_at`.
  /// `Some`  — no tranche is claimable until `starts_at + cliff_duration`
  ///           has elapsed, even if `unlock_at` has passed.
  pub cliff_duration: Option<i64>,
  
  /// Current count of active `VestingReceiver` accounts under this policy.
  /// Incremented on enrollment, decremented on cancellation or completion.
  pub receiver_count: u8,
  
  /// Wallet authorized to modify this vesting policy.
  /// Can update splits, or cancel the schedule before it starts.
  pub update_authority: Pubkey,
  
  /// Optional wallet authorized to cancel this policy mid-schedule.
  /// `None`  — policy cannot be cancelled once started.
  /// `Some`  — this wallet may invoke cancellation at any time.
  pub cancel_authority: Option<Pubkey>,
  
  /// Unix timestamp of when this policy was cancelled.
  /// `None`  — not cancelled; policy is pending, active, or complete.
  /// `Some`  — cancelled at this timestamp; all remaining unclaimed
  ///           funds were returned to `maker`.
  pub cancelled_at: Option<i64>,
  
  /// Unix timestamp of when this vesting policy was created.
  pub created_at: i64,
  
  /// Unix timestamp of when this vesting policy was last updated.
  /// `None`  — never modified after creation.
  /// `Some`  — last modification timestamp.
  pub updated_at: Option<i64>,
  
  /// PDA bump seed for this `VestingPolicy` account.
  pub bump: u8,
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 32],
}

/// Represents a receiver enrolled in a `VestingPolicy`.
/// Tracks the receiver's vault, cancellation settings,
/// vesting start time, and minimum lock period before cancellation.
///
/// State is derived from fields:
/// - `started_at: None`      → waiting for receiver to sign (if `is_cancelable: true`)
///                           → or not yet begun (if `is_cancelable: false`)
/// - `started_at: Some`      → vesting is active and tranches are accruing
/// - `cancelled_at: Some`    → vesting was cancelled; remaining funds returned to maker
/// - account deleted         → all tranches fully claimed
#[account]
#[derive(InitSpace)]
pub struct VestingReceiver {
  /// The `VestingPolicy` this receiver belongs to.
  pub vesting_policy: Pubkey,
  
  /// The escrow vault that holds this receiver's vested funds
  /// until each tranche is claimed.
  pub vault: Pubkey,
  
  /// PDA bump seed for the `vault` account.
  pub vault_bump: u8,
  
  /// The wallet address that will receive vested fund tranches.
  pub receiver: Pubkey,
  
  /// Whether the maker can cancel this vesting before it completes.
  /// `None` — vesting is irrevocable once started.
  /// `Some(val)`  — maker can cancel after `min_vest_duration that is (val)` has elapsed.
  pub is_cancelable: Option<i64>,
  
  /// Unix timestamp of when vesting officially begins.
  /// `None`  — if `is_cancelable: true`, vesting starts only after
  ///           the receiver signs an acceptance instruction.
  ///           if `is_cancelable: false`, starts immediately at creation.
  /// `Some`  — vesting has started; tranches unlock relative to this timestamp.
  pub started_at: Option<i64>,
  
  /// Unix timestamp of when this vesting was cancelled.
  /// `None`  — not cancelled; vesting is active or complete.
  /// `Some`  — cancelled at this timestamp; remaining unclaimed funds
  ///           were returned to the maker.
  pub cancelled_at: Option<i64>,
  
  /// Total amount already claimed by the receiver across all tranches.
  pub claimed_amount: u64,
  
  /// PDA bump seed for this `VestingReceiver` account.
  pub bump: u8,
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 16],
}

/// Defines the interval after which payment has to re-occur
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum BillingInterval {
  /// Every N seconds (arbitrary precision).
  Custom { seconds: i64 },
  Daily,
  Weekly,
  Monthly,
  Yearly,
}

/// Defines a recurring subscription payment policy for an `Order`.
/// Governs how and when periodic charges are made to enrolled subscribers.
///
/// State is derived from fields — no explicit status enum needed:
/// - `max_cycles: None`                          → policy bills indefinitely
/// - `max_cycles: Some(n)` + cycles not reached  → policy is active and bounded
/// - `max_cycles: Some(n)` + all cycles reached  → policy is exhausted; no new charges
/// - account deleted                             → policy was closed by `authority`
#[account]
#[derive(InitSpace)]
pub struct SubscriptionPolicy {
  /// PDA bump seed for this `SubscriptionPolicy` account.
  pub bump: u8,
  
  /// The authority that can mutate or close this policy (typically the merchant).
  pub authority: Pubkey,
  
  /// Destination wallet that receives every periodic charge.
  pub recipient: Pubkey,
  
  /// SPL token mint used for billing.
  /// Use `So11111111111111111111111111111111111111112` for native SOL.
  pub mint: Pubkey,
  
  /// Amount charged per billing cycle, denominated in the smallest unit
  /// of `mint` (e.g. lamports for SOL, micro-USDC for USDC).
  /// Does NOT include any platform or `ApiUser` fee.
  pub amount: u64,
  
  /// Cadence at which enrolled subscribers are billed.
  pub billing_interval: BillingInterval,
  
  /// Number of free billing intervals granted before the first real charge.
  /// `0` — no trial; billing begins immediately after `initiated_at`.
  /// `n` — subscriber receives `n` full intervals at no charge before
  ///        the first payment is collected.
  pub trial_intervals: u8,
  
  /// Optional hard cap on the number of billable cycles per subscriber.
  /// `None`  — subscription renews indefinitely.
  /// `Some`  — subscription ends automatically once `Subscriber.cycle_count`
  ///           reaches this value.
  pub max_cycles: Option<u32>,
  
  /// Maximum number of times a failed charge may be retried before the
  /// subscriber is considered delinquent and no further attempts are made.
  /// Defaults to `3`; must be in the range `[0, 10]`.
  pub max_retries: u8,
  
  /// Unix timestamp of when this policy was created.
  pub created_at: i64,
  
  /// Active state is this policy accepting new users
  pub is_active: bool,
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 16],
}

/// Tracks the state of a single subscriber enrolled in a `SubscriptionPolicy`.
///
/// State is derived from fields — no explicit status enum needed:
/// - `initiated_at: None`                              → pending; subscriber has not yet signed
/// - `initiated_at: Some` + `trial_interval_left > 0` → in trial; no charges collected yet
/// - `initiated_at: Some` + `trial_interval_left == 0`
///   + `payment_retry_count == 0`                      → active; billing normally
/// - `initiated_at: Some` + `payment_retry_count > 0` → retrying a failed charge
/// - account deleted                                   → subscription ended (cancelled,
///                                                        completed, or delinquent)
#[account]
#[derive(InitSpace)]
pub struct Subscriber {
  /// The `SubscriptionPolicy` this subscriber is enrolled in.
  pub policy: Pubkey,
  
  /// The subscriber's wallet address.
  pub subscriber: Pubkey,
  
  /// Escrow / token vault that holds the subscriber's pre-authorized funds.
  pub vault: Pubkey,
  
  /// PDA bump seed for the `vault` account.
  pub vault_bump: u8,
  
  /// Number of free billing intervals still remaining in the trial period.
  /// Copied from `SubscriptionPolicy.trial_interval` at enrollment and
  /// decremented by one each time a trial interval elapses without a charge.
  /// `0` — trial is over; normal billing applies.
  pub trial_interval_left: u8,
  
  /// Unix timestamp of when the subscriber signed and activated the subscription.
  /// `None`  — subscriber has not yet approved; account is in `Pending` state.
  /// `Some`  — subscription is active; all time-based fields are relative to this.
  pub initiated_at: Option<i64>,
  
  /// Unix timestamp of the most recent successful charge.
  /// `None`  — no charge has been collected yet (subscriber may still be in trial).
  /// `Some`  — last successful billing timestamp.
  pub last_charged_at: Option<i64>,
  
  /// Unix timestamp when the next charge attempt is due.
  /// `None`  — subscription has not started or has ended.
  /// `Some`  — scheduler / crank should attempt a charge at or after this time.
  /// Derived from `last_charged_at + billing_interval`, or
  /// `initiated_at + (trial_interval_left * billing_interval)` before the first real charge.
  pub next_charge_at: Option<i64>,
  
  /// Number of consecutive failed charge attempts for the current billing cycle.
  /// Reset to `0` after a successful charge.
  /// Once the elapsed time since `last_retry_at` exceeds `SubscriptionPolicy.max_retry_period`,
  /// no further retries are made and the subscription is considered delinquent.
  pub payment_retry_count: u8,
  
  /// Unix timestamp of the most recent charge attempt, successful or not.
  /// Used alongside `SubscriptionPolicy.max_retry_period` to determine
  /// whether the retry window has elapsed.
  pub last_retry_at: i64,
  
  /// Running count of successfully completed (paid) billing cycles.
  /// Compared against `SubscriptionPolicy.max_cycles` to detect completion.
  pub cycle_count: u32,
  
  /// PDA bump seed for this `Subscriber` account.
  pub bump: u8,
  
  /// Reserved bytes for future fields or migrations without breaking account layout.
  pub _reserved: [u8; 16],
}

// escrow can be done after this main part

/// helper structs
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeSubscriptionPolicyParams {
  pub recipient: Pubkey,
  
  pub mint: Pubkey,
  
  pub amount: u64,
  
  pub billing_interval: BillingInterval,
  
  pub trial_intervals: u8,
  
  pub max_cycles: Option<u32>,
  
  pub max_retries: u8,
}

/// helpers implementation

/// Verify is that is api_user is authorized & correct key is calling this api_user
impl ApiUser {
  pub fn verify_authority(&self, authority: &Pubkey) -> Result<()> {
    match self.authority {
      None => err!(ErrorCode::NotAuthorized),
      Some(key) => {
        require!(key == *authority, ErrorCode::AuthorityMismatch);
        Ok(())
      }
    }
  }
}