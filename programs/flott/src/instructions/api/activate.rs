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
  pub fn handler(&mut self, ctx: Context<ActivateApiUser>) -> Result<()> {
    self.api_user.verify_authority(&self.authority.key())?;
    
    require!(
      self.vault.lamports() > API_USER_MIN_BALANCE,
      ErrorCode::InsufficientVaultBalance
    );
    
    require!(
      !self.api_user.is_active,
      ErrorCode::AlreadyActive
    );
    
    self.api_user.is_active = false;
    
    emit_cpi!(ApiUserAccountActiveState {
      account: self.api_user.key(),
      is_active: true
    });
    
    Ok(())
  }
}