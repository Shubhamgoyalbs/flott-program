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
      api_user.key().as_ref(),
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
  pub fn handler(ctx: Context<InitializeApiUser>, fee_percentage: u32) -> Result<()> {
    transfer(
      CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.owner.to_account_info(),
          to: ctx.accounts.vault.to_account_info()
        }
      ),
      API_USER_MIN_BALANCE + API_USER_MPC_INITIAL_BALANCE + 5000000 // this extra amount lives vault and can be deductible
    )?;
    
    ctx.accounts.api_user.authority = None;
    ctx.accounts.api_user.vault = ctx.accounts.vault.key();
    ctx.accounts.api_user.owner = ctx.accounts.owner.key();
    ctx.accounts.api_user.bump = ctx.bumps.api_user;
    ctx.accounts.api_user.vault_bump = ctx.bumps.vault;
    ctx.accounts.api_user.fee_percentage = fee_percentage;
    ctx.accounts.api_user.is_active = false;
    ctx.accounts.api_user.created_at = Clock::get()?.unix_timestamp;
    Ok(())
  }
}