use solana_program::decode_error::DecodeError;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum HitMyBetError {
  #[error("Account is not the expected program id")]
  IncorrectProgramId,
  #[error("Program version mismatch")]
  VersionMismatch,
  #[error("Failed to unpack instruction data")]
  InstructionUnpackError,
  #[error("Prediction market is already initialized")]
  AlreadyInitialized,
  #[error("Invalid program address generated from bump seed and key")]
  InvalidProgramDerivedAddress,
  #[error("Input account must be a signer")]
  InvalidSigner,
  #[error("Prediction market is not owned by the expected program id")]
  InvalidMarketOwner,
  #[error("Market is resolved")]
  MarketIsResolved,
  #[error("Market is closed")]
  MarketIsClosed,
  #[error("Resolver is not authorized for this market")]
  InvalidResolver,
  #[error("Market is not resolved")]
  MarketIsNotResolved,
}

impl Into<u32> for HitMyBetError {
  fn into(self) -> u32 {
    match self {
      HitMyBetError::IncorrectProgramId => 1,
      HitMyBetError::VersionMismatch => 2,
      HitMyBetError::InstructionUnpackError => 3,
      HitMyBetError::AlreadyInitialized => 4,
      HitMyBetError::InvalidProgramDerivedAddress => 5,
      HitMyBetError::InvalidSigner => 6,
      HitMyBetError::InvalidMarketOwner => 7,
      HitMyBetError::MarketIsResolved => 8,
      HitMyBetError::MarketIsClosed => 9,
      HitMyBetError::InvalidResolver => 10,
      HitMyBetError::MarketIsNotResolved => 11,
    }
  }
}

impl From<HitMyBetError> for ProgramError {
  fn from(value: HitMyBetError) -> Self {
    ProgramError::Custom(value.into())
  }
}

impl<T> DecodeError<T> for HitMyBetError {
  fn type_of() -> &'static str {
    "HitMyBetError"
  }
}
