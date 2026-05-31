use anchor_lang::prelude::*;

use crate::state::*;
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
      self.api_user.is_active,
      ErrorCode::AlreadyNotActive
    );
    
    self.api_user.is_active = false;
    
    Ok(())
  }
}