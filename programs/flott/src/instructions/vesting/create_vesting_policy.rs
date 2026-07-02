use anchor_lang::{
  prelude::*,
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
pub struct CreateVestingPolicy<'info> {
  pub maker: SystemAccount<'info>,
  
  #[account(mut)]
  pub authority: Signer<'info>,
  
  pub owner: SystemAccount<'info>,
  
  #[account(
    init,
    payer = authority,
    space = 8 + VestingPolicy::INIT_SPACE,
    seeds = [
      "vesting".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      policy_cuid.as_bytes(),
    ],
    bump,
  )]
  pub vesting_policy: Account<'info, VestingPolicy>,
  
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

impl<'info> CreateVestingPolicy<'info> {
  pub fn handler(
    mut params: CreateVestingPolicyParams,
    ctx: Context<CreateVestingPolicy>
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(params.total_amount > 0, ErrorCode::InvalidAmount);
    
    require_keys_neq!(params.update_authority, Pubkey::default(), ErrorCode::InvalidUpdateAuthority);
    
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
    
    ctx.accounts.vesting_policy.bump = ctx.bumps.vesting_policy;
    ctx.accounts.vesting_policy.maker = ctx.accounts.maker.key();
    ctx.accounts.vesting_policy.api_user = ctx.accounts.api_user.key();
    ctx.accounts.vesting_policy.token = params.token;
    ctx.accounts.vesting_policy.total_amount = params.total_amount;
    ctx.accounts.vesting_policy.splits = params.splits;
    ctx.accounts.vesting_policy.cliff_duration = params.cliff_duration;
    ctx.accounts.vesting_policy.receiver_count = 0;
    ctx.accounts.vesting_policy.update_authority = params.update_authority;
    ctx.accounts.vesting_policy.cancel_authority = params.cancel_authority;
    ctx.accounts.vesting_policy.cancelled_at = None;
    ctx.accounts.vesting_policy.created_at = clock.unix_timestamp;
    ctx.accounts.vesting_policy.updated_at = None;
    ctx.accounts.vesting_policy._reserved = [0u8; 32];
    
    emit_cpi!(VestingPolicyInitialized {
      account: ctx.accounts.vesting_policy.key()
    });
    
    Ok(())
  }
}