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

impl<'info> AuthorizeApiUser<'info> {
  pub fn handler(ctx: Context<AuthorizeApiUser>) -> Result<()> {
    require!(
      ctx.accounts.api_user.owner == ctx.accounts.owner.key(),
      ErrorCode::OwnerMismatch
    );
    
    require!(
      ctx.accounts.api_user.authority.is_none(),
      ErrorCode::AlreadyAuthorized
    );
    
    require!(
      !ctx.accounts.api_user.is_active,
      ErrorCode::AlreadyActive
    );
    
    require!(
      ctx.accounts.vault.lamports() > API_USER_MIN_BALANCE + API_USER_MPC_INITIAL_BALANCE,
      ErrorCode::InsufficientVaultBalance
    );
    
    let api_user_key = ctx.accounts.api_user.key();
    let vault_bump = [ctx.accounts.api_user.vault_bump];
    
    let vault_seeds: &[&[u8]] = &[
      b"api",
      b"user",
      b"vault",
      api_user_key.as_ref(),
      &vault_bump,
    ];
    
    transfer(
      CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.vault.to_account_info(),
          to:   ctx.accounts.authority.to_account_info(),
        },
        &[vault_seeds],
      ),
      API_USER_MPC_INITIAL_BALANCE,
    )?;
    
    ctx.accounts.api_user.authority = Some(ctx.accounts.authority.key());
    ctx.accounts.api_user.is_active = true;
    
    emit_cpi!(ApiUserAccountGotAuthorized {
      account: ctx.accounts.api_user.key(),
      authority: ctx.accounts.authority.key()
    });
    
    Ok(())
  }
}