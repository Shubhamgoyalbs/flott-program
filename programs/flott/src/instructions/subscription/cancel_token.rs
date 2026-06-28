use anchor_lang::prelude::*;
use anchor_spl::{
  token_interface::{
    TokenAccount, TokenInterface,
    close_account, CloseAccount,
    transfer_checked, TransferChecked,
  },
  token::Mint,
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;
use crate::NATIVE_SOL_MINT;

#[derive(Accounts)]
#[event_cpi]
#[instruction(
  cuid: String,
  policy_cuid: String,
)]
pub struct CancelToken<'info> {
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: Signer<'info>,
  
  #[account(
    constraint = mint.key() == subscription_policy.mint @ ErrorCode::InvalidTokenMint,
  )]
  pub mint: Account<'info, Mint>,
  
  #[account(
    mut,
    token::mint = mint,
    token::authority = subscriber,
    token::token_program = token_program,
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
    token::token_program = token_program,
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
    mut,
    seeds = [
      "api".as_ref(),
      "vault".as_ref(),
      api_user.key().as_ref(),
    ],
    bump = api_user.vault_bump,
  )]
  pub vault: SystemAccount<'info>,
  
  #[account(
    mut,
    close = vault,
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

impl<'info> CancelToken<'info> {
  pub fn handler(ctx: Context<CancelToken>) -> Result<()> {
    require!(
      ctx.accounts.subscription_policy.mint != NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );
    
    let subscriber_pda_key = ctx.accounts.subscriber_pda.key();
    let api_user_key = ctx.accounts.api_user.key();
    let signer_seeds: &[&[u8]] = &[
      b"subscriber",
      b"vault",
      subscriber_pda_key.as_ref(),
      api_user_key.as_ref(),
      &[ctx.accounts.subscriber_pda.vault_bump],
    ];
    
    if ctx.accounts.subscriber_vault.amount > 0 {
      transfer_checked(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.key(),
          TransferChecked {
            from: ctx.accounts.subscriber_vault.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.subscriber_token_account.to_account_info(),
            authority: ctx.accounts.subscriber_vault.to_account_info(),
          },
          &[signer_seeds],
        ),
        ctx.accounts.subscriber_vault.amount,
        ctx.accounts.mint.decimals,
      )?;
    }
    
    close_account(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        CloseAccount {
          account: ctx.accounts.subscriber_vault.to_account_info(),
          destination: ctx.accounts.subscriber.to_account_info(),
          authority: ctx.accounts.subscriber_vault.to_account_info(),
        },
        &[signer_seeds],
      ),
    )?;
    
    emit_cpi!(SubscriptionCancelled {
      account: ctx.accounts.subscriber_pda.key(),
      reason: CancellationReason::BySubscriber,
    });
    
    Ok(())
  }
}