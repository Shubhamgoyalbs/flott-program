use anchor_lang::prelude::*;

/// This event update the authorized field in api in db and sets the active field true
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
pub struct TransfersFundsToAuthority {
  pub account: Pubkey
}