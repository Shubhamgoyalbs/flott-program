use anchor_lang::{
  prelude::*,
  system_program::{
    transfer,
    Transfer
  }
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::SubscriptionCancelled;
use crate::NATIVE_SOL_MINT;

#[derive(Accounts)]
#[event_cpi]
#[instruction(
  cuid: String,
  policy_cuid: String,
)]
pub struct CancelSubscription<'info> {
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: Signer<'info>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      "vault".as_ref(),
      api_user.key().as_ref(),
    ],
    bump = api_user.vault_bump
  )]
  pub vault: SystemAccount<'info>,
  
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
      policy_cuid.as_bytes(),
    ],
    bump = subscription_policy.bump,
  )]
  pub subscription_policy: Account<'info, SubscriptionPolicy>,
  
  #[account(
      mut,
      close = vault,
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

impl <'info > CancelSubscription<'info> {
  pub fn handler(
    ctx: Context<CancelSubscription>
  ) -> Result<()> {
    require!(
      ctx.accounts.subscription_policy.mint == NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );
    
    let vault_balance = ctx.accounts.subscriber_vault.lamports();
    
    let sub_pda_key = ctx.accounts.subscriber_pda.key();
    let api_user_key = ctx.accounts.api_user.key();
    let signer_seeds = &[
      b"subscriber".as_ref(),
      b"vault".as_ref(),
      sub_pda_key.as_ref(),
      api_user_key.as_ref(),
      &[ctx.accounts.subscriber_pda.vault_bump],
    ];
    
    if vault_balance > 0 {
      transfer(
        CpiContext::new_with_signer(
          ctx.accounts.system_program.key(),
          Transfer {
            from: ctx.accounts.subscriber_vault.to_account_info(),
            to: ctx.accounts.subscriber.to_account_info(),
          },
          &[signer_seeds],
        ),
        vault_balance,
      )?;
    }
    
    emit_cpi!(SubscriptionCancelled {
      account: ctx.accounts.subscriber_pda.key(),
      reason: CancellationReason::BySubscriber,
    });
  
    Ok(())
  }
}