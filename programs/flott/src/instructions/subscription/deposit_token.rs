use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
  transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::NATIVE_SOL_MINT;

#[derive(Accounts)]
#[event_cpi]
#[instruction(
  cuid: String,
  policy_cuid: String
)]
pub struct DepositToSubscriptionVaultToken<'info> {
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: Signer<'info>,
  
  #[account(
    mut,
    token::mint = mint,
    token::authority = subscriber,
  )]
  pub subscriber_token_account: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
    mut,
    seeds = [
        "subscriber".as_ref(),
        "vault".as_ref(),
        subscriber_pda.key().as_ref(),
        api_user.key().as_ref(),
    ],
    bump = subscriber_pda.vault_bump,
    token::mint = mint,
    token::authority = subscriber_vault,
  )]
  pub subscriber_vault: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
      constraint = mint.key() == subscription_policy.mint @ ErrorCode::InvalidTokenMint,
  )]
  pub mint: InterfaceAccount<'info, Mint>,
  
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
    mut,
    seeds = [
      "subscriber".as_ref(),
      api_user.key().as_ref(),
      subscriber.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = subscriber_pda.bump,
    has_one = subscriber @ ErrorCode::SubscriberMismatch,
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

impl<'info> DepositToSubscriptionVaultToken<'info> {
  pub fn handler(
    ctx: Context<DepositToSubscriptionVaultToken>,
    amount: u64,
  ) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    
    require!(
      ctx.accounts.subscriber_pda.initiated_at.is_some(),
      ErrorCode::SubscriberNotInitialized
    );
    
    require!(
      ctx.accounts.subscription_policy.mint != NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );
    
    let current_balance = ctx.accounts.subscriber_vault.amount;
    let projected_balance = current_balance
      .checked_add(amount)
      .ok_or(ErrorCode::ArithmeticOverflow)?;
    
    require!(
      projected_balance >= ctx.accounts.subscription_policy.amount,
      ErrorCode::InsufficientVaultBalanceAfterDeposit
    );
    
    transfer_checked(
      CpiContext::new(
        ctx.accounts.token_program.key(),
        TransferChecked {
          from: ctx.accounts.subscriber_token_account.to_account_info(),
          mint: ctx.accounts.mint.to_account_info(),
          to: ctx.accounts.subscriber_vault.to_account_info(),
          authority: ctx.accounts.subscriber.to_account_info(),
        },
      ),
      amount,
      ctx.accounts.mint.decimals,
    )?;
    
    Ok(())
  }
}