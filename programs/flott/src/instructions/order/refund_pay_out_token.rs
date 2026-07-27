use anchor_lang::prelude::*;
use anchor_spl::{
  token::Mint,
  token_interface::{
    TokenInterface, TokenAccount,
    transfer_checked, TransferChecked,
    close_account, CloseAccount,
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
  pub order: Box<Account<'info, Order>>,
  
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
    close = vault,
    seeds = [
      "refund".as_ref(),
      order.key().as_ref(),
    ],
    bump = refund.bump,
  )]
  pub refund: Box<Account<'info, Refund>>,
  
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
  
  #[account(
    constraint = mint.key() == order.token @ ErrorCode::InvalidTokenMint,
  )]
  pub mint: Box<Account<'info, Mint>>,
  
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
  pub refund_vault: Box<InterfaceAccount<'info, TokenAccount>>,
  
  #[account(
    mut,
    constraint = maker_token_account.owner == maker.key() @ ErrorCode::InvalidTokenAccountOwner,
    constraint = maker_token_account.mint == mint.key() @ ErrorCode::InvalidTokenMint,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub maker_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
  
  pub token_program: Interface<'info, TokenInterface>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> RefundPayoutToken<'info> {
  pub fn handler<'a>(ctx: Context<'a, RefundPayoutToken<'a>>) -> Result<()> {
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
    let amount = ctx.accounts.order.total_amount;
    let decimals = ctx.accounts.mint.decimals;
    let mint_key = ctx.accounts.mint.key();
    
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
      
      let token_account_data =
        InterfaceAccount::<TokenAccount>::try_from(account_info)
          .map_err(|_| ErrorCode::InvalidTokenAccount)?;
      
      require!(
        token_account_data.mint == mint_key,
        ErrorCode::InvalidTokenMint
      );
      
      let share = ctx.accounts.split.shares
        .iter()
        .find_map(|s| s.filter(|sh| sh.account == account_info.key()))
        .ok_or(ErrorCode::InvalidShareReceiver)?;
      
      let amount_to_transfer = (amount * share.percentage as u64) / 100_000_000;
      
      transfer_checked(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.key(),
          TransferChecked {
            from: ctx.accounts.refund_vault.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: account_info.to_account_info(),
            authority: ctx.accounts.refund_vault.to_account_info(),
          },
          &[vault_seeds],
        ),
        amount_to_transfer,
        decimals,
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
    
    emit_cpi!(RefundPaidOut {
      account: ctx.accounts.order.key(),
    });
    
    Ok(())
  }
}
