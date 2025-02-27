mod prediction_market;
mod user_prediction;

use solana_program::native_token::LAMPORTS_PER_SOL;
pub use prediction_market::*;
pub use user_prediction::*;

pub const UNINITIALIZED_VERSION: u8 = 0;
pub const PROGRAM_VERSION: u8 = 1;

pub const CREATE_MARKET_FEE: u64 = LAMPORTS_PER_SOL / 10;

pub const VOTE_PRICE: u64 = LAMPORTS_PER_SOL / 10;
