use anchor_lang::{
  prelude::*,
  system_program::{transfer, Transfer},
};
use crate::event::ApiAccountClosed;
use crate::state::*;
use crate::error::ErrorCode;

#[event_cpi]
#[derive(Accounts)]
pub struct CloseApiAccount<'info> {
  #[account(mut)]
  pub owner: SystemAccount<'info>,
  
  pub authority: Signer<'info>,
  
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
    close = owner,
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

impl<'info> CloseApiAccount<'info> {
  pub fn handler(ctx: Context<CloseApiAccount>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    let api_user_key = ctx.accounts.api_user.key();
    
    let vault_seeds: &[&[u8]] = &[
      b"api",
      b"user",
      b"vault",
      api_user_key.as_ref(),
      &[ctx.accounts.api_user.vault_bump],
    ];
    
    transfer(
      CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.vault.to_account_info(),
          to: ctx.accounts.owner.to_account_info(),
        },
        &[vault_seeds]
      ),
      ctx.accounts.vault.lamports()
    )?;
    
    transfer(
      CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.authority.to_account_info(),
          to: ctx.accounts.owner.to_account_info(),
        },
      ),
      ctx.accounts.authority.lamports()
    )?;
    
    emit_cpi!(ApiAccountClosed {
      account: ctx.accounts.api_user.key()
    });
    
    Ok(())    
  }
}