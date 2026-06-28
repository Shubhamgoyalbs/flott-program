use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
  transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;
use crate::NATIVE_SOL_MINT;

#[derive(Accounts)]
#[event_cpi]
#[instruction(
  cuid: String,
  policy_cuid: String,
)]
pub struct ActivateSubscriptionToken<'info> {
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub subscriber: Signer<'info>,
  
  #[account(
    mut,
    token::mint = mint,
    token::authority = recipient,
  )]
  pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
  
  pub recipient: SystemAccount<'info>,
  
  #[account(
    mut,
    token::mint = mint,
    token::authority = subscriber,
  )]
  pub subscriber_token_account: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
    mut,
    seeds = [
      "subscriber".as_ref(),
      "vault".as_ref(),
      subscriber_pda.key().as_ref(),
      api_user.key().as_ref(),
    ],
    bump = subscriber_pda.vault_bump,
    token::mint = mint,
    token::authority = subscriber_pda,
  )]
  pub subscriber_vault: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
    constraint = mint.key() == subscription_policy.mint @ ErrorCode::InvalidTokenMint,
  )]
  pub mint: InterfaceAccount<'info, Mint>,
  
  #[account(
    seeds = [
      "subscription".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      policy_cuid.as_bytes(),
    ],
    bump = subscription_policy.bump,
    has_one = recipient @ ErrorCode::InvalidRecipient,
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
    has_one = subscriber @ ErrorCode::SubscriberMismatch,
  )]
  pub subscriber_pda: Account<'info, Subscriber>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref(),
    ],
    bump = api_user.bump,
    constraint = api_user.is_active @ ErrorCode::ApiUserInactive,
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
}

impl<'info> ActivateSubscriptionToken<'info> {
  pub fn handler(
    ctx: Context<ActivateSubscriptionToken>,
    amount: u64,
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      amount >= ctx.accounts.subscription_policy.amount,
      ErrorCode::InsufficientAmount
    );
    require!(
      ctx.accounts.subscriber_pda.initiated_at == None,
      ErrorCode::SubscriberAlreadyInitialized
    );
    require!(
      ctx.accounts.subscription_policy.mint != NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );
    
    let decimals = ctx.accounts.mint.decimals;
    
    transfer_checked(
      CpiContext::new(
        ctx.accounts.token_program.key(),
        TransferChecked {
          from: ctx.accounts.subscriber_token_account.to_account_info(),
          mint: ctx.accounts.mint.to_account_info(),
          to: ctx.accounts.subscriber_vault.to_account_info(),
          authority: ctx.accounts.subscriber.to_account_info(),
        },
      ),
      amount,
      decimals,
    )?;
    
    if ctx.accounts.subscriber_pda.trial_interval_left > 0 {
      ctx.accounts.subscriber_pda.trial_interval_left -= 1;
      
      emit_cpi!(TrialPeriodUsed {
        account: ctx.accounts.subscriber_pda.key(),
        left_cycles: ctx.accounts.subscriber_pda.trial_interval_left,
      });
    } else {
      let subscriber_pda_key = ctx.accounts.subscriber_pda.key();
      let api_user_key = ctx.accounts.api_user.key();
      
      let signer_seeds: &[&[u8]] = &[
        "subscriber".as_ref(),
        "vault".as_ref(),
        subscriber_pda_key.as_ref(),
        api_user_key.as_ref(),
        &[ctx.accounts.subscriber_pda.vault_bump],
      ];
      
      transfer_checked(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.key(),
          TransferChecked {
            from: ctx.accounts.subscriber_vault.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.subscriber_pda.to_account_info(),
          },
          &[signer_seeds],
        ),
        ctx.accounts.subscription_policy.amount,
        decimals,
      )?;
    }
    
    let clock_timestamp = Clock::get()?.unix_timestamp;
    
    ctx.accounts.subscriber_pda.initiated_at = Some(clock_timestamp);
    ctx.accounts.subscriber_pda.last_charged_at = Some(clock_timestamp);
    ctx.accounts.subscriber_pda.next_charge_at = Some(
      clock_timestamp + ctx.accounts.subscription_policy.get_interval_timestamp(),
    );
    ctx.accounts.subscriber_pda.cycle_count = 1;
    
    emit_cpi!(SubscriberActivated {
      account: ctx.accounts.subscriber_pda.key(),
    });
    
    Ok(())
  }
}