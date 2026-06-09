use anchor_lang::{
  prelude::*,
  system_program::{
    Transfer,
    transfer
  }
};

use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;
use crate::event::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(cuid: String)]
pub struct UpdateSubscriptionPolicy<'info> {
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
    bump = api_user.vault_bump,
  )]
  pub vault: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "subscription".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = subscription_policy.bump,
  )]
  pub subscription_policy: Account<'info, SubscriptionPolicy>,
  
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

impl<'info> UpdateSubscriptionPolicy<'info> {
  pub fn handler(
    &mut self,
    is_active: bool,
    amount: u64,
    trial_intervals: u8,
    ctx: Context<UpdateSubscriptionPolicy>
  ) -> Result<()> {
    self.api_user.verify_authority(&self.authority.key())?;
    
    require!(amount > 0, ErrorCode::InvalidAmount);
    
    let api_user_key = self.api_user.key();
    
    let vault_seeds: &[&[u8]] = &[
      b"api",
      b"user",
      b"vault",
      api_user_key.as_ref(),
      &[self.api_user.vault_bump],
    ];
    
    self.subscription_policy.amount = amount;
    self.subscription_policy.is_active = is_active;
    self.subscription_policy.trial_intervals = trial_intervals;
    
    if self.authority.to_account_info().lamports() < API_USER_MPC_MIN_BALANCE {
      transfer(
        CpiContext::new_with_signer(
          self.system_program.key(),
          Transfer {
            from: self.vault.to_account_info(),
            to:   self.authority.to_account_info(),
          },
          &[vault_seeds],
        ),
        100000000,
      )?;
      
      emit_cpi!(TransfersFundsToAuthority {
        account: self.authority.key()
      })
    }
    
    if self.vault.to_account_info().lamports() < API_USER_MIN_BALANCE {
      self.api_user.is_active = false;
      
      emit_cpi!(ApiUserAccountActiveState {
        account: self.api_user.key(),
        is_active: false
      })
    }
    
    emit_cpi!(SubscriptionPolicyUpdated {
      account: self.subscription_policy.key()
    });
    Ok(())
  }
}