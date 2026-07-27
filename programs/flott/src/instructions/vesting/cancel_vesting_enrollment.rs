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
use crate::NATIVE_SOL_MINT;

#[event_cpi]
#[derive(Accounts)]
#[instruction(
  cuid: String,
  policy_cuid: String
)]
pub struct CancelVestingEnrollment<'info> {
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  #[account(
    mut,
    address = vesting_policy.maker @ ErrorCode::InvalidMaker,
  )]
  pub maker: SystemAccount<'info>,
  
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
  
  pub cancel_authority: Signer<'info>,
  
  pub vesting_receiver: SystemAccount<'info>,
  
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
    close = maker,
  )]
  pub vesting_receiver_pda: Box<Account<'info, VestingReceiver>>,
  
  #[account(
    mut,
    seeds = [
      "vesting".as_ref(),
      "vault".as_ref(),
      vesting_receiver_pda.key().as_ref(),
    ],
    bump = vesting_receiver_pda.vault_bump,
  )]
  pub vesting_vault: SystemAccount<'info>,
  
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
  pub api_user: Box<Account<'info, ApiUser>>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> CancelVestingEnrollment<'info> {
  pub fn handler(ctx: Context<CancelVestingEnrollment>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.vesting_policy.token == NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );
    
    match ctx.accounts.vesting_policy.cancel_authority {
      None => {
        require!(ctx.accounts.maker.key() == ctx.accounts.cancel_authority.key(), ErrorCode::InvalidSigners);
      }
      Some(adr) => {
        require!(adr == ctx.accounts.cancel_authority.key(), ErrorCode::InvalidSigners);
      }
    }
    
    let clock = Clock::get()?;
    
    match ctx.accounts.vesting_receiver_pda.started_at {
      None => {
        return err!(ErrorCode::EnrollmentMustBeActivated)
      }
      Some(at) => {
        match ctx.accounts.vesting_receiver_pda.is_cancelable {
          None => return err!(ErrorCode::EnrollmentNotCancelable),
          Some(cancelable_after) => {
            require!(
              clock.unix_timestamp - at <= cancelable_after,
              ErrorCode::CancelWindowExpired
            );
          }
        }
      }
    }
    
    let vault_lamports = ctx.accounts.vesting_vault.lamports();
    
    if vault_lamports > 0 {
      let receiver_pda_key = ctx.accounts.vesting_receiver_pda.key();
      let vault_bump = ctx.accounts.vesting_receiver_pda.vault_bump;
      
      let signer_seeds: &[&[&[u8]]] = &[&[
        "vesting".as_bytes(),
        "vault".as_bytes(),
        receiver_pda_key.as_ref(),
        &[vault_bump],
      ]];
      
      transfer(
        CpiContext::new_with_signer(
          ctx.accounts.system_program.key(),
          Transfer {
            from: ctx.accounts.vesting_vault.to_account_info(),
            to: ctx.accounts.maker.to_account_info(),
          },
          signer_seeds,
        ),
        vault_lamports,
      )?;
    }
    
    emit_cpi!(EnrollmentCancelled {
      account: ctx.accounts.vesting_receiver_pda.key()
    });
    
    Ok(())
  }
}