use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::pubkey;

#[constant]
pub const EXPIRY_EXTEND_FEE: u64 = 6000000;

#[constant]
pub const API_USER_MIN_BALANCE: u64 = 500000000;

#[constant]
pub const API_USER_MPC_INITIAL_BALANCE: u64 = 200000000;

#[constant]
pub const API_USER_MPC_MIN_BALANCE: u64 = 50000000;

#[constant]
pub const PROGRAM_FEE: u32 = 45; // 10000000 == 10%

#[constant]
pub const SERVER_AUTHORIZED_KEY: Pubkey = Pubkey::new_from_array([ 25,135,148,227,232,27,155,43,163,97,35,40,187,181,153,216,196,51,97,162,123,91,103,56,156,58,46,13,101,73,210,242]);
// this is a temp key, this is going to be changed on deployment of this program

#[constant]
pub const NATIVE_SOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
