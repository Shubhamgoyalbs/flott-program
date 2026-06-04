use anchor_lang::prelude::*;

#[constant]
pub const EXPIRY_EXTEND_FEE: u64 = 6000000;

#[constant]
pub const API_USER_MIN_BALANCE: u64 = 500000000;

#[constant]
pub const API_USER_MPC_INITIAL_BALANCE: u64 = 200000000;

#[constant]
pub const API_USER_MPC_MIN_BALANCE: u64 = 50000000;

#[constant]
pub const SERVER_AUTHORIZED_KEY: Pubkey = Pubkey::new_from_array([ 108, 224, 134, 12, 190, 36, 134, 218, 205, 126, 254, 152, 34, 35, 123, 186, 199, 211, 182, 2, 215, 95, 152, 222, 142, 32, 205, 165, 232, 1, 135, 121 ]);
// this is a temp key, this is going to be changed on deployment of this program


