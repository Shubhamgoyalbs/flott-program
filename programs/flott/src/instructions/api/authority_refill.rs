use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;
use crate::event::*;

#[event_cpi]
#[derive(Accounts)]
pub struct AuthorityRefill<'info> {
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

impl<'info> AuthorityRefill<'info> {
  pub fn handler(ctx: Context<AuthorityRefill>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(ctx.accounts.authority.to_account_info().lamports() < API_USER_MPC_MIN_BALANCE, ErrorCode::MustFulfillRequirements);
    
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
          to:   ctx.accounts.authority.to_account_info(),
        },
        &[vault_seeds],
      ),
      100000000,
    )?;
      
    emit_cpi!(TransfersFundsToAuthority {
      account: ctx.accounts.authority.key()
    });
    
    if ctx.accounts.vault.to_account_info().lamports() < API_USER_MIN_BALANCE {
      ctx.accounts.api_user.is_active = false;
      emit_cpi!(ApiUserAccountActiveState {
        account: ctx.accounts.api_user.key(),
        is_active: false
      })
    }
    Ok(())
  }
}
