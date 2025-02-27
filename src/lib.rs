#![allow(unexpected_cfgs)]

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub use solana_program;

solana_program::declare_id!("H1tBeT1u5GYAdMiXb1xvgSsfxxgAWTBzenKxt8iA2Tzu");

pub const ADMIN_RESOLVER: solana_program::pubkey::Pubkey =
  solana_program::pubkey::pubkey!("odd9qnGfY6iRzDV9sGLQrgSGxXTRAF6TQoiaYFJaDaR");
