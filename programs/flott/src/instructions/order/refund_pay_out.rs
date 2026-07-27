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
use crate::constants::NATIVE_SOL_MINT;

#[event_cpi]
#[derive(Accounts)]
#[instruction(
  cuid: String,
)]
pub struct RefundPayout<'info> {
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
    constraint = order.token == NATIVE_SOL_MINT @ ErrorCode::InvalidTokenMint,
  )]
  pub order: Box<Account<'info, Order>>,
  
  #[account(
    mut,
    close = vault,
    seeds = [
      "refund".as_ref(),
      order.key().as_ref(),
    ],
    bump = refund.bump,
  )]
  pub refund: Box<Account<'info, Refund>>,
  
  #[account(
    mut,
    close = vault,
    seeds = [
      "expiry".as_ref(),
      order.key().as_ref(),
    ],
    bump = expiry.bump,
  )]
  pub expiry: Box<Account<'info, Expiry>>,
  
  #[account(
    mut,
    close = vault,
    seeds = [
      "split".as_ref(),
      order.key().as_ref(),
    ],
    bump = split.bump,
  )]
  pub split: Box<Account<'info, Split>>,
  
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
      "refund".as_ref(),
      "vault".as_ref(),
      refund.key().as_ref(),
    ],
    bump,
  )]
  pub refund_vault: SystemAccount<'info>,
  
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
  pub api_user: Box<Account<'info, ApiUser>>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> RefundPayout<'info> {
  pub fn handler<'a>(ctx: Context<'a, RefundPayout<'a>>) -> Result<()> {
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
    let signer_seeds: &[&[&[u8]]] = &[vault_seeds];
    
    let vault_balance = ctx.accounts.refund_vault.lamports();
    
    let amount = ctx.accounts.order.total_amount;
    
    let expected_count = ctx.accounts.split.shares
      .iter()
      .filter(|s| s.is_some())
      .count();
    
    require!(
      ctx.remaining_accounts.len() == expected_count,
      ErrorCode::AccountCountMismatch
    );
    
    let mut percentage_sum: u64 = 0;
    let mut total_transferred: u64 = 0;
    
    for account_info in ctx.remaining_accounts.iter() {
      require!(account_info.is_writable, ErrorCode::AccountNotWritable);
      
      let share = ctx.accounts.split.shares
        .iter()
        .find_map(|s| s.filter(|sh| sh.account == account_info.key()))
        .ok_or(ErrorCode::InvalidShareReceiver)?;
      
      let amount_to_transfer = (amount * share.percentage as u64) / 100_000_000;
      
      transfer(
        CpiContext::new_with_signer(
          ctx.accounts.system_program.key(),
          Transfer {
            from: ctx.accounts.refund_vault.to_account_info(),
            to: account_info.to_account_info(),
          },
          signer_seeds,
        ),
        amount_to_transfer,
      )?;
      
      total_transferred = total_transferred
        .checked_add(amount_to_transfer)
        .ok_or(ErrorCode::MathOverflow)?;
      percentage_sum = percentage_sum
        .checked_add(share.percentage as u64)
        .ok_or(ErrorCode::MathOverflow)?;
    }
    
    require!(
      percentage_sum == 100_000_000,
      ErrorCode::IncompleteSplitDistribution
    );
    
    require!(
      total_transferred <= vault_balance,
      ErrorCode::InsufficientVaultBalance
    );
    
    transfer(
      CpiContext::new_with_signer(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.refund_vault.to_account_info(),
          to: ctx.accounts.vault.to_account_info(),
        },
        signer_seeds,
      ),
      ctx.accounts.refund_vault.lamports(),
    )?;
    
    emit_cpi!(RefundPaidOut {
      account: ctx.accounts.order.key(),
    });
    
    Ok(())
  }
}