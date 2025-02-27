use crate::state::PROGRAM_VERSION;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Default, PartialEq, Debug)]
pub struct PredictionMarket {
  pub version: u8,
  pub bump_seed: u8,
  pub resolver: Pubkey,
  pub num_yes: u64,
  pub num_no: u64,
  pub balance_yes: u64,
  pub balance_no: u64,
  pub resolution: MarketResolution,
  pub open_until: UnixTimestamp,
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub enum MarketResolution {
  Unresolved,
  Tie,
  Yes,
  No,
}

impl PredictionMarket {
  pub const LEN: usize = core::mem::size_of::<PredictionMarket>();

  pub const SEED_PREFIX: &'static str = "prediction_market";

  pub fn with_params(params: InitPredictionMarketParams) -> PredictionMarket {
    PredictionMarket {
      version: PROGRAM_VERSION,
      bump_seed: params.bump_seed,
      resolver: params.resolver,
      open_until: params.open_until,
      ..Default::default()
    }
  }
}

pub struct InitPredictionMarketParams {
  pub bump_seed: u8,
  pub resolver: Pubkey,
  pub open_until: UnixTimestamp,
}

impl Default for MarketResolution {
  fn default() -> MarketResolution {
    MarketResolution::Unresolved
  }
}

// impl Sized for PredictionMarket {
//
// }
//
// impl Sealed for PredictionMarket {}
//
// impl IsInitialized for PredictionMarket {
//   fn is_initialized(&self) -> bool {
//     self.version != UNINITIALIZED_VERSION
//   }
// }

// impl Pack for PredictionMarket {
//   // const LEN: usize = 128;
//   // const LEN: usize = core::mem::size_of::<PredictionMarket>();
//   const LEN: usize = Self::LEN;
//
//   fn pack_into_slice(&self, dst: &mut [u8]) {
//     dst[0] = self.version.to_le();
//     dst[1] = self.bump_seed.to_le();
//     dst[2..2 + PUBKEY_BYTES].copy_from_slice(self.resolver.as_ref());
//     dst[34..34 + 8].copy_from_slice(&self.num_yes.to_le_bytes());
//     dst[42..42 + 8].copy_from_slice(&self.num_no.to_le_bytes());
//     dst[50..50 + 8].copy_from_slice(&self.balance_yes.to_le_bytes());
//     dst[58..58 + 8].copy_from_slice(&self.balance_no.to_le_bytes());
//   }
//
//   fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
//     let version = u8::from_le(src[0]);
//     if version != PROGRAM_VERSION {
//       return Err(ProgramError::InvalidAccountData);
//     }
//     Ok(PredictionMarket {
//       version,
//       bump_seed: u8::from_le(src[1]),
//       resolver: Pubkey::new_from_array(src[2..2 + PUBKEY_BYTES].try_into().unwrap()),
//       num_yes: u64::from_le_bytes(src[34..42].try_into().unwrap()),
//       num_no: u64::from_le_bytes(src[42..50].try_into().unwrap()),
//       balance_yes: u64::from_le_bytes(src[50..58].try_into().unwrap()),
//       balance_no: u64::from_le_bytes(src[58..66].try_into().unwrap()),
//     })
//   }
// }
