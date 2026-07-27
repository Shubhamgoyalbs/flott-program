pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod event;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use event::*;

declare_id!("9vG8CUJ5Szcr7HVgMzsdzvUAauCbmyiCYTuyUAyeDnpq");

#[program]
pub mod flott {
  use super::*;

  /// Api Instructions
  
  pub fn initialize_api_user(ctx: Context<InitializeApiUser>, fee_percentage: u32) -> Result<()> {
   InitializeApiUser::handler(ctx, fee_percentage)
  }
  
  pub fn activate_api_user(ctx: Context<ActivateApiUser>) -> Result<()> {
    ActivateApiUser::handler(ctx)
  }
  
  pub fn deactivate_api_user(ctx: Context<DeactivateApiUser>) -> Result<()> {
    DeactivateApiUser::handler(ctx)
  }
  
  pub fn authorize_api_user(ctx: Context<AuthorizeApiUser>) -> Result<()> {
    AuthorizeApiUser::handler(ctx)
  }
  
  pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
    DepositToVault::handler(ctx, amount)
  }
  
  pub fn withdraw_from_vault(ctx: Context<WithdrawFromVault>, amount: u64) -> Result<()> {
    WithdrawFromVault::handler(ctx, amount)
  }
  
  pub fn close_api_account(ctx: Context<CloseApiAccount>) -> Result<()> {
    CloseApiAccount::handler(ctx)
  }
  
  pub fn authority_refill(ctx: Context<AuthorityRefill>) -> Result<()> {
    AuthorityRefill::handler(ctx)
  }
  
  pub fn withdraw_from_vault_token(ctx: Context<WithdrawFromVaultToken>, amount: u64) -> Result<()> {
    WithdrawFromVaultToken::handler(ctx, amount)
  }
  
  /// Order Instructions
  
  pub fn initialize_order(ctx: Context<InitializeOrder>, _cuid: String, params: InitializeOrderParams) -> Result<()> {
    InitializeOrder::handler(params, ctx)
  }
  
  pub fn initialize_order_token(ctx: Context<InitializeOrderToken>, _cuid: String, token: Pubkey, params: InitializeOrderParams) -> Result<()> {
    InitializeOrderToken::handler(token, params, ctx)
  }
  
  pub fn pay_order<'a>(ctx: Context<'a, PayOrder<'a>>, _cuid: String) -> Result<()> {
    PayOrder::handler(ctx)
  }
  
  pub fn pay_order_token<'a>(ctx: Context<'a, PayOrderToken<'a>>, _cuid: String) -> Result<()> {
    PayOrderToken::handler(ctx)
  }
  
  pub fn expire_order<'a>(ctx: Context<'a, ExpireOrder<'a>>, _cuid: String) -> Result<()> {
    ExpireOrder::handler(ctx)
  }
  
  pub fn extend_expiry(ctx: Context<ExtendExpiry>, _cuid: String) -> Result<()> {
    ExtendExpiry::handler(ctx)
  }
  
  pub fn refund_payout<'a>(ctx: Context<'a, RefundPayout<'a>>, _cuid: String) -> Result<()> {
    RefundPayout::handler(ctx)
  }
  
  pub fn refund_payout_token<'a>(ctx: Context<'a, RefundPayoutToken<'a>>, _cuid: String) -> Result<()> {
    RefundPayoutToken::handler(ctx)
  }
  
  /// Subscription Instructions
  
  pub fn initialize_subscription_policy(ctx: Context<InitializeSubscriptionPolicy>, _cuid: String, _policy_cuid: String, params: InitializeSubscriptionPolicyParams) -> Result<()> {
    InitializeSubscriptionPolicy::handler(params, ctx)
  }
  
  pub fn update_subscription_policy(ctx: Context<UpdateSubscriptionPolicy>, _policy_cuid: String, is_active: bool, amount: u64, trial_intervals: u8) -> Result<()> {
    UpdateSubscriptionPolicy::handler(is_active, amount, trial_intervals, ctx)
  }
  
  pub fn initialize_subscriber(ctx: Context<InitializeSubscriber>, cuid: String, _policy_cuid: String) -> Result<()> {
    InitializeSubscriber::handler(ctx, cuid)
  }
  
  pub fn initialize_subscriber_token(ctx: Context<InitializeSubscriberToken>, cuid: String, _policy_cuid: String) -> Result<()> {
    InitializeSubscriberToken::handler(ctx, cuid)
  }
  
  pub fn activate_subscription(ctx: Context<ActivateSubscription>, _cuid: String, _policy_cuid: String, amount: u64) -> Result<()> {
    ActivateSubscription::handler(ctx, amount)
  }
  
  pub fn activate_subscription_token(ctx: Context<ActivateSubscriptionToken>, _cuid: String, _policy_cuid: String, amount: u64) -> Result<()> {
    ActivateSubscriptionToken::handler(ctx, amount)
  }
  
  pub fn cancel_subscription(ctx: Context<CancelSubscription>, _cuid: String, _policy_cuid: String) -> Result<()> {
    CancelSubscription::handler(ctx)
  }
  
  pub fn cancel_subscription_token(ctx: Context<CancelToken>, _cuid: String, _policy_cuid: String) -> Result<()> {
    CancelToken::handler(ctx)
  }
  
  pub fn deposit_to_subscription_vault(ctx: Context<DepositToSubscriptionVault>, _cuid: String, _policy_cuid: String, amount: u64) -> Result<()> {
    DepositToSubscriptionVault::handler(ctx, amount)
  }
  
  pub fn deposit_to_subscription_vault_token(ctx: Context<DepositToSubscriptionVaultToken>, _cuid: String, _policy_cuid: String, amount: u64) -> Result<()> {
    DepositToSubscriptionVaultToken::handler(ctx, amount)
  }
  
  pub fn pay_subscription(ctx: Context<PayForSubscription>, _cuid: String, _policy_cuid: String) -> Result<()> {
    PayForSubscription::handler(ctx)
  }
  
  pub fn pay_subscription_token(ctx: Context<PayForSubscriptionToken>, _cuid: String, _policy_cuid: String) -> Result<()> {
    PayForSubscriptionToken::handler(ctx)
  }
  
  /// Vesting Instructions
  
  pub fn create_vesting_policy(ctx: Context<CreateVestingPolicy>, _policy_cuid: String, params: CreateVestingPolicyParams) -> Result<()> {
    CreateVestingPolicy::handler(params, ctx)
  }
  
  pub fn update_vesting_policy(ctx: Context<UpdateVestingPolicy>, _policy_cuid: String, params: UpdateVestingPolicyParams) -> Result<()> {
    UpdateVestingPolicy::handler(params, ctx)
  }
  
  pub fn cancel_vesting_policy(ctx: Context<CancelVestingPolicy>, _policy_cuid: String) -> Result<()> {
    CancelVestingPolicy::handler(ctx)
  }
  
  pub fn enroll_vesting_policy(ctx: Context<EnrollInVestingPolicy>, cuid: String, _policy_cuid: String, is_cancelable: Option<i64>) -> Result<()> {
    EnrollInVestingPolicy::handler(ctx, cuid, is_cancelable)
  }
  
  pub fn cancel_vesting_enrollment(ctx: Context<CancelVestingEnrollment>, _cuid: String, _policy_cuid: String) -> Result<()> {
    CancelVestingEnrollment::handler(ctx)
  }
  
  pub fn enroll_token(ctx: Context<EnrollToken>, cuid: String, _policy_cuid: String, is_cancelable: Option<i64>) -> Result<()> {
    EnrollToken::handler(ctx, cuid, is_cancelable)
  }
  
  pub fn dump_enrollment(ctx: Context<DumpEnrollment>, _cuid: String, _policy_cuid: String) -> Result<()> {
    DumpEnrollment::handler(ctx)
  }
  
  pub fn activate_enrollment(ctx: Context<ActivateEnrollment>, _cuid: String, _policy_cuid: String) -> Result<()> {
    ActivateEnrollment::handler(ctx)
  }
  
  pub fn claim_token(ctx: Context<ClaimToken>, _cuid: String, _policy_cuid: String) -> Result<()> {
    ClaimToken::handler(ctx, _cuid)
  }
  
  pub fn cancel_enrollment_token(ctx: Context<CancelEnrollmentToken>, _cuid: String, _policy_cuid: String) -> Result<()> {
    CancelEnrollmentToken::handler(ctx)
  }
  
  pub fn claim_vesting_tranche(ctx: Context<ClaimVestingTranche>, _cuid: String, _policy_cuid: String) -> Result<()> {
    ClaimVestingTranche::handler(ctx)
  }
  
  pub fn dump_token(ctx: Context<DumpToken>, _cuid: String, _policy_cuid: String) -> Result<()> {
    DumpToken::handler(ctx)
  }
}
