use crate::state::PROGRAM_VERSION;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Default, Debug)]
pub struct UserPrediction {
  pub version: u8,
  pub bump_seed: u8,
  pub num_votes_yes: u64,
  pub num_votes_no: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum UserVote {
  Unspecified,
  Yes,
  No,
}

impl UserPrediction {
  pub const LEN: usize = core::mem::size_of::<UserPrediction>();

  pub const SEED_PREFIX: &'static str = "user_prediction";

  pub fn with_seed(bump_seed: u8) -> UserPrediction {
    UserPrediction {
      version: PROGRAM_VERSION,
      bump_seed,
      ..Default::default()
    }
  }
}
