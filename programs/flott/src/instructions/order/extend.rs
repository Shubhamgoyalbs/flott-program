use anchor_lang::{
  prelude::*,
};
use anchor_lang::system_program::{transfer, Transfer};
use crate::state::*;
use crate::error::ErrorCode;
use crate::event::*;
use crate::constants::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(cuid: String)]
pub struct ExtendExpiry<'info> {
  #[account(mut)]
  pub extend_authority: Signer<'info>,
  
  #[account(
    mut,
    seeds = [
      "order".as_ref(),
      maker.key().as_ref(),
      api_user.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = order.bump,
  )]
  pub order: Account<'info, Order>,
  
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
      "expiry".as_ref(),
      order.key().as_ref(),
    ],
    bump = expiry.bump,
  )]
  pub expiry: Account<'info, Expiry>,
  
  pub maker: SystemAccount<'info>,
  
  pub authority: SystemAccount<'info>,
  
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
  
  pub owner: SystemAccount<'info>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> ExtendExpiry<'info> {
  pub fn handler(
    ctx: Context<ExtendExpiry>,
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    let clock = Clock::get()?;
    
    require!(
      ctx.accounts.extend_authority.key() == ctx.accounts.expiry.extend_authority.unwrap(),
      ErrorCode::AuthorityMismatch
    );
    
    let current_extended_count = ctx.accounts.expiry.extended_count.ok_or(ErrorCode::ExtendLimitReached)?;
    
    require!(current_extended_count > 0, ErrorCode::ExtendLimitReached);
    
    let one_hour = 60 * 60;
    let time_until_expiry = ctx.accounts.expiry.expires_at.checked_sub(clock.unix_timestamp).ok_or(ErrorCode::InvalidExpiry)?;
    
    require!(time_until_expiry >= one_hour, ErrorCode::ExtendTooSoon);
    
    let extension_duration = 12 * 60 * 60;
    let new_expires_at = ctx.accounts.expiry.expires_at.checked_add(extension_duration).ok_or(ErrorCode::MathOverflow)?;
    
    require!(new_expires_at <= ctx.accounts.expiry.max_expires_at, ErrorCode::MaxExpiryExceeded);
    
    let extend_fee = (ctx.accounts.order.total_amount * EXPIRY_EXTEND_FEE) / 100_000_000;
    
    transfer(
      CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.extend_authority.to_account_info(),
          to: ctx.accounts.vault.to_account_info(),
        },
      ),
      extend_fee,
    )?;
    
    ctx.accounts.expiry.expires_at = new_expires_at;
    ctx.accounts.expiry.extended_count = Some(current_extended_count.checked_sub(1).ok_or(ErrorCode::MathOverflow)?);
    
    emit_cpi!(OrderExpiryExtended {
      account: ctx.accounts.order.key(),
    });
    
    Ok(())
  }
}