use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;
use crate::event::ApiUserAccountActiveState;

#[event_cpi]
#[derive(Accounts)]
pub struct ActivateApiUser<'info> {
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
    bump = api_user.vault_bump
  )]
  pub vault: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref()
    ],
    bump = api_user.bump,
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub system_program: Program<'info, System>
}

impl<'info> ActivateApiUser<'info> {
  pub fn handler(ctx: Context<ActivateApiUser>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.vault.lamports() > API_USER_MIN_BALANCE,
      ErrorCode::InsufficientVaultBalance
    );
    
    require!(
      !ctx.accounts.api_user.is_active,
      ErrorCode::AlreadyActive
    );
    
    ctx.accounts.api_user.is_active = true;
    
    emit_cpi!(ApiUserAccountActiveState {
      account: ctx.accounts.api_user.key(),
      is_active: true
    });
    
    Ok(())
  }
}