use anchor_lang::{
  prelude::*,
  system_program::{
    transfer,
    Transfer
  }
};
use crate::state::*;
use crate::constants::*;
use crate::error::ErrorCode;
use crate::event::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(
  cuid: String,
  policy_cuid: String
)]
pub struct ClaimVestingTranche<'info> {
  #[account(
    constraint = server.key() == SERVER_AUTHORIZED_KEY @ ErrorCode::InvalidAuthorizeRequest
  )]
  pub server: Signer<'info>,
  
  pub authority: SystemAccount<'info>,
  
  #[account(
    mut,
    address = vesting_policy.maker @ ErrorCode::InvalidMaker,
  )]
  pub maker: SystemAccount<'info>,
  
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
  
  pub vesting_receiver: SystemAccount<'info>,
  
  #[account(
    mut,
    seeds = [
      "vesting".as_ref(),
      "receiver".as_ref(),
      vesting_receiver.key().as_ref(),
      vesting_policy.key().as_ref(),
      cuid.as_bytes(),
    ],
    bump = vesting_receiver_pda.bump,
  )]
  pub vesting_receiver_pda: Box<Account<'info, VestingReceiver>>,
  
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
      "vesting".as_ref(),
      "vault".as_ref(),
      vesting_receiver_pda.key().as_ref(),
    ],
    bump = vesting_receiver_pda.vault_bump,
  )]
  pub vesting_vault: SystemAccount<'info>,
  
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
  
  pub system_program: Program<'info, System>,
}

impl<'info> ClaimVestingTranche<'info> {
  pub fn handler(ctx: Context<ClaimVestingTranche>) -> Result<()> {
    ctx.accounts.api_user.verify_authority(&ctx.accounts.authority.key())?;
    
    require!(
      ctx.accounts.vesting_policy.token == NATIVE_SOL_MINT,
      ErrorCode::InvalidTokenMint
    );
    
    let clock = Clock::get()?;
    
    require!(
      !ctx.accounts.vesting_receiver_pda.started_at.is_none(),
      ErrorCode::EnrollmentMustBeActivated
    );
    
    let receiver_pda_key = ctx.accounts.vesting_receiver_pda.key();
    let vault_bump = ctx.accounts.vesting_receiver_pda.vault_bump;
    
    let vault_signer_seeds: &[&[&[u8]]] = &[&[
      "vesting".as_bytes(),
      "vault".as_bytes(),
      receiver_pda_key.as_ref(),
      &[vault_bump],
    ]];
    
    let tranche = ctx.accounts.vesting_policy.splits[ctx.accounts.vesting_receiver_pda.trache_to_claim as usize];
    
    match tranche {
      None => {
        
        let vault_lamports = ctx.accounts.vesting_vault.lamports();
        
        transfer(
          CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            Transfer {
              from: ctx.accounts.vesting_vault.to_account_info(),
              to: ctx.accounts.maker.to_account_info(),
            },
            vault_signer_seeds,
          ),
          vault_lamports,
        )?;
        
        ctx.accounts.vesting_receiver_pda.close(ctx.accounts.maker.to_account_info())?;
        
        emit_cpi!(CompletedVesting {
          account: ctx.accounts.vesting_receiver_pda.key(),
        });
      }
      Some(split) => {
        
        let gap = clock.unix_timestamp - ctx.accounts.vesting_receiver_pda.started_at.unwrap();
        
        require!(gap > split.unlock_at, ErrorCode::InvalidClaim);
        
        let amount = (split.percentage as u64 * ctx.accounts.vesting_policy.total_amount) / 100_000_000;
        
        let server_fee = (amount * PROGRAM_FEE as u64) / 100_000_000;
        
        let api_fee = (amount * ctx.accounts.api_user.fee_percentage as u64) / 100_000_000;
        
        let receiving_amount = amount
          .checked_sub(server_fee)
          .unwrap()
          .checked_sub(api_fee)
          .unwrap();
        
        transfer(
          CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            Transfer {
              from: ctx.accounts.vesting_vault.to_account_info(),
              to: ctx.accounts.server.to_account_info(),
            },
            vault_signer_seeds,
          ),
          server_fee,
        )?;
        
        transfer(
          CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            Transfer {
              from: ctx.accounts.vesting_vault.to_account_info(),
              to: ctx.accounts.vault.to_account_info(),
            },
            vault_signer_seeds,
          ),
          api_fee,
        )?;
        
        transfer(
          CpiContext::new_with_signer(
            ctx.accounts.system_program.key(),
            Transfer {
              from: ctx.accounts.vesting_vault.to_account_info(),
              to: ctx.accounts.vesting_receiver.to_account_info(),
            },
            vault_signer_seeds,
          ),
          receiving_amount,
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