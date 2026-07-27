use anchor_lang::{
  prelude::*,
};
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{
    transfer_checked,
    Mint,
    TokenAccount,
    TokenInterface,
    TransferChecked,
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
  policy_cuid: String
)]
pub struct EnrollToken<'info> {
  #[account(mut)]
  pub maker: Signer<'info>,
  
  pub authority: SystemAccount<'info>,
  
  pub owner: SystemAccount<'info>,
  
  #[account(
    seeds = [
      "vesting".as_ref(),
      "policy".as_ref(),
      api_user.key().as_ref(),
      policy_cuid.as_bytes(),
    ],
    bump = vesting_policy.bump,
  )]
  pub vesting_policy: Box<Account<'info, VestingPolicy>>,
  
  #[account(
    address = vesting_policy.token @ ErrorCode::InvalidTokenMint,
    mint::token_program = token_program,
  )]
  pub mint: Box<InterfaceAccount<'info, Mint>>,
  
  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = maker,
    associated_token::token_program = token_program,
    constraint = maker_ata.amount >= vesting_policy.total_amount @ ErrorCode::InsufficientAmount,
  )]
  pub maker_ata: Box<InterfaceAccount<'info, TokenAccount>>,
  
  pub vesting_receiver: SystemAccount<'info>,
  
  #[account(
    init,
    payer = maker,
    space = VestingReceiver::INIT_SPACE + 8,
    seeds = [
      "vesting".as_ref(),
      "receiver".as_ref(),
      vesting_receiver.key().as_ref(),
      vesting_policy.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump,
  )]
  pub vesting_receiver_pda: Box<Account<'info, VestingReceiver>>,
  
  #[account(
    init,
    payer = maker,
    seeds = [
      "vesting".as_ref(),
      "vault".as_ref(),
      vesting_receiver_pda.key().as_ref(),
    ],
    bump,
    token::mint = mint,
    token::authority = vesting_vault,
    token::token_program = token_program,
  )]
  pub vesting_vault: Box<InterfaceAccount<'info, TokenAccount>>,
  
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
  pub api_user: Box<Account<'info, ApiUser>>,
  
  pub token_program: Interface<'info, TokenInterface>,
  
  pub associated_token_program: Program<'info, AssociatedToken>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> EnrollToken<'info> {
  pub fn handler(
    ctx: Context<EnrollToken>,
    cuid: String,
    is_cancelable: Option<i64>,
  ) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.vesting_policy.token != NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );
    
    require!(ctx.accounts.vesting_policy.total_amount > 0, ErrorCode::InvalidAmount);
    
    let mut is_starting = true;
    
    let clock = Clock::get()?;
    
    if let Some(cancelable) = is_cancelable {
      is_starting = false;
      
      let first_unlock_at = ctx
        .accounts
        .vesting_policy
        .splits[0]
        .ok_or(ErrorCode::EmptySplits)?
        .unlock_at;
      
      require!(cancelable > first_unlock_at, ErrorCode::InvalidCancelableDuration);
    }
    
    ctx.accounts.vesting_receiver_pda.vesting_policy = ctx.accounts.vesting_policy.key();
    ctx.accounts.vesting_receiver_pda.vault = ctx.accounts.vesting_vault.key();
    ctx.accounts.vesting_receiver_pda.vault_bump = ctx.bumps.vesting_vault;
    ctx.accounts.vesting_receiver_pda.receiver = ctx.accounts.vesting_receiver.key();
    ctx.accounts.vesting_receiver_pda.is_cancelable = is_cancelable;
    ctx.accounts.vesting_receiver_pda.cuid = cuid;
    ctx.accounts.vesting_receiver_pda.started_at = if is_starting { Some(clock.unix_timestamp) } else { None };
    ctx.accounts.vesting_receiver_pda.trache_to_claim = 0;
    ctx.accounts.vesting_receiver_pda.claimed_amount = 0;
    ctx.accounts.vesting_receiver_pda.bump = ctx.bumps.vesting_receiver_pda;
    ctx.accounts.vesting_receiver_pda.created_at = clock.unix_timestamp;
    ctx.accounts.vesting_receiver_pda._reserved = [0u8; 16];
    
    transfer_checked(
      CpiContext::new(
        ctx.accounts.token_program.key(),
        TransferChecked {
          from: ctx.accounts.maker_ata.to_account_info(),
          mint: ctx.accounts.mint.to_account_info(),
          to: ctx.accounts.vesting_vault.to_account_info(),
          authority: ctx.accounts.maker.to_account_info(),
        },
      ),
      ctx.accounts.vesting_policy.total_amount,
      ctx.accounts.mint.decimals,
    )?;
    
    emit_cpi!(EnrolledInVestingPolicy {
      account: ctx.accounts.vesting_receiver_pda.key()
    });
    
    Ok(())
  }
}