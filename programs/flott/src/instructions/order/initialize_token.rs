use anchor_lang::{
  prelude::*,
};
use anchor_spl::{
  token::Mint,
  token_interface::{
    TokenInterface, TokenAccount,
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
  token: Pubkey 
)]
pub struct InitializeOrderToken<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  
  pub owner: SystemAccount<'info>,
  
  pub maker: SystemAccount<'info>,
  
  #[account(
    init,
    payer = authority,
    space = 8 + Order::INIT_SPACE,
    seeds = [
      "order".as_ref(),
      maker.key().as_ref(),
      api_user.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump,
  )]
  pub order: Box< Account<'info, Order>>,
  
  #[account(
    init,
    payer = authority,
    space = 8 + Refund::INIT_SPACE,
    seeds = [
      "refund".as_ref(),
      order.key().as_ref(),
    ],
    bump,
  )]
  pub refund: Box< Account<'info, Refund>>,
  
  #[account(
    init,
    payer = authority,
    space = 8 + Expiry::INIT_SPACE,
    seeds = [
      "expiry".as_ref(),
      order.key().as_ref(),
    ],
    bump,
  )]
  pub expiry: Box< Account<'info, Expiry>>,
  
  #[account(
    init,
    payer = authority,
    space = 8 + Split::INIT_SPACE,
    seeds = [
      "split".as_ref(),
      order.key().as_ref(),
    ],
    bump,
  )]
  pub split: Box< Account<'info, Split>>,
  
  #[account(
    constraint = mint.key() == token @ ErrorCode::InvalidTokenMint,
  )]
  pub mint: Box< Account<'info, Mint>>,
  
  #[account(
    init,
    payer = authority,
    seeds = [
      "refund".as_ref(),
      "vault".as_ref(),
      refund.key().as_ref(),
    ],
    bump,
    token::mint = mint,
    token::authority = refund_token_vault,
    token::token_program = token_program,
  )]
  pub refund_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
  
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
  pub api_user: Box< Account<'info, ApiUser>>,
  
  pub token_program: Interface<'info, TokenInterface>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> InitializeOrderToken<'info> {
  pub fn handler(
    token: Pubkey,
    params: InitializeOrderParams,
    ctx: Context<InitializeOrderToken>
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(token != NATIVE_SOL_MINT, ErrorCode::InvalidTokenMint);
    
    require!(params.total_amount > 0, ErrorCode::InvalidAmount);
    
    require!(params.non_refundable_percentage <= 100_000_000, ErrorCode::InvalidSplitPercentage);
    
    if params.non_refundable_percentage < 100_000_000 {
      require!(params.refund_valid_until.is_some(), ErrorCode::InvalidRefundConfig);
    }
    
    if params.extend_authority.is_some() {
      require!(params.extended_count.is_some(), ErrorCode::InvalidExtendedCount);
    }
    
    if let Some(extended_count) = params.extended_count {
      require!(extended_count > 0, ErrorCode::InvalidExtendedCount);
    }
    
    let clock = Clock::get()?;
    
    let max_expiry_duration = 10 * 24 * 60 * 60;
    require!(params.expires_at <= clock.unix_timestamp + max_expiry_duration, ErrorCode::InvalidExpiry);
    require!(params.max_expires_at <= clock.unix_timestamp + max_expiry_duration, ErrorCode::InvalidExpiry);
    require!(params.expires_at <= params.max_expires_at, ErrorCode::InvalidExpiry);
    
    let mut percentage_sum: u32 = 0;
    let mut is_empty = true;
    
    for share_opt in params.shares.iter() {
      if percentage_sum == 100_000_000 {
        break;
      }
      if let Some(share) = share_opt {
        require!(share.percentage > 0, ErrorCode::InvalidSplitPercentage);
        
        percentage_sum = percentage_sum
          .checked_add(share.percentage)
          .ok_or(ErrorCode::MathOverflow)?;
        
        is_empty = false;
      } else {
        break;
      }
    }
    
    require!(!is_empty, ErrorCode::EmptySplits);
    
    require!(percentage_sum == 100_000_000, ErrorCode::InvalidSplitTotal);
    
    ctx.accounts.order.metadata = params.metadata;
    ctx.accounts.order.total_amount = params.total_amount;
    ctx.accounts.order.token = token;
    ctx.accounts.order.api_user = ctx.accounts.api_user.key();
    ctx.accounts.order.cuid = params.cuid;
    ctx.accounts.order.payer = params.payer;
    ctx.accounts.order.created_at = clock.unix_timestamp;
    ctx.accounts.order.bump = ctx.bumps.order;
    ctx.accounts.order._reserved = [0u8; 16];
    
    ctx.accounts.refund.order = ctx.accounts.order.key();
    ctx.accounts.refund.non_refundable_percentage = params.non_refundable_percentage;
    ctx.accounts.refund.bump = ctx.bumps.refund;
    ctx.accounts.refund.refund_valid_until = params.refund_valid_until;
    ctx.accounts.refund.order_payment = None;
    
    if params.non_refundable_percentage < 100_000_000 {
      ctx.accounts.refund.vault = Some(ctx.accounts.refund_token_vault.key());
      ctx.accounts.refund.vault_bump = Some(ctx.bumps.refund_token_vault);
    } else {
      ctx.accounts.refund.vault = None;
      ctx.accounts.refund.vault_bump = None;
    }
    
    ctx.accounts.refund._reserved = [0u8; 16];
    
    ctx.accounts.expiry.bump = ctx.bumps.expiry;
    ctx.accounts.expiry.expires_at = params.expires_at;
    ctx.accounts.expiry.extend_authority = params.extend_authority;
    ctx.accounts.expiry.extended_count = params.extended_count;
    ctx.accounts.expiry.max_expires_at = params.max_expires_at;
    ctx.accounts.expiry._reserved = [0u8; 16];
    
    ctx.accounts.split.bump = ctx.bumps.split;
    ctx.accounts.split.shares = params.shares;
    ctx.accounts.split._reserved = [0u8; 16];
    
    emit_cpi!(OrderInitialized {
      account: ctx.accounts.order.key()
    });
    
    Ok(())
  }
}