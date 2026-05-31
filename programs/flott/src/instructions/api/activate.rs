use anchor_lang::prelude::*;

use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct DeactivateApiUser<'info> {
  pub authority: Signer<'info>,
  
  pub owner: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      "vault".as_ref(),
      owner.key().as_ref(),
      SERVER_AUTHORIZED_KEY.as_ref()
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

impl<'info> DeactivateApiUser<'info> {
  pub fn handler(&mut self) -> Result<()> {
    self.api_user.verify_authority(&self.authority.key())?;
    
    require!(
      self.vault.lamports() > API_USER_MIN_BALANCE,
      ErrorCode::InsufficientVaultBalance
    );
    
    require!(
      !self.api_user.authority.is_none(),
      ErrorCode::NotAuthorized
    );
    
    require!(
      !self.api_user.is_active,
      ErrorCode::AlreadyActive
    );
    
    self.api_user.is_active = false;
    
    Ok(())
  }
}