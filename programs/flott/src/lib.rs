pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod event;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use event::*;

declare_id!("9vG8CUJ5Szcr7HVgMzsdzvUAauCbmyiCYTuyUAyeDnpq");

#[program]
pub mod flott {
  use super::*;

  pub fn initialize_api_user(ctx: Context<InitializeApiUser>, fee_percentage: u32) -> Result<()> {
   InitializeApiUser::handler(ctx, fee_percentage)
  }
  
  pub fn activate_api_user(ctx: Context<ActivateApiUser>) -> Result<()> {
    ActivateApiUser::handler(ctx)
  }
  
  pub fn deactivate_api_user(ctx: Context<DeactivateApiUser>) -> Result<()> {
    DeactivateApiUser::handler(ctx)
  }
  
  pub fn authorize_api_user(ctx: Context<AuthorizeApiUser>) -> Result<()> {
    AuthorizeApiUser::handler(ctx)
  }
  
  pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
    DepositToVault::handler(ctx, amount)
  }
  
  pub fn withdraw_from_vault(ctx: Context<WithdrawFromVault>, amount: u64) -> Result<()> {
    WithdrawFromVault::handler(ctx, amount)
  }
}
