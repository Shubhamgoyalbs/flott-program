use anchor_lang::{
  prelude::*,
  system_program::{
    transfer,
    Transfer,
  },
  
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;

#[derive(Accounts)]
#[event_cpi]
#[instruction(cuid: String)]
pub struct PayForSubscription<'info> {
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: SystemAccount<'info>,
  
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

impl <'info > PayForSubscription<'info> {
  pub fn handler(
    ctx: Context<PayForSubscription>,
    cuid: String
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    let clock_timestamp = Clock::get()?.unix_timestamp;
    
    match ctx.accounts.subscriber_pda.next_charge_at {
      None => {
        return err!(ErrorCode::SubscriberNotInitialized)
      }
      Some(time) => {
        if time > clock_timestamp {
          return err!(ErrorCode::InvalidSchedulerRequest)
        }
      }
    }
    
    let vault_balance = ctx.accounts.subscriber_vault.lamports();
    
    let sub_pda_key = ctx.accounts.subscriber_pda.key();
    let api_user_key = ctx.accounts.api_user.key();
    
    if ctx.accounts.subscriber_pda.trial_interval_left > 0 {
      
      ctx.accounts.subscriber_pda.trial_interval_left -= 1;
      ctx.accounts.subscriber_pda.last_charged_at = Some(clock_timestamp);
      ctx.accounts.subscriber_pda.cycle_count += 1;
      ctx.accounts.subscriber_pda.next_charge_at = Some(clock_timestamp + ctx.accounts.subscription_policy.get_interval_timestamp());
      
      emit_cpi!(TrialPeriodUsed {
        account: sub_pda_key,
        left_cycles: ctx.accounts.subscriber_pda.trial_interval_left
      });
      
    } else {
      
      match ctx.accounts.subscription_policy.max_cycles {
        None => {}
        Some(cycles) => {
          if cycles <= ctx.accounts.subscriber_pda.cycle_count {
            ctx.accounts.close_account(&cuid)?;
            emit_cpi!(SubscriptionCancelled {
              account: sub_pda_key,
              reason: CancellationReason::MaxCyclesReached
            });
            
            return Ok(());
          }
        }
      }
      
      match ctx.accounts.subscriber_pda.last_retry_at {
        None => {}
        Some(_timestamp) => {
          if ctx.accounts.subscriber_pda.payment_retry_count == 0 {
            ctx.accounts.close_account(&cuid)?;
            emit_cpi!(SubscriptionCancelled {
              account: sub_pda_key,
              reason: CancellationReason::PaymentFailed
            });
            
            emit_cpi!(RemoveSubscriberRetryScheduler {
              account: sub_pda_key
            });
            
            return Ok(())
          }
        }
      }
      
      if vault_balance < ctx.accounts.subscription_policy.amount {
        match ctx.accounts.subscriber_pda.last_retry_at {
          None => {
            ctx.accounts.subscriber_pda.payment_retry_count = ctx.accounts.subscription_policy.max_retries - 1;
          }
          Some(_timestamp) => {
            ctx.accounts.subscriber_pda.payment_retry_count -= 1;
          }
        }
        
        emit_cpi!(AddRetryScheduler{
          account: sub_pda_key
        });
        
        ctx.accounts.subscriber_pda.last_retry_at = Some(clock_timestamp);
      }else {
        let vault_signer_seeds = &[
          b"subscriber".as_ref(),
          b"vault".as_ref(),
          sub_pda_key.as_ref(),
          api_user_key.as_ref(),
          &[ctx.accounts.subscriber_pda.vault_bump],
        ];
        
        transfer(
          CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            Transfer {
              from: ctx.accounts.subscriber_vault.to_account_info(),
              to: ctx.accounts.recipient.to_account_info()
            },
            &[vault_signer_seeds]
          ),
          ctx.accounts.subscription_policy.amount
        )?;
        
        emit_cpi!(PaymentSuccessfulSubscription{
          account: sub_pda_key
        });
        
        ctx.accounts.subscriber_pda.last_retry_at = None;
        ctx.accounts.subscriber_pda.cycle_count += 1;
        ctx.accounts.subscriber_pda.next_charge_at = Some(clock_timestamp + ctx.accounts.subscription_policy.get_interval_timestamp());
      }
    }
    Ok(())
  }
  
  pub fn close_account(&self, cuid: &str) -> Result<()> {
    
    let subscriber_pda = &self.subscriber_pda;
    let subscriber = &self.subscriber;
    
    // Refund remaining balance in vault + PDA rent to subscriber
    let vault_balance = self.subscriber_vault.lamports();
    let pda_balance = subscriber_pda.get_lamports();
    
    let sub_pda_key = self.subscriber_pda.key();
    let sub_key = self.subscriber.key();
    let api_user_key = self.api_user.key();
    let vault_signer_seeds = &[
      b"subscriber".as_ref(),
      b"vault".as_ref(),
      sub_pda_key.as_ref(),
      api_user_key.as_ref(),
      &[self.subscriber_pda.vault_bump],
    ];
    
    let pda_signer_seeds = &[
      b"subscriber".as_ref(),
      api_user_key.as_ref(),
      sub_key.as_ref(),
      cuid.as_bytes(),
      &[self.subscriber_pda.bump],
    ];
    
    transfer(
      CpiContext::new_with_signer(
        self.system_program.key(),
        Transfer {
          from: self.subscriber_vault.to_account_info(),
          to: self.subscriber.to_account_info(),
        },
        &[vault_signer_seeds],
      ),
      vault_balance,
    )?;
    
    transfer(
      CpiContext::new_with_signer(
        self.system_program.key(),
        Transfer {
          from: self.subscriber_pda.to_account_info(),
          to: self.subscriber.to_account_info(),
        },
        &[pda_signer_seeds],
      ),
      pda_balance
    )?;
    
    self.subscriber_pda.to_account_info().data.borrow_mut().fill(0);
    self.subscriber_pda.to_account_info().assign(&System::id());
    
    Ok(())
  }
}

