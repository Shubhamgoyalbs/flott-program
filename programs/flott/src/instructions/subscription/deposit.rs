use anchor_lang::{
  prelude::*,
  system_program::{
    transfer,
    Transfer
  }
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::SubscriberActivated;

#[derive(Accounts)]
#[event_cpi]
#[instruction(cuid: String)]
pub struct DepositToSubscriptionVault<'info> {
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: Signer<'info>,
  
  #[account(
    mut,
    seeds = [
      "subscriber".as_ref(),
      "vault".as_ref(),
      subscriber_pda.key().as_ref(),
      api_user.key().as_ref(),
    ],
    bump = subscriber_pda.vault_bump,
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
  
  #[account(
    mut,
    seeds = [
      "subscriber".as_ref(),
      api_user.key().as_ref(),
      subscriber.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = subscriber_pda.bump,
    has_one = subscriber @ ErrorCode::SubscriberMismatch
  )]
  pub subscriber_pda: Account<'info, Subscriber>,
  
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

impl <'info > DepositToSubscriptionVault<'info> {
  pub fn handler(
    ctx: Context<DepositToSubscriptionVault>,
    amount: u64
  ) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    require!(
      ctx.accounts.subscriber_pda.initiated_at.is_some(),
      ErrorCode::SubscriberNotInitialized
    );
    
    let current_balance = ctx.accounts.subscriber_vault.lamports();
    
    let projected_balance = current_balance
    .checked_add(amount)
    .ok_or(ErrorCode::ArithmeticOverflow)?;
    
    require!(
      projected_balance >= ctx.accounts.subscription_policy.amount,
      ErrorCode::InsufficientVaultBalanceAfterDeposit
    );
    
    transfer(
      CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.subscriber.to_account_info(),
          to: ctx.accounts.subscriber_vault.to_account_info(),
        },
      ),
      amount,
    )?;
    
    Ok(())
  }
}