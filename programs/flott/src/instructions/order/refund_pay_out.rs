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
  pub order: Account<'info, Order>,
  
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
    close = vault,
    seeds = [
      "split".as_ref(),
      order.key().as_ref(),
    ],
    bump = split.bump,
  )]
  pub split: Account<'info, Split>,
  
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
  pub api_user: Account<'info, ApiUser>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> RefundPayout<'info> {
  pub fn handler(ctx: Context<RefundPayout>) -> Result<()> {
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
    
    let mut percentage_sum = 0;
    let mut index: usize = 0;
    let amount = ctx.accounts.order.total_amount;
    
    for account_info in ctx.remaining_accounts.iter() {
      require!(account_info.is_writable, ErrorCode::AccountNotWritable);
      require!(ctx.accounts.split.shares[index].is_some(), ErrorCode::InvalidSplitPercentage);
      
      let share = ctx.accounts.split.shares[index].unwrap();
      
      require!(share.account == account_info.key(), ErrorCode::InvalidShareReceiver);
      
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
      
      index += 1;
      percentage_sum += share.percentage;
      
      if percentage_sum == 100_000_000 {
        break;
      }
    }
    
    ctx.accounts.refund.vault = None;
    ctx.accounts.refund.vault_bump = None;
    
    emit_cpi!(RefundPaidOut {
      account: ctx.accounts.order.key(),
    });
    
    Ok(())
  }
}