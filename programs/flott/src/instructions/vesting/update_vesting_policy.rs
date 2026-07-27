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
pub struct UpdateVestingPolicy<'info> {
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

impl<'info> UpdateVestingPolicy<'info> {
  pub fn handler(
    mut params: UpdateVestingPolicyParams,
    ctx: Context<UpdateVestingPolicy>
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(ctx.accounts.vesting_policy.receiver_count == 0, ErrorCode::InvalidRequest);
    require!(params.total_amount > 0, ErrorCode::InvalidAmount);
    
    let clock = Clock::get()?;
    
    
    if let Some(cliff_duration) = params.cliff_duration {
      require!(cliff_duration > 0, ErrorCode::InvalidCliffDuration);
    }
    
    let mut percentage_sum: u32 = 0;
    let mut is_empty = true;
    let mut previous_time = 0;
    
    if let Some(time) = params.cliff_duration {
      previous_time = time;
    }
    
    for split_opt in params.splits.iter_mut() {
      if let Some(split) = split_opt {
        require!(split.percentage > 0, ErrorCode::InvalidSplitPercentage);
        require!(split.unlock_at > 0, ErrorCode::InvalidSplitPercentage);
        
        split.unlock_at = split
          .unlock_at
          .checked_add(previous_time)
          .ok_or(ErrorCode::MathOverflow)?;
        
        previous_time = split.unlock_at;
        
        percentage_sum = percentage_sum
          .checked_add(split.percentage)
          .ok_or(ErrorCode::MathOverflow)?;
        
        is_empty = false;
        
      } else {
        break;
      }
    }
    
    require!(!is_empty, ErrorCode::EmptySplits);
    
    require!(percentage_sum == 100_000_000, ErrorCode::InvalidSplitTotal);
    
    ctx.accounts.vesting_policy.total_amount = params.total_amount;
    ctx.accounts.vesting_policy.splits = params.splits;
    ctx.accounts.vesting_policy.cliff_duration = params.cliff_duration;
    ctx.accounts.vesting_policy.updated_at = Some(clock.unix_timestamp);
    
    emit_cpi!(VestingPolicyUpdated {
      account: ctx.accounts.vesting_policy.key()
    });
    
    Ok(())
  }
}