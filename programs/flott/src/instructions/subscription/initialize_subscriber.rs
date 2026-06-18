use anchor_lang::{
  prelude::*,
};
use anchor_lang::system_program::{create_account, CreateAccount};
use crate::state::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(cuid: String)]
pub struct InitializeSubscriber<'info> {
  pub authority: Signer<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: SystemAccount<'info>,
  
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
    mut,
    seeds = [
      "subscriber".as_ref(),
      "vault".as_ref(),
      subscriber_pda.key().as_ref(),
      api_user.key().as_ref(),
    ],
    bump,
  )]
  pub subscriber_vault: SystemAccount<'info>,
  
  #[account(
    seeds = [
      "subscription".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = subscription_policy.bump,
  )]
  pub subscription_policy: Account<'info, SubscriptionPolicy>,
  
  /// CHECK: This account is manually created via CPI using CreateAccount.
  /// We use UncheckedAccount because Anchor doesn't support direct init with PDA payer.
  #[account(
      mut,
      seeds = [
        "subscriber".as_ref(),
        api_user.key().as_ref(),
        subscriber.key().as_ref(),
        cuid.as_bytes(),
      ],
      bump,
    )]
  pub subscriber_pda: UncheckedAccount<'info>,
  
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

impl<'info> InitializeSubscriber<'info> {
  pub fn handler(
    cuid: String,
    ctx: Context<InitializeSubscriber>
  ) ->  Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(ctx.accounts.subscription_policy.is_active, ErrorCode::PolicyInactive);
    
    let rent = Rent::get()?;
    
    let space = 8 + Subscriber::INIT_SPACE;
    
    let lamports = rent.minimum_balance(space);
    
    let api_user_key = ctx.accounts.api_user.key();
    
    let subscriber_key = ctx.accounts.subscriber.key();
    
    let vault_seeds: &[&[u8]] = &[
      b"api",
      b"user",
      b"vault",
      api_user_key.as_ref(),
      &[ctx.accounts.api_user.vault_bump],
    ];
    
    let subscriber_pda_seeds: &[&[u8]] = &[
      "subscription".as_ref(),
      subscriber_key.as_ref(),
      api_user_key.as_ref(),
      cuid.as_bytes(),
    ];
    
    create_account(
      CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        CreateAccount {
          from: ctx.accounts.vault.to_account_info(),
          to: ctx.accounts.subscriber_pda.to_account_info()
        },
        &[vault_seeds, subscriber_pda_seeds]
      ),
      lamports,
      space as u64,
      &crate::ID
    )?;
    
    let clock = Clock::get()?;
    
    let subscriber_data = Subscriber {
      policy: ctx.accounts.subscription_policy.key(),
      subscriber: ctx.accounts.subscriber.key(),
      vault: ctx.accounts.subscriber_vault.key(),
      vault_bump: ctx.bumps.subscriber_vault,
      trial_interval_left: ctx.accounts.subscription_policy.trial_intervals,
      initiated_at: None,
      last_charged_at: None,
      next_charge_at: None,
      payment_retry_count: 0,
      last_retry_at: None,
      cycle_count: 0,
      bump : ctx.bumps.subscriber_pda,
      created_at: clock.unix_timestamp,
      _reserved : [0u8; 16]
    };
    
    let mut data = ctx.accounts.subscriber_pda.try_borrow_mut_data()?;
    let mut writer = &mut *data;
    
    subscriber_data.try_serialize(&mut writer)?;
    
    Ok(())
  }
}