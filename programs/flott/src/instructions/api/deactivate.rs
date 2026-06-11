use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::ApiUserAccountActiveState;

#[event_cpi]
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
  pub fn handler(ctx: Context<DeactivateApiUser>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.api_user.is_active,
      ErrorCode::AlreadyNotActive
    );
    
    ctx.accounts.api_user.is_active = false;
    
    emit_cpi!(ApiUserAccountActiveState {
      account: ctx.accounts.api_user.key(),
      is_active: false
    });
    
    Ok(())
  }
}