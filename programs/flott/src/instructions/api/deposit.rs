use anchor_lang::{
  prelude::*,
  system_program::{transfer, Transfer},
};
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct DepositToVault<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  
  pub authority: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      "vault".as_ref(),
      api_user.key().as_ref(),
    ],
    bump = api_user.vault_bump
  )]
  pub vault: SystemAccount<'info>,
  
  #[account(
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref()
    ],
    bump = api_user.bump,
    has_one = owner @ ErrorCode::OwnerMismatch
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> DepositToVault<'info> {
  pub fn handler(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    let balance_after = ctx.accounts.vault.lamports()
      .checked_add(amount)
      .ok_or(ErrorCode::Overflow)?;
    
    require!(
      balance_after > API_USER_MIN_BALANCE,
      ErrorCode::InsufficientDeposit
    );
    
    transfer(
      CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.owner.to_account_info(),
          to:   ctx.accounts.vault.to_account_info(),
        },
      ),
      amount,
    )?;
    
    Ok(())
  }
}