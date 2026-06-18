use anchor_lang::{
  prelude::*,
  system_program::{
    transfer,
    Transfer
  }
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;

#[derive(Accounts)]
#[event_cpi]
#[instruction(cuid: String)]
pub struct ActivateSubscription<'info> {
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: Signer<'info>,
  
  pub recipient: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "subscriber".as_ref(),
      "vault".as_ref(),
      subscriber_pda.key().as_ref(),
      api_user.key().as_ref(),
    ],
    bump = subscriber_pda.vault_bump,
  )]
  pub subscriber_vault: SystemAccount<'info>,
  
  #[account(
    seeds = [
      "subscription".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = subscription_policy.bump,
    has_one = recipient @ ErrorCode::InvalidRecipient
  )]
  pub subscription_policy: Account<'info, SubscriptionPolicy>,
  
  #[account(
    mut,
    seeds = [
      "subscriber".as_ref(),
      api_user.key().as_ref(),
      subscriber.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = subscriber_pda.bump,
    has_one = subscriber @ ErrorCode::SubscriberMismatch
  )]
  pub subscriber_pda: Account<'info, Subscriber>,
  
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

impl <'info > ActivateSubscription<'info> {
  pub fn handler(
    ctx: Context<ActivateSubscription>,
    amount: u64
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(amount >= ctx.accounts.subscription_policy.amount, ErrorCode::InsufficientAmount);
    require!(ctx.accounts.subscriber_pda.initiated_at == None, ErrorCode::SubscriberAlreadyInitialized);
    
    transfer(
      CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.subscriber.to_account_info(),
          to: ctx.accounts.subscriber_vault.to_account_info()
        }
      ),
      amount
    )?;
    
    if ctx.accounts.subscriber_pda.trial_interval_left > 0 {
      ctx.accounts.subscriber_pda.trial_interval_left -= 1;
      
      emit_cpi!(TrialPeriodUsed {
        account: ctx.accounts.subscriber_pda.key(),
        left_cycles: ctx.accounts.subscriber_pda.trial_interval_left
      });
    }else {
      
      let subscriber_pda_key = ctx.accounts.subscriber_pda.key();
      let api_user_key = ctx.accounts.api_user.key();
      let subscriber_vault_bump = ctx.accounts.subscriber_pda.vault_bump;
      
      let signer_seeds: &[&[u8]] = &[
        b"subscriber",
        b"vault",
        subscriber_pda_key.as_ref(),
        api_user_key.as_ref(),
        &[subscriber_vault_bump],
      ];
      
      transfer(
        CpiContext::new_with_signer(
          ctx.accounts.system_program.key(),
          Transfer {
            from: ctx.accounts.subscriber_vault.to_account_info(),
            to: ctx.accounts.recipient.to_account_info()
          },
          &[signer_seeds]
        ),
        ctx.accounts.subscription_policy.amount
      )?;
    }
    
    let clock_timestamp = Clock::get()?.unix_timestamp;
    
    ctx.accounts.subscriber_pda.initiated_at = Some(clock_timestamp);
    ctx.accounts.subscriber_pda.last_charged_at = Some(clock_timestamp);
    ctx.accounts.subscriber_pda.next_charge_at = Some(clock_timestamp + ctx.accounts.subscription_policy.get_interval_timestamp());
    ctx.accounts.subscriber_pda.cycle_count = 1;
    
    emit_cpi!(SubscriberActivated {
      account: ctx.accounts.subscriber_pda.key()
    });
    Ok(())
  }
}