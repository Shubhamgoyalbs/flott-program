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
use crate::constants::*;

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
  pub order: Box<Account<'info, Order>>,
  
  #[account(
    constraint = server.key() == SERVER_AUTHORIZED_KEY @ ErrorCode::InvalidAuthorizeRequest
  )]
  pub server: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "refund".as_ref(),
      order.key().as_ref(),
    ],
    bump = refund.bump,
  )]
  pub refund: Box<Account<'info, Refund>>,
  
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
  pub expiry: Box<Account<'info, Expiry>>,
  
  #[account(
    mut,
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
  pub api_user_vault: SystemAccount<'info>,
  
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

impl<'info> PayOrder<'info> {
  pub fn handler<'a>(ctx: Context<'a, PayOrder<'a>>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(ctx.accounts.order.token == NATIVE_SOL_MINT, ErrorCode::InvalidTokenMint);
    
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
    
    let total_amount = ctx.accounts.order.total_amount;
    let server_fee = (total_amount * PROGRAM_FEE as u64) / 100_000_000;
    let api_fee = (total_amount * ctx.accounts.api_user.fee_percentage as u64) / 100_000_000;
    
    transfer(
      CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.payer.to_account_info(),
          to: ctx.accounts.server.to_account_info(),
        },
      ),
      server_fee,
    )?;
    
    transfer(
      CpiContext::new(
        ctx.accounts.system_program.key(),
        Transfer {
          from: ctx.accounts.payer.to_account_info(),
          to: ctx.accounts.api_user_vault.to_account_info(),
        },
      ),
      api_fee,
    )?;
    
    if ctx.accounts.refund.non_refundable_percentage < 100_000_000 {
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
      });
    } else {
      let expected_count = ctx.accounts.split.shares
        .iter()
        .filter(|s| s.is_some())
        .count();
      
      require!(
        ctx.remaining_accounts.len() == expected_count,
        ErrorCode::AccountCountMismatch
      );
      
      let mut percentage_sum: u64 = 0;
      
      for account_info in ctx.remaining_accounts.iter() {
        require!(account_info.is_writable, ErrorCode::AccountNotWritable);
        
        let share = ctx.accounts.split.shares
          .iter()
          .find_map(|s| s.filter(|sh| sh.account == account_info.key()))
          .ok_or(ErrorCode::InvalidShareReceiver)?;
        
        let amount_to_transfer = (total_amount * share.percentage as u64) / 100_000_000;
        
        transfer(
          CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
              from: ctx.accounts.payer.to_account_info(),
              to: account_info.to_account_info(),
            },
          ),
          amount_to_transfer,
        )?;

        percentage_sum = percentage_sum
          .checked_add(share.percentage as u64)
          .ok_or(ErrorCode::MathOverflow)?;
      }
      
      require!(
        percentage_sum == 100_000_000,
        ErrorCode::IncompleteSplitDistribution
      );
      
      let rent_destination = ctx.accounts.api_user_vault.to_account_info();
      
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
      
      transfer(
        CpiContext::new_with_signer(
          ctx.accounts.system_program.key(),
          Transfer {
            from: ctx.accounts.refund_vault.to_account_info(),
            to: rent_destination.clone(),
          },
          &[vault_seeds]
        ),
        ctx.accounts.refund_vault.lamports(),
      )?;
      
      ctx.accounts.split.close(rent_destination.clone())?;
      ctx.accounts.expiry.close(rent_destination.clone())?;
      ctx.accounts.refund.close(rent_destination.clone())?;
      ctx.accounts.order.close(rent_destination)?;
      
      emit_cpi!(OrderCompleted {
        account: ctx.accounts.order.key(),
      });
    }
    
    Ok(())
  }
}