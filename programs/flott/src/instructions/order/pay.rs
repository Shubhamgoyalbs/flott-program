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
#[instruction(cuid: String)]
pub struct PayOrder<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,
  
  pub authority: SystemAccount<'info>,
  
  #[account(
    mut,
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
    seeds = [
      "refund".as_ref(),
      order.key().as_ref(),
    ],
    bump = refund.bump,
  )]
  pub refund: Account<'info, Refund>,
  
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
  
  #[account(
    mut,
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
  pub api_vault: SystemAccount<'info>,
  
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

impl<'info> PayOrder<'info> {
  pub fn handler(
    ctx: Context<PayOrder>,
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    if let Some(expected_payer) = ctx.accounts.order.payer {
      require!(
        ctx.accounts.payer.key() == expected_payer,
        ErrorCode::PayerMismatch
      );
    }
    
    let expected_vault = ctx.accounts.refund.vault.ok_or(ErrorCode::RefundVaultMissing)?;
    require!(
      ctx.accounts.refund_vault.key() == expected_vault,
      ErrorCode::InvalidVault
    );
    
    require!(
      ctx.accounts.refund.order_payment.is_none(),
      ErrorCode::OrderAlreadyPaid
    );
    
    let clock = Clock::get()?;
    
   
    
    if ctx.accounts.refund.non_refundable_percentage < 100_000_000 {
      let total_amount = ctx.accounts.order.total_amount;
      
      transfer(
        CpiContext::new(
          ctx.accounts.system_program.key(),
          Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: ctx.accounts.refund_vault.to_account_info(),
          },
        ),
        total_amount,
      )?;
      
      ctx.accounts.refund.order_payment = Some(OrderPayment {
        payer: ctx.accounts.payer.key(),
        paid_at: clock.unix_timestamp,
      });
      
      emit_cpi!(OrderPaid {
        account: ctx.accounts.order.key(),
        amount: total_amount,
      });
    } else {
      // 7. Otherwise directly pay it to the receiver thing or make it as todo
      // TODO: Implement direct payment to receiver when non_refundable_percentage is 100%
      // This will be implemented later due to some issue
      return Err(ErrorCode::NotImplemented.into());
    }
    
    Ok(())
  }
}