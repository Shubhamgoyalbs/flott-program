use anchor_lang::{
  prelude::*,
};
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
  associated_token::AssociatedToken,
  token_interface::{
    transfer_checked,
    Mint,
    TokenAccount,
    TokenInterface,
    TransferChecked,
    close_account,
    CloseAccount
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
pub struct ClaimToken<'info> {
  
  #[account(
    constraint = server.key() == SERVER_AUTHORIZED_KEY @ ErrorCode::InvalidAuthorizeRequest
  )]
  pub server: Signer<'info>,
  
  #[account(
    mut,
    address = vesting_policy.maker @ ErrorCode::InvalidMaker,
  )]
  pub maker: SystemAccount<'info>,
  
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
  pub vesting_policy: Account<'info, VestingPolicy>,
  
  #[account(
    address = vesting_policy.token @ ErrorCode::InvalidTokenMint,
    mint::token_program = token_program,
  )]
  pub mint: InterfaceAccount<'info, Mint>,
  
  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = maker,
    associated_token::token_program = token_program,
  )]
  pub maker_ata: InterfaceAccount<'info, TokenAccount>,
  
  pub vesting_receiver: SystemAccount<'info>,
  
  #[account(
    mut,
    associated_token::mint = mint,
    associated_token::authority = vesting_receiver,
    associated_token::token_program = token_program,
  )]
  pub receiver_ata: InterfaceAccount<'info, TokenAccount>,
  
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
  pub vesting_receiver_pda: Account<'info, VestingReceiver>,
  
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
  pub vesting_vault: InterfaceAccount<'info, TokenAccount>,
  
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
  
  pub token_program: Interface<'info, TokenInterface>,
  
  pub associated_token_program: Program<'info, AssociatedToken>,
  
  pub system_program: Program<'info, System>,
}

impl<'info> ClaimToken<'info> {
  pub fn handler(
    ctx: Context<ClaimToken>, cuid: String
  ) -> Result<()> {
    
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.vesting_policy.token != NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );
    
    let clock = Clock::get()?;
    
    require!(
      !ctx.accounts.vesting_receiver_pda.started_at.is_none(),
      ErrorCode::EnrollmentMustBeActivated
    );
    
    let receiver_pda_key = ctx.accounts.vesting_receiver_pda.key();
    let receiver_key = ctx.accounts.vesting_receiver.key();
    let policy_key = ctx.accounts.vesting_policy.key();
    let vault_bump = ctx.accounts.vesting_receiver_pda.vault_bump;
    
    let pda_signer_seeds: &[&[&[u8]]] = &[&[
      "vesting".as_bytes(),
      "receiver".as_bytes(),
      receiver_key.as_ref(),
      policy_key.as_ref(),
      cuid.as_bytes(),
      &[ctx.accounts.vesting_receiver_pda.bump],
    ]];
    
    let vault_signer_seeds: &[&[&[u8]]] = &[&[
      "vesting".as_bytes(),
      "vault".as_bytes(),
      receiver_pda_key.as_ref(),
      &[vault_bump],
    ]];
    
    let tranche = ctx.accounts.vesting_policy.splits[ctx.accounts.vesting_receiver_pda.trache_to_claim as usize];
    
    match tranche {
      None => {
        let vault_amount = ctx.accounts.vesting_vault.amount;
        let pda_lamports = ctx.accounts.vesting_receiver_pda.get_lamports();
        
        if vault_amount > 0 {
          transfer_checked(
            CpiContext::new_with_signer(
              ctx.accounts.token_program.key(),
              TransferChecked {
                from: ctx.accounts.vesting_vault.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.maker_ata.to_account_info(),
                authority: ctx.accounts.vesting_vault.to_account_info(),
              },
              vault_signer_seeds
            ),
            vault_amount,
            ctx.accounts.mint.decimals,
          )?;
        }
        
        close_account(
          CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            CloseAccount {
              account: ctx.accounts.vesting_vault.to_account_info(),
              destination: ctx.accounts.maker.to_account_info(),
              authority: ctx.accounts.vesting_vault.to_account_info(),
            },
            vault_signer_seeds
          ),
        )?;
        
        transfer(
          CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            Transfer {
              from: ctx.accounts.vesting_receiver_pda.to_account_info(),
              to: ctx.accounts.maker.to_account_info(),
            },
            pda_signer_seeds,
          ),
          pda_lamports,
        )?;
        
        emit_cpi!(CompletedVesting {
          account: ctx.accounts.vesting_receiver_pda.key(),
        });
      }
      Some(split) => {
        let gap = clock.unix_timestamp - ctx.accounts.vesting_receiver_pda.started_at.unwrap();
        
        require!(gap > split.unlock_at, ErrorCode::InvalidClaim);
        
        let amount = (split.percentage as u64 * ctx.accounts.vesting_policy.total_amount) / 1_000_000;
        
        transfer_checked(
          CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
              from: ctx.accounts.vesting_vault.to_account_info(),
              mint: ctx.accounts.mint.to_account_info(),
              to: ctx.accounts.receiver_ata.to_account_info(),
              authority: ctx.accounts.vesting_vault.to_account_info(),
            },
            vault_signer_seeds
          ),
          amount,
          ctx.accounts.mint.decimals,
        )?;
        
        
        ctx.accounts.vesting_receiver_pda.claimed_amount += amount;
        ctx.accounts.vesting_receiver_pda.trache_to_claim += 1;
        
        emit_cpi!(ClaimedVestingTranche {
          account: ctx.accounts.vesting_receiver_pda.key(),
        })
      }
    }
    
    Ok(())
  }
}