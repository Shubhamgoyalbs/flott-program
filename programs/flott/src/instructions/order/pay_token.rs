use anchor_lang::prelude::*;
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
#[instruction(cuid: String)]
pub struct PayOrderToken<'info> {
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
    constraint = order.token != NATIVE_SOL_MINT @ ErrorCode::InvalidTokenMint,
    constraint = order.token == mint.key() @ ErrorCode::InvalidTokenMint,
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
    bump = refund.vault_bump.ok_or(ErrorCode::RefundVaultMissing)?,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub refund_vault: Box<InterfaceAccount<'info, TokenAccount>>,
  
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
  pub api_vault: SystemAccount<'info>,
  
  #[account(
    mut,
    constraint = api_vault_token_account.owner == api_vault.key() @ ErrorCode::InvalidTokenAccountOwner,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub api_vault_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
  
  #[account(
    mut,
    constraint = server_token_account.owner == server.key() @ ErrorCode::InvalidTokenAccountOwner,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub server_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
  
  #[account(
    mut,
    constraint = payer_token_account.owner == payer.key() @ ErrorCode::InvalidTokenAccountOwner,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub payer_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
  
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
  
  pub token_program: Interface<'info, TokenInterface>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> PayOrderToken<'info> {
  pub fn handler<'a>(ctx: Context<'a, PayOrderToken<'a>>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(ctx.accounts.order.token != NATIVE_SOL_MINT, ErrorCode::InvalidTokenMint);
    
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
    let decimals = ctx.accounts.mint.decimals;
    let server_fee = (total_amount * PROGRAM_FEE as u64) / 100_000_000;
    let api_fee = (total_amount * ctx.accounts.api_user.fee_percentage as u64) / 100_000_000;
    
    transfer_checked(
      CpiContext::new(
        ctx.accounts.token_program.key(),
        TransferChecked {
          from: ctx.accounts.payer_token_account.to_account_info(),
          mint: ctx.accounts.mint.to_account_info(),
          to: ctx.accounts.server_token_account.to_account_info(),
          authority: ctx.accounts.payer.to_account_info(),
        },
      ),
      server_fee,
      decimals,
    )?;
    
    transfer_checked(
      CpiContext::new(
        ctx.accounts.token_program.key(),
        TransferChecked {
          from: ctx.accounts.payer_token_account.to_account_info(),
          mint: ctx.accounts.mint.to_account_info(),
          to: ctx.accounts.api_vault_token_account.to_account_info(),
          authority: ctx.accounts.payer.to_account_info(),
        },
      ),
      api_fee,
      decimals,
    )?;
    
    if ctx.accounts.refund.non_refundable_percentage < 100_000_000 {
      transfer_checked(
        CpiContext::new(
          ctx.accounts.token_program.key(),
          TransferChecked {
            from: ctx.accounts.payer_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.refund_vault.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
          },
        ),
        total_amount,
        decimals,
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
        
        let token_account_data =
          InterfaceAccount::<TokenAccount>::try_from(account_info)
            .map_err(|_| ErrorCode::InvalidTokenAccount)?;
        
        require!(
          token_account_data.mint == ctx.accounts.mint.key(),
          ErrorCode::InvalidTokenMint
        );
        
        let share = ctx.accounts.split.shares
          .iter()
          .find_map(|s| s.filter(|sh| sh.account == account_info.key()))
          .ok_or(ErrorCode::InvalidShareReceiver)?;
        
        let amount_to_transfer = (total_amount * share.percentage as u64) / 100_000_000;
        
        transfer_checked(
          CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
              from: ctx.accounts.payer_token_account.to_account_info(),
              mint: ctx.accounts.mint.to_account_info(),
              to: account_info.to_account_info(),
              authority: ctx.accounts.payer.to_account_info(),
            },
          ),
          amount_to_transfer,
          decimals,
        )?;
        
        percentage_sum = percentage_sum
          .checked_add(share.percentage as u64)
          .ok_or(ErrorCode::MathOverflow)?;
      }
      
      require!(
        percentage_sum == 100_000_000,
        ErrorCode::IncompleteSplitDistribution
      );
      
      let rent_destination = ctx.accounts.api_vault.to_account_info();
      
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
      
      close_account(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.key(),
          CloseAccount {
            account: ctx.accounts.refund_vault.to_account_info(),
            destination: rent_destination.clone(),
            authority: ctx.accounts.refund_vault.to_account_info(),
          },
          &[vault_seeds],
        ),
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