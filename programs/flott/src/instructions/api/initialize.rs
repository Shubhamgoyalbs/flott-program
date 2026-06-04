use anchor_lang::{
  prelude::*,
  system_program::{
    transfer,
    Transfer
  }
};
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct InitializeApiUser<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      "vault".as_ref(),
      owner.key().as_ref(),
      SERVER_AUTHORIZED_KEY.as_ref()
    ],
    bump
  )]
  pub vault: SystemAccount<'info>,
  
  #[account(
    init,
    payer = owner,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref()
    ],
    space = 8 + ApiUser::INIT_SPACE,
    bump
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub system_program: Program<'info, System>
}

impl <'info> InitializeApiUser<'info> {
  pub fn handler(&mut self, bumps: InitializeApiUserBumps, fee_percentage: u32) -> Result<()> {
    transfer(
      CpiContext::new(
        self.system_program.key(),
        Transfer {
          from: self.owner.to_account_info(),
          to: self.vault.to_account_info()
        }
      ),
      API_USER_MIN_BALANCE + API_USER_MPC_INITIAL_BALANCE + 5000000 // this extra amount lives vault and can be deductible
    )?;
    
    self.api_user.authority = None;
    self.api_user.vault = self.vault.key();
    self.api_user.owner = self.owner.key();
    self.api_user.bump = bumps.api_user;
    self.api_user.vault_bump = bumps.vault;
    self.api_user.fee_percentage = fee_percentage;
    self.api_user.is_active = false;
    self.api_user.created_at = Clock::get()?.unix_timestamp;
    Ok(())
  }
}