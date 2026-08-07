use anchor_lang::{
  prelude::*
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
pub struct ExpireOrder<'info> {
  #[account(
    constraint = server.key() == SERVER_AUTHORIZED_KEY @ ErrorCode::InvalidAuthorizeRequest
  )]
  pub server: Signer<'info>,
  
  pub authority: SystemAccount<'info>,
  
  #[account(
    mut,
    close = api_user_vault,
    seeds = [
      "order".as_ref(),
      maker.key().as_ref(),
      api_user.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = order.bump,
  )]
  pub order: Box< Account<'info, Order>>,
  
  #[account(
    mut,
    close = api_user_vault,
    seeds = [
      "refund".as_ref(),
      order.key().as_ref(),
    ],
    bump = refund.bump,
  )]
  pub refund: Box< Account<'info, Refund>>,
  
  #[account(
    mut,
    close = api_user_vault,
    seeds = [
      "expiry".as_ref(),
      order.key().as_ref(),
    ],
    bump = expiry.bump,
  )]
  pub expiry: Box< Account<'info, Expiry>>,
  
  #[account(
    mut,
    close = api_user_vault,
    seeds = [
      "split".as_ref(),
      order.key().as_ref(),
    ],
    bump = split.bump,
  )]
  pub split: Box< Account<'info, Split>>,
  
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
  pub api_user_vault: SystemAccount<'info>,

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
  pub api_user: Box< Account<'info, ApiUser>>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> ExpireOrder<'info> {
  pub fn handler<'a>(ctx: Context<'a, ExpireOrder<'a>>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.refund.order_payment.is_none(),
      ErrorCode::OrderAlreadyPaid
    );
    
    let clock = Clock::get()?;
    
    require!(
      clock.unix_timestamp >= ctx.accounts.expiry.expires_at,
      ErrorCode::OrderNotExpiredYet
    );
    
    emit_cpi!(OrderExpired {
      account: ctx.accounts.order.key(),
    });
    
    Ok(())
  }
}