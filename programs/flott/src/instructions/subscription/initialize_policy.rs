use anchor_lang::{
  prelude::*,
  system_program::{
    CreateAccount,
    create_account,
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
pub struct InitializeSubscriptionPolicy<'info> {
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
  
  /// CHECK: This account is manually created via CPI using CreateAccount.
  /// We use UncheckedAccount because Anchor doesn't support direct init with PDA payer.
  #[account(
    mut,
    seeds = [
      "subscription".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump,
  )]
  pub subscription_policy: UncheckedAccount<'info>,
  
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

impl<'info> InitializeSubscriptionPolicy<'info> {
  pub fn handler(
    &mut self,
    bumps: &InitializeSubscriptionPolicyBumps,
    cuid: String,
    params: InitializeSubscriptionPolicyParams,
    ctx: Context<InitializeSubscriptionPolicy>
  ) -> Result<()> {
    
    self.api_user.verify_authority(&self.authority.key())?;
    
    require!(params.amount > 0, ErrorCode::InvalidAmount);
    
    require!(params.max_retries <= 10, ErrorCode::InvalidMaxRetries);
    
    let rent = Rent::get()?;
    
    let space = 8 + SubscriptionPolicy::INIT_SPACE;
    
    let lamports = rent.minimum_balance(space);
    
    let owner_key = self.owner.key();
    
    let api_user_key = self.api_user.key();
    
    let vault_seeds: &[&[u8]] = &[
      b"api",
      b"user",
      b"vault",
      owner_key.as_ref(),
      SERVER_AUTHORIZED_KEY.as_ref(),
      &[self.api_user.vault_bump],
    ];
    
    let policy_seeds: &[&[u8]] = &[
      "subscription".as_ref(),
      "policy".as_ref(),
      api_user_key.as_ref(),
      cuid.as_bytes(),
    ];
    
    create_account(
      CpiContext::new_with_signer(
        self.system_program.key(),
        CreateAccount {
          from: self.vault.to_account_info(),
          to: self.subscription_policy.to_account_info()
        },
        &[vault_seeds, policy_seeds]
      ),
      lamports,
      space as u64,
      &crate::ID
    )?;
    
    let clock = Clock::get()?;
    
    let policy_data = SubscriptionPolicy {
      bump : bumps.subscription_policy,
      authority : self.authority.key(),
      recipient : params.recipient,
      mint : params.mint,
      amount : params.amount,
      billing_interval : params.billing_interval,
      trial_intervals : params.trial_intervals,
      max_cycles : params.max_cycles,
      max_retries : params.max_retries,
      created_at : clock.unix_timestamp,
      is_active : true,
      _reserved : [0u8; 16]
    };
    
    let mut data = ctx.accounts.subscription_policy.try_borrow_mut_data()?;
    let mut writer = &mut *data;
    
    policy_data.try_serialize(&mut writer)?;
    
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
    
    emit_cpi!(SubscriptionPolicyInitialized {
      account: self.subscription_policy.key()
    });
    
    Ok(())
  }
}