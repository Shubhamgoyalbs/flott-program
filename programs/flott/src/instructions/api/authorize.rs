use anchor_lang::{
  prelude::*,
  system_program::{
    transfer,
    Transfer
  },
};

use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;
use crate::event::*;

#[event_cpi]
#[derive(Accounts)]
pub struct AuthorizeApiUser<'info> {
  #[account(mut)]
  pub server: Signer<'info>,
  
  #[account(mut)]
  pub authority: SystemAccount<'info>,
  
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

impl<'info> AuthorizeApiUser<'info> {
  pub fn handler(&mut self, ctx: Context<AuthorizeApiUser>) -> Result<()> {
    require!(
      self.api_user.owner == self.owner.key(),
      ErrorCode::OwnerMismatch
    );
    
    require!(
      self.api_user.authority.is_none(),
      ErrorCode::AlreadyAuthorized
    );
    
    require!(
      !self.api_user.is_active,
      ErrorCode::AlreadyActive
    );
    
    require!(
      self.vault.lamports() > API_USER_MIN_BALANCE + API_USER_MPC_MIN_BALANCE,
      ErrorCode::InsufficientVaultBalance
    );
    
    let owner_key = self.owner.key();
    let vault_bump = [self.api_user.vault_bump];
    
    let vault_seeds: &[&[u8]] = &[
      b"api",
      b"user",
      b"vault",
      owner_key.as_ref(),
      SERVER_AUTHORIZED_KEY.as_ref(),
      &vault_bump,
    ];
    
    transfer(
      CpiContext::new_with_signer(
        self.system_program.key(),
        Transfer {
          from: self.vault.to_account_info(),
          to:   self.authority.to_account_info(),
        },
        &[vault_seeds],
      ),
      API_USER_MPC_MIN_BALANCE,
    )?;
    
    self.api_user.authority = Some(self.authority.key());
    self.api_user.is_active = true;
    
    emit_cpi!(ApiUserAccountGotAuthorized {
      account: self.api_user.key(),
      authority: self.authority.key()
    });
    
    Ok(())
  }
}