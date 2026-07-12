use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
  token::Mint,
  token_interface::{
    TokenInterface, TokenAccount,
    transfer_checked, TransferChecked,
    close_account, CloseAccount
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
)]
pub struct RefundPayoutToken<'info> {
  pub authority: Signer<'info>,
  
  #[account(
    mut,
    close = vault,
    seeds = [
      "order".as_ref(),
      maker.key().as_ref(),
      api_user.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = order.bump,
    constraint = order.token != NATIVE_SOL_MINT @ ErrorCode::InvalidTokenMint,
  )]
  pub order: Account<'info, Order>,
  
  #[account(
    mut,
    close = vault,
    seeds = [
      "expiry".as_ref(),
      order.key().as_ref(),
    ],
    bump = expiry.bump,
  )]
  pub expiry: Account<'info, Expiry>,
  
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
    close = vault,
    seeds = [
      "refund".as_ref(),
      order.key().as_ref(),
    ],
    bump = refund.bump,
  )]
  pub refund: Account<'info, Refund>,
  
  #[account(mut)]
  pub maker: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  #[account(
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref()
    ],
    bump = api_user.bump,
    constraint = api_user.is_active @ ErrorCode::ApiUserInactive,
  )]
  pub api_user: Account<'info, ApiUser>,
  
  #[account(
    constraint = mint.key() == order.token @ ErrorCode::InvalidTokenMint,
  )]
  pub mint: Account<'info, Mint>,
  
  #[account(
    mut,
    seeds = [
      "refund".as_ref(),
      "vault".as_ref(),
      refund.key().as_ref(),
    ],
    bump = refund.vault_bump.ok_or(ErrorCode::RefundVaultMissing)?,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub refund_vault: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
    mut,
    constraint = maker_token_account.owner == maker.key() @ ErrorCode::InvalidTokenAccountOwner,
    constraint = maker_token_account.mint == mint.key() @ ErrorCode::InvalidTokenMint,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub maker_token_account: InterfaceAccount<'info, TokenAccount>,
  
  pub token_program: Interface<'info, TokenInterface>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> RefundPayoutToken<'info> {
  pub fn handler(ctx: Context<RefundPayoutToken>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.refund.order_payment.is_some(),
      ErrorCode::OrderNotPaidYet
    );
    
    require!(
      ctx.accounts.refund.non_refundable_percentage < 100_000_000,
      ErrorCode::FullyNonRefundable
    );
    
    let refund_valid_until = ctx.accounts.refund
      .refund_valid_until
      .ok_or(ErrorCode::InvalidRefundConfig)?;
    
    let clock = Clock::get()?;
    
    require!(
      clock.unix_timestamp > refund_valid_until,
      ErrorCode::RefundWindowActive
    );
    
    let vault_bump = ctx.accounts.refund
      .vault_bump
      .ok_or(ErrorCode::RefundVaultMissing)?;
    
    let refund_key = ctx.accounts.refund.key();
    
    let vault_seeds: &[&[u8]] = &[
      "refund".as_ref(),
      "vault".as_ref(),
      refund_key.as_ref(),
      &[vault_bump],
    ];
    
    let vault_balance = ctx.accounts.refund_vault.amount;
    
    transfer_checked(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        TransferChecked {
          from: ctx.accounts.refund_vault.to_account_info(),
          mint: ctx.accounts.mint.to_account_info(),
          to: ctx.accounts.maker_token_account.to_account_info(),
          authority: ctx.accounts.refund_vault.to_account_info(),
        },
        &[vault_seeds],
      ),
      vault_balance,
      ctx.accounts.mint.decimals,
    )?;
    
    close_account(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        CloseAccount {
          account: ctx.accounts.refund_vault.to_account_info(),
          destination: ctx.accounts.maker_token_account.to_account_info(),
          authority: ctx.accounts.refund_vault.to_account_info(),
        },
        &[vault_seeds],
      ),
    )?;
    
    ctx.accounts.refund.vault = None;
    ctx.accounts.refund.vault_bump = None;
    
    emit_cpi!(RefundPaidOut {
      account: ctx.accounts.order.key(),
    });
    
    Ok(())
  }
}
