use crate::error::HitMyBetError;
use crate::state::{MarketResolution, UserVote, PROGRAM_VERSION};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::UnixTimestamp;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize)]
pub enum HitMyBetInstruction {
  InitPredictionMarket {
    version: u8,
    market_id: [u8; 16],
    open_until: UnixTimestamp,
  },
  MakePrediction {
    version: u8,
    vote: UserVote,
    num_votes: u16,
  },
  ResolveMarket {
    version: u8,
    resolution: MarketResolution,
  },
  ClaimMarket {
    version: u8,
  },
  SetMarketResolverAdmin {
    version: u8,
  },
  ResolveMarketAdmin {
    version: u8,
    resolution: MarketResolution,
  },
}

impl HitMyBetInstruction {
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    BorshDeserialize::try_from_slice(input)
      .map_err(|_| HitMyBetError::InstructionUnpackError.into())
  }

  pub fn pack(&self) -> borsh::io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::with_capacity(core::mem::size_of::<Self>());
    self.serialize(&mut buf)?;
    Ok(buf)
  }
}

pub fn init_prediction_market(
  program_id: Pubkey,
  creator: Pubkey,
  market_pubkey: Pubkey,
  resolver: Pubkey,
  market_id: [u8; 16],
  open_until: UnixTimestamp,
) -> Instruction {
  Instruction {
    program_id,
    accounts: vec![
      AccountMeta::new(creator, true),
      AccountMeta::new(market_pubkey, false),
      AccountMeta::new_readonly(resolver, true),
      AccountMeta::new_readonly(solana_program::system_program::ID, false),
    ],
    data: HitMyBetInstruction::InitPredictionMarket {
      version: PROGRAM_VERSION,
      market_id,
      open_until,
    }
    .pack()
    .expect("init_prediction_market pack"),
  }
}

pub fn make_prediction(
  program_id: Pubkey,
  voter: Pubkey,
  market_pubkey: Pubkey,
  user_prediction_pubkey: Pubkey,
  vote: UserVote,
  num_votes: u16,
) -> Instruction {
  Instruction {
    program_id,
    accounts: vec![
      AccountMeta::new(voter, true),
      AccountMeta::new(market_pubkey, false),
      AccountMeta::new(user_prediction_pubkey, false),
      AccountMeta::new_readonly(solana_program::system_program::ID, false),
    ],
    data: HitMyBetInstruction::MakePrediction {
      version: PROGRAM_VERSION,
      vote,
      num_votes,
    }
    .pack()
    .expect("make_prediction pack"),
  }
}

pub fn resolve_market(
  program_id: Pubkey,
  resolver: Pubkey,
  market_pubkey: Pubkey,
  resolution: MarketResolution,
) -> Instruction {
  Instruction {
    program_id,
    accounts: vec![
      AccountMeta::new(resolver, true),
      AccountMeta::new(market_pubkey, false),
    ],
    data: HitMyBetInstruction::ResolveMarket {
      version: PROGRAM_VERSION,
      resolution,
    }
    .pack()
    .expect("resolve_market pack"),
  }
}

pub fn claim_market(
  program_id: Pubkey,
  claimer: Pubkey,
  market_pubkey: Pubkey,
  prediction_pubkey: Pubkey,
) -> Instruction {
  Instruction {
    program_id,
    accounts: vec![
      AccountMeta::new(claimer, true),
      AccountMeta::new(market_pubkey, false),
      AccountMeta::new(prediction_pubkey, false),
    ],
    data: HitMyBetInstruction::ClaimMarket {
      version: PROGRAM_VERSION,
    }
    .pack()
    .expect("claim_market pack"),
  }
}
