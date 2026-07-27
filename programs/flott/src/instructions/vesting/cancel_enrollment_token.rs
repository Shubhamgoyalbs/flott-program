use anchor_lang::{
  prelude::*,
};
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{
    transfer_checked,
    Mint,
    TokenAccount,
    TokenInterface,
    TransferChecked,
    close_account,
    CloseAccount
  },
};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;
use crate::constants::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(
  cuid: String,
  policy_cuid: String
)]
pub struct CancelEnrollmentToken<'info> {
  #[account(
    mut,
    address = vesting_policy.maker @ ErrorCode::InvalidMaker,
  )]
  pub maker: SystemAccount<'info>,
  
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
    address = vesting_policy.token @ ErrorCode::InvalidTokenMint,
    mint::token_program = token_program,
  )]
  pub mint: Box<InterfaceAccount<'info, Mint>>,
  
  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = maker,
    associated_token::token_program = token_program,
  )]
  pub maker_ata: Box<InterfaceAccount<'info, TokenAccount>>,
  
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
    seeds = [
      "vesting".as_ref(),
      "vault".as_ref(),
      vesting_receiver_pda.key().as_ref(),
    ],
    bump = vesting_receiver_pda.vault_bump,
    token::mint = mint,
    token::authority = vesting_vault,
    token::token_program = token_program,
  )]
  pub vesting_vault: Box<InterfaceAccount<'info, TokenAccount>>,
  
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
  
  pub cancel_authority: Signer<'info>,
  
  pub token_program: Interface<'info, TokenInterface>,
  
  pub associated_token_program: Program<'info, AssociatedToken>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> CancelEnrollmentToken<'info> {
  pub fn handler(
    ctx: Context<CancelEnrollmentToken>,
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.vesting_policy.token != NATIVE_SOL_MINT,
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
    
    let vault_amount = ctx.accounts.vesting_vault.amount;
    
    let receiver_pda_key = ctx.accounts.vesting_receiver_pda.key();
    let vault_bump = ctx.accounts.vesting_receiver_pda.vault_bump;
    
    let signer_seeds: &[&[&[u8]]] = &[&[
      "vesting".as_bytes(),
      "vault".as_bytes(),
      receiver_pda_key.as_ref(),
      &[vault_bump],
    ]];
    
    if vault_amount > 0 {
      transfer_checked(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.key(),
          TransferChecked {
            from: ctx.accounts.vesting_vault.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.maker_ata.to_account_info(),
            authority: ctx.accounts.vesting_vault.to_account_info(),
          },
          signer_seeds
        ),
        vault_amount,
        ctx.accounts.mint.decimals,
      )?;
    }
    
    close_account(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        CloseAccount {
          account: ctx.accounts.vesting_vault.to_account_info(),
          destination: ctx.accounts.maker.to_account_info(),
          authority: ctx.accounts.vesting_vault.to_account_info(),
        },
        signer_seeds
      ),
    )?;
    
    emit_cpi!(EnrollmentCancelled {
      account: ctx.accounts.vesting_receiver_pda.key()
    });
    
    Ok(())
  }
}