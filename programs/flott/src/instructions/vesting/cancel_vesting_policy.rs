use anchor_lang::{
  prelude::*,
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(
  policy_cuid: String
)]
pub struct CancelVestingPolicy<'info> {
  #[account(mut)]
  pub maker: Signer<'info>,
  
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  #[account(
    seeds = [
      "vesting".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      policy_cuid.as_bytes(),
    ],
    bump = vesting_policy.bump,
  )]
  pub vesting_policy: Box<Account<'info, VestingPolicy>>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      "vault".as_ref(),
      api_user.key().as_ref(),
    ],
    bump = api_user.vault_bump,
  )]
  pub vault: SystemAccount<'info>,
  
  #[account(
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref()
    ],
    bump = api_user.bump,
    constraint = api_user.is_active @ ErrorCode::ApiUserInactive,
  )]
  pub api_user: Box<Account<'info, ApiUser>>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> CancelVestingPolicy<'info> {
  pub fn handler(
    ctx: Context<CancelVestingPolicy>
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(ctx.accounts.vesting_policy.receiver_count == 0, ErrorCode::InvalidRequest);
    
    emit_cpi!(VestingPolicyCanceled {
      account: ctx.accounts.vesting_policy.key()
    });
    
    Ok(())
  }
}