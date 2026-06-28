use anchor_lang::{
  prelude::*,
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(
  cuid: String,
  policy_cuid: String
)]
pub struct InitializeSubscriptionPolicy<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  
  pub owner: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      "vault".as_ref(),
      api_user.key().as_ref(),
    ],
    bump = api_user.vault_bump,
  )]
  pub vault: SystemAccount<'info>,
  
  #[account(
    init,
    payer = authority,
    space = 8 + SubscriptionPolicy::INIT_SPACE,
    seeds = [
      "subscription".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      policy_cuid.as_bytes(),
    ],
    bump,
  )]
  pub subscription_policy: Account<'info, SubscriptionPolicy>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref()
    ],
    bump = api_user.bump,
    constraint = api_user.is_active @ ErrorCode::ApiUserInactive,
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> InitializeSubscriptionPolicy<'info> {
  pub fn handler(
    params: InitializeSubscriptionPolicyParams,
    ctx: Context<InitializeSubscriptionPolicy>
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(params.amount > 0, ErrorCode::InvalidAmount);
    
    require!(params.max_retries <= 10, ErrorCode::InvalidMaxRetries);
    
    match params.max_cycles {
      None => {}
      Some(val) => {
        require!(val > 1, ErrorCode::InvalidMaxCycle);
        require!(val > params.trial_intervals as u32, ErrorCode::InvalidMaxCycle);
      }
    }
    
    let clock = Clock::get()?;
    
    ctx.accounts.subscription_policy.bump = ctx.bumps.subscription_policy;
    ctx.accounts.subscription_policy.authority = ctx.accounts.authority.key();
    ctx.accounts.subscription_policy.recipient = params.recipient;
    ctx.accounts.subscription_policy.mint = params.mint;
    ctx.accounts.subscription_policy.amount = params.amount;
    ctx.accounts.subscription_policy.billing_interval = params.billing_interval;
    ctx.accounts.subscription_policy.trial_intervals = params.trial_intervals;
    ctx.accounts.subscription_policy.max_cycles = params.max_cycles;
    ctx.accounts.subscription_policy.max_retries = params.max_retries;
    ctx.accounts.subscription_policy.created_at = clock.unix_timestamp;
    ctx.accounts.subscription_policy.is_active = true;
    ctx.accounts.subscription_policy._reserved = [0u8; 16];
    
    emit_cpi!(SubscriptionPolicyInitialized {
      account: ctx.accounts.subscription_policy.key()
    });
    
    Ok(())
  }
}