use anchor_lang::prelude::*;
use anchor_spl::{
  token::{
    Mint
  },
  token_interface::{TokenInterface, TokenAccount},
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::NATIVE_SOL_MINT;

#[derive(Accounts)]
#[instruction(
  cuid: String,
  policy_cuid: String
)]
pub struct InitializeSubscriberToken<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: SystemAccount<'info>,
  
  #[account(
    constraint = mint.key() == subscription_policy.mint @ ErrorCode::InvalidTokenMint,
    constraint = mint.key() != NATIVE_SOL_MINT @ ErrorCode::InvalidTokenMint,
  )]
  pub mint: Account<'info, Mint>,
  
  #[account(
    init,
    payer = authority,
    token::mint = mint,
    token::authority = subscriber_vault,
    token::token_program = token_program,
    seeds = [
      "subscriber".as_ref(),
      "vault".as_ref(),
      subscriber_pda.key().as_ref(),
      api_user.key().as_ref(),
    ],
    bump,
  )]
  pub subscriber_vault: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
    seeds = [
      "subscription".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      policy_cuid.as_bytes(),
    ],
    bump = subscription_policy.bump,
  )]
  pub subscription_policy: Account<'info, SubscriptionPolicy>,
  
  #[account(
    init,
    payer = authority,
    space = 8 + Subscriber::INIT_SPACE,
    seeds = [
      "subscriber".as_ref(),
      api_user.key().as_ref(),
      subscriber.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump,
  )]
  pub subscriber_pda: Account<'info, Subscriber>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref(),
    ],
    bump = api_user.bump,
    constraint = api_user.is_active @ ErrorCode::ApiUserInactive,
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitializeSubscriberToken<'info> {
  pub fn handler(ctx: Context<InitializeSubscriberToken>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(ctx.accounts.subscription_policy.is_active, ErrorCode::PolicyInactive);
    
    require!(
      ctx.accounts.subscription_policy.mint != NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );

    let clock = Clock::get()?;
    
    ctx.accounts.subscriber_pda.policy = ctx.accounts.subscription_policy.key();
    ctx.accounts.subscriber_pda.subscriber = ctx.accounts.subscriber.key();
    ctx.accounts.subscriber_pda.vault = ctx.accounts.subscriber_vault.key();
    ctx.accounts.subscriber_pda.vault_bump = ctx.bumps.subscriber_vault;
    ctx.accounts.subscriber_pda.trial_interval_left = ctx.accounts.subscription_policy.trial_intervals;
    ctx.accounts.subscriber_pda.initiated_at = None;
    ctx.accounts.subscriber_pda.last_charged_at = None;
    ctx.accounts.subscriber_pda.next_charge_at = None;
    ctx.accounts.subscriber_pda.payment_retry_count = 0;
    ctx.accounts.subscriber_pda.last_retry_at = None;
    ctx.accounts.subscriber_pda.cycle_count = 0;
    ctx.accounts.subscriber_pda.bump = ctx.bumps.subscriber_pda;
    ctx.accounts.subscriber_pda.created_at = clock.unix_timestamp;
    ctx.accounts.subscriber_pda._reserved = [0u8; 16];
    
    Ok(())
  }
}