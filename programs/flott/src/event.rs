use anchor_lang::prelude::*;
use crate::state::CancellationReason;

#[event]
pub struct ApiUserAccountGotAuthorized {
  pub account: Pubkey,
  pub authority: Pubkey
}

#[event]
pub struct ApiUserAccountActiveState {
  pub account: Pubkey,
  pub is_active: bool
}

#[event]
pub struct SubscriptionPolicyInitialized {
  pub account: Pubkey,
}

#[event]
pub struct VestingPolicyInitialized {
  pub account: Pubkey,
}

#[event]
pub struct VestingPolicyUpdated {
  pub account: Pubkey,
}

#[event]
pub struct VestingPolicyCanceled {
  pub account: Pubkey,
}

#[event]
pub struct EnrolledInVestingPolicy {
  pub account: Pubkey,
}

#[event]
pub struct TransfersFundsToAuthority {
  pub account: Pubkey
}

#[event]
pub struct SubscriptionPolicyUpdated {
  pub account: Pubkey
}

#[event]
pub struct ApiAccountClosed {
  pub account: Pubkey
}

#[event]
pub struct SubscriptionCancelled {
  pub account: Pubkey,
  pub reason: CancellationReason,
}

#[event]
pub struct TrialPeriodUsed {
  pub account: Pubkey,
  pub left_cycles: u8
}

#[event]
pub struct AddRetryScheduler {
  pub account: Pubkey,
}

#[event]
pub struct RemoveSubscriberRetryScheduler {
  pub account: Pubkey,
}

#[event]
pub struct PaymentSuccessfulSubscription {
  pub account: Pubkey,
}

#[event]
pub struct EnrollmentActivated {
  pub account: Pubkey,
}

#[event]
pub struct EnrollmentCancelled {
  pub account: Pubkey,
}

#[event]
pub struct EnrollmentDumped {
  pub account: Pubkey,
}

#[event]
pub struct SubscriberActivated {
  pub account: Pubkey
}

#[event]
pub struct CompletedVesting {
  pub account: Pubkey
}

#[event]
pub struct ClaimedVestingTranche {
  pub account: Pubkey
}

#[event]
pub struct OrderInitialized {
  pub account: Pubkey
}

#[event]
pub struct OrderExpiryExtended {
  pub account: Pubkey,
}

#[event]
pub struct RefundPaidOut {
  pub account: Pubkey,
}

#[event]
pub struct OrderPaid {
  pub account: Pubkey,
}

#[event]
pub struct OrderCompleted {
  pub account: Pubkey,
}


#[event]
pub struct OrderExpired {
  pub account: Pubkey,
}

