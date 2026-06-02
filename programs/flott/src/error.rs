use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("api_user.owner does not match the supplied owner account")]
    OwnerMismatch,
    
    #[msg("This ApiUser has already been authorized (authority is set)")]
    AlreadyAuthorized,
    
    #[msg("This ApiUser is already active")]
    AlreadyActive,
    
    #[msg("Vault balance too low — must exceed API_USER_MIN_BALANCE + API_USER_MPC_MIN_BALANCE")]
    InsufficientVaultBalance,
    
    #[msg("Deposit would not bring vault above API_USER_MIN_BALANCE")]
    InsufficientDeposit,
    
    #[msg("Arithmetic overflow")]
    Overflow,
    
    #[msg("Arithmetic underflow")]
    Underflow,
    
    #[msg("No authority is set on this ApiUser — account was never authorized")]
    NotAuthorized,
    
    #[msg("Signer does not match the authority recorded on this ApiUser")]
    AuthorityMismatch,
    
    #[msg("This ApiUser is already inactive")]
    AlreadyNotActive,
}
