use anchor_lang::{
  prelude::*,
  system_program::{transfer, Transfer},
};
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct WithdrawFromVault<'info> {
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

impl<'info> WithdrawFromVault<'info> {
  pub fn handler(&mut self, amount: u64) -> Result<()> {
    self.api_user.verify_authority(&self.authority.key())?;
    
    let balance_after = self.vault.lamports()
      .checked_sub(amount)
      .ok_or(ErrorCode::Underflow)?;
    
    require!(
      balance_after > API_USER_MIN_BALANCE,
      ErrorCode::InsufficientDeposit
    );
    
    let owner_key     = self.owner.key();
    let vault_bump    = [self.api_user.vault_bump];
    
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
          from:   self.vault.to_account_info(),
          to: self.owner.to_account_info(),
        },
        &[vault_seeds],
      ),
      amount,
    )?;
    
    Ok(())
  }
}