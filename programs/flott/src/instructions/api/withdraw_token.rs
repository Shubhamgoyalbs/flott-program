use anchor_lang::prelude::*;
use anchor_spl::{
  token::Mint,
  token_interface::{
    TokenInterface, TokenAccount,
    transfer_checked, TransferChecked,
  },
};
use crate::state::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct WithdrawFromVaultToken<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  
  pub authority: SystemAccount<'info>,
  
  pub mint: Account<'info, Mint>,
  
  #[account(
    mut,
    seeds = [
      "api".as_ref(),
      "vault".as_ref(),
      api_user.key().as_ref(),
    ],
    bump = api_user.vault_bump,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
    mut,
    constraint = owner_token_account.owner == owner.key() @ ErrorCode::InvalidTokenAccountOwner,
    constraint = owner_token_account.mint == mint.key() @ ErrorCode::InvalidTokenMint,
    token::mint = mint,
    token::token_program = token_program,
  )]
  pub owner_token_account: InterfaceAccount<'info, TokenAccount>,
  
  #[account(
    seeds = [
      "api".as_ref(),
      "user".as_ref(),
      owner.key().as_ref(),
    ],
    bump = api_user.bump,
    has_one = owner @ ErrorCode::OwnerMismatch,
  )]
  pub api_user: Account<'info, ApiUser>,
  
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
}

impl<'info> WithdrawFromVaultToken<'info> {
  pub fn handler(ctx: Context<WithdrawFromVaultToken>, amount: u64) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(amount > 0, ErrorCode::InvalidAmount);
    
    require!(
      amount <= ctx.accounts.vault_token_account.amount,
      ErrorCode::InsufficientDeposit
    );
    
    let api_user_key = ctx.accounts.api_user.key();
    let vault_bump = [ctx.accounts.api_user.vault_bump];
    
    let vault_seeds: &[&[u8]] = &[
      b"api",
      b"vault",
      api_user_key.as_ref(),
      &vault_bump,
    ];
    
    transfer_checked(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        TransferChecked {
          from: ctx.accounts.vault_token_account.to_account_info(),
          mint: ctx.accounts.mint.to_account_info(),
          to: ctx.accounts.owner_token_account.to_account_info(),
          authority: ctx.accounts.vault_token_account.to_account_info(),
        },
        &[vault_seeds],
      ),
      amount,
      ctx.accounts.mint.decimals,
    )?;
    
    Ok(())
  }
}