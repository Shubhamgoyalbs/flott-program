use anchor_lang::{
  prelude::*
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(
  cuid: String,
  policy_cuid: String
)]
pub struct ActivateEnrollment<'info> {
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
  pub vesting_policy: Account<'info, VestingPolicy>,
  
  pub vesting_receiver: Signer<'info>,
  
  #[account(
    mut,
    seeds = [
      "vesting".as_ref(),
      "receiver".as_ref(),
      vesting_receiver.key().as_ref(),
      vesting_policy.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = vesting_receiver_pda.bump,
  )]
  pub vesting_receiver_pda: Account<'info, VestingReceiver>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref()
    ],
    bump = api_user.bump,
    constraint = api_user.is_active @ ErrorCode::ApiUserInactive,
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> ActivateEnrollment<'info> {
  pub fn handler(
    ctx: Context<ActivateEnrollment>,
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    let clock = Clock::get()?;
    
    match ctx.accounts.vesting_receiver_pda.started_at {
      None => {
        let time_gap = clock.unix_timestamp - ctx.accounts.vesting_receiver_pda.created_at;
        if time_gap > 172800 {
          return err!(ErrorCode::EnrollmentWindowExpired);
        } else {
          ctx.accounts.vesting_receiver_pda.started_at = Some(clock.unix_timestamp);
        }
      }
      Some(_) => {
        return err!(ErrorCode::EnrollmentAlreadyActivated);
      }
    }
    
    emit_cpi!(EnrollmentActivated {
      account: ctx.accounts.vesting_receiver_pda.key()
    });
    
    Ok(())
  }
}