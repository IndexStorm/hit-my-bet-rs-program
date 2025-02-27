use crate::error::HitMyBetError;
use crate::instruction::HitMyBetInstruction;
use crate::state::{
  InitPredictionMarketParams, MarketResolution, PredictionMarket, UserPrediction, UserVote,
  CREATE_MARKET_FEE, PROGRAM_VERSION, VOTE_PRICE,
};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::clock::UnixTimestamp;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, system_instruction};

pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  input: &[u8],
) -> ProgramResult {
  let instruction = HitMyBetInstruction::unpack(input)?;
  match instruction {
    HitMyBetInstruction::InitPredictionMarket {
      version,
      market_id,
      open_until,
    } => {
      if version != PROGRAM_VERSION {
        return Err(HitMyBetError::VersionMismatch.into());
      }
      process_init_prediction_market(program_id, accounts, &market_id, open_until)
    }
    HitMyBetInstruction::MakePrediction {
      version,
      vote,
      num_votes,
    } => {
      if version != PROGRAM_VERSION {
        return Err(HitMyBetError::VersionMismatch.into());
      }
      if let UserVote::Unspecified = vote {
        return Err(HitMyBetError::InstructionUnpackError.into());
      }
      process_make_prediction(program_id, accounts, vote, num_votes)
    }
    HitMyBetInstruction::ResolveMarket {
      version,
      resolution,
    } => {
      if version != PROGRAM_VERSION {
        return Err(HitMyBetError::VersionMismatch.into());
      }
      if let MarketResolution::Unresolved = resolution {
        return Err(HitMyBetError::InstructionUnpackError.into());
      }
      process_resolve_market(accounts, resolution)
    }
    HitMyBetInstruction::ClaimMarket { version } => {
      if version != PROGRAM_VERSION {
        return Err(HitMyBetError::VersionMismatch.into());
      }
      process_claim_market(program_id, accounts)
    }
    HitMyBetInstruction::SetMarketResolverAdmin { version } => {
      if version != PROGRAM_VERSION {
        return Err(HitMyBetError::VersionMismatch.into());
      }
      process_set_market_resolver_admin(accounts)
    }
    HitMyBetInstruction::ResolveMarketAdmin {
      version,
      resolution,
    } => {
      if version != PROGRAM_VERSION {
        return Err(HitMyBetError::VersionMismatch.into());
      }
      if let MarketResolution::Unresolved = resolution {
        return Err(HitMyBetError::InstructionUnpackError.into());
      }
      process_resolve_market_admin(accounts, resolution)
    }
  }
}

fn process_init_prediction_market(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  market_id: &[u8; 16],
  open_until: UnixTimestamp,
) -> ProgramResult {
  let account_info_iter = &mut accounts.iter();
  let creator_info = next_account_info(account_info_iter)?;
  let prediction_market_info = next_account_info(account_info_iter)?;
  let resolver_info = next_account_info(account_info_iter)?;
  let system_program = next_account_info(account_info_iter)?;
  if !creator_info.is_signer {
    return Err(HitMyBetError::InvalidSigner.into());
  }
  if !resolver_info.is_signer {
    return Err(HitMyBetError::InvalidSigner.into());
  }
  assert_system_program(system_program.key)?;

  let (prediction_market_pda, prediction_market_bump) = Pubkey::find_program_address(
    &[PredictionMarket::SEED_PREFIX.as_bytes(), market_id.as_ref()],
    program_id,
  );
  if !prediction_market_info.key.eq(&prediction_market_pda) {
    return Err(HitMyBetError::InvalidProgramDerivedAddress.into());
  }
  if !prediction_market_info.data_is_empty() {
    return Err(HitMyBetError::AlreadyInitialized.into());
  }

  let rent = Rent::get()?;
  let rent_lamports = rent.minimum_balance(PredictionMarket::LEN) + CREATE_MARKET_FEE;

  invoke_signed(
    &system_instruction::create_account(
      creator_info.key,
      &prediction_market_info.key,
      rent_lamports,
      u64::try_from(PredictionMarket::LEN).expect("data size"),
      program_id,
    ),
    &[
      creator_info.clone(),
      prediction_market_info.clone(),
      system_program.clone(),
    ],
    &[&[
      PredictionMarket::SEED_PREFIX.as_bytes(),
      market_id.as_ref(),
      &[prediction_market_bump],
    ]],
  )?;

  borsh::BorshSerialize::serialize(
    &PredictionMarket::with_params(InitPredictionMarketParams {
      bump_seed: prediction_market_bump,
      resolver: resolver_info.key.clone(),
      open_until,
    }),
    &mut &mut prediction_market_info.data.borrow_mut()[..],
  )?;

  Ok(())
}

fn process_make_prediction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  vote: UserVote,
  num_votes: u16,
) -> ProgramResult {
  let account_info_iter = &mut accounts.iter();
  let voter_info = next_account_info(account_info_iter)?;
  let prediction_market_info = next_account_info(account_info_iter)?;
  let user_prediction_info = next_account_info(account_info_iter)?;
  let system_program = next_account_info(account_info_iter)?;
  if !voter_info.is_signer {
    return Err(HitMyBetError::InvalidSigner.into());
  }
  assert_market_owner(prediction_market_info.owner)?;
  assert_system_program(system_program.key)?;

  let (user_prediction_pda, user_prediction_bump) = Pubkey::find_program_address(
    &[
      UserPrediction::SEED_PREFIX.as_bytes(),
      prediction_market_info.key.as_ref(),
      voter_info.key.as_ref(),
    ],
    program_id,
  );
  if !user_prediction_info.key.eq(&user_prediction_pda) {
    return Err(HitMyBetError::InvalidProgramDerivedAddress.into());
  }

  let mut market: PredictionMarket =
    borsh::BorshDeserialize::deserialize(&mut prediction_market_info.data.borrow().as_ref())?;

  if market.resolution != MarketResolution::Unresolved {
    return Err(HitMyBetError::MarketIsResolved.into());
  }

  let clock = solana_program::clock::Clock::get()?;
  let timestamp = clock.unix_timestamp;
  if timestamp >= market.open_until {
    return Err(HitMyBetError::MarketIsClosed.into());
  }

  let mut prediction: UserPrediction;
  if user_prediction_info.data_is_empty() {
    prediction = UserPrediction::with_seed(user_prediction_bump);

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(UserPrediction::LEN);

    invoke_signed(
      &system_instruction::create_account(
        voter_info.key,
        &user_prediction_info.key,
        rent_lamports,
        u64::try_from(UserPrediction::LEN).expect("data size"),
        program_id,
      ),
      &[
        voter_info.clone(),
        user_prediction_info.clone(),
        system_program.clone(),
      ],
      &[&[
        UserPrediction::SEED_PREFIX.as_bytes(),
        prediction_market_info.key.as_ref(),
        voter_info.key.as_ref(),
        &[user_prediction_bump],
      ]],
    )?;
    msg!("init new prediction");
  } else {
    prediction =
      borsh::BorshDeserialize::deserialize(&mut user_prediction_info.data.borrow().as_ref())?;

    msg!("existing prediction");
  }

  let bet_amount = u64::from(num_votes) * VOTE_PRICE;

  invoke_signed(
    &system_instruction::transfer(voter_info.key, &prediction_market_info.key, bet_amount),
    &[voter_info.clone(), prediction_market_info.clone()],
    &[],
  )?;

  match vote {
    UserVote::Unspecified => {
      return Err(HitMyBetError::InstructionUnpackError.into());
    }
    UserVote::Yes => {
      market.balance_yes += bet_amount;
      market.num_yes += u64::from(num_votes);
      prediction.num_votes_yes += u64::from(num_votes);
      msg!(
        "bet={},balance={},num={},votes={}",
        bet_amount,
        market.balance_yes,
        market.num_yes,
        prediction.num_votes_yes
      );
    }
    UserVote::No => {
      market.balance_no += bet_amount;
      market.num_no += u64::from(num_votes);
      prediction.num_votes_no += u64::from(num_votes);
      msg!(
        "bet={},balance={},num={},votes={}",
        bet_amount,
        market.balance_no,
        market.num_no,
        prediction.num_votes_no
      );
    }
  }

  borsh::BorshSerialize::serialize(
    &market,
    &mut &mut prediction_market_info.data.borrow_mut()[..],
  )?;

  borsh::BorshSerialize::serialize(
    &prediction,
    &mut &mut user_prediction_info.data.borrow_mut()[..],
  )?;

  Ok(())
}

fn process_resolve_market(accounts: &[AccountInfo], resolution: MarketResolution) -> ProgramResult {
  let account_info_iter = &mut accounts.iter();
  let resolver_info = next_account_info(account_info_iter)?;
  let prediction_market_info = next_account_info(account_info_iter)?;
  assert_market_owner(prediction_market_info.owner)?;

  if !resolver_info.is_signer {
    return Err(HitMyBetError::InvalidSigner.into());
  }

  let mut market: PredictionMarket =
    borsh::BorshDeserialize::deserialize(&mut prediction_market_info.data.borrow().as_ref())?;

  if !market.resolver.eq(&resolver_info.key) {
    return Err(HitMyBetError::InvalidResolver.into());
  }

  if market.resolution != MarketResolution::Unresolved {
    return Err(HitMyBetError::MarketIsResolved.into());
  }

  market.resolution = resolution;
  let clock = solana_program::clock::Clock::get()?;
  market.open_until = clock.unix_timestamp;

  borsh::BorshSerialize::serialize(
    &market,
    &mut &mut prediction_market_info.data.borrow_mut()[..],
  )?;

  msg!("resolution: {:?}", market.resolution);

  Ok(())
}

fn process_set_market_resolver_admin(accounts: &[AccountInfo]) -> ProgramResult {
  let account_info_iter = &mut accounts.iter();
  let resolver_info = next_account_info(account_info_iter)?;
  let prediction_market_info = next_account_info(account_info_iter)?;
  assert_market_owner(prediction_market_info.owner)?;

  if !resolver_info.is_signer {
    return Err(HitMyBetError::InvalidSigner.into());
  }

  let mut market: PredictionMarket =
    borsh::BorshDeserialize::deserialize(&mut prediction_market_info.data.borrow().as_ref())?;

  market.resolver = resolver_info.key.clone();

  borsh::BorshSerialize::serialize(
    &market,
    &mut &mut prediction_market_info.data.borrow_mut()[..],
  )?;

  msg!("resolver updated");

  Ok(())
}

fn process_resolve_market_admin(
  accounts: &[AccountInfo],
  resolution: MarketResolution,
) -> ProgramResult {
  let account_info_iter = &mut accounts.iter();
  let resolver_info = next_account_info(account_info_iter)?;
  let prediction_market_info = next_account_info(account_info_iter)?;
  assert_market_owner(prediction_market_info.owner)?;

  if !resolver_info.is_signer {
    return Err(HitMyBetError::InvalidSigner.into());
  }

  let mut market: PredictionMarket =
    borsh::BorshDeserialize::deserialize(&mut prediction_market_info.data.borrow().as_ref())?;

  if !crate::ADMIN_RESOLVER.eq(&resolver_info.key) {
    return Err(HitMyBetError::InvalidResolver.into());
  }

  market.resolution = resolution;
  let clock = solana_program::clock::Clock::get()?;
  market.open_until = clock.unix_timestamp;

  borsh::BorshSerialize::serialize(
    &market,
    &mut &mut prediction_market_info.data.borrow_mut()[..],
  )?;

  msg!("resolution: {:?}", market.resolution);

  Ok(())
}

fn process_claim_market(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
  let account_info_iter = &mut accounts.iter();
  let claimer_info = next_account_info(account_info_iter)?;
  let prediction_market_info = next_account_info(account_info_iter)?;
  let user_prediction_info = next_account_info(account_info_iter)?;
  assert_market_owner(prediction_market_info.owner)?;
  if !claimer_info.is_signer {
    return Err(HitMyBetError::InvalidSigner.into());
  }

  let (user_prediction_pda, _) = Pubkey::find_program_address(
    &[
      UserPrediction::SEED_PREFIX.as_bytes(),
      prediction_market_info.key.as_ref(),
      claimer_info.key.as_ref(),
    ],
    program_id,
  );
  if !user_prediction_info.key.eq(&user_prediction_pda) {
    return Err(HitMyBetError::InvalidProgramDerivedAddress.into());
  }

  let market: PredictionMarket =
    borsh::BorshDeserialize::deserialize(&mut prediction_market_info.data.borrow().as_ref())?;

  if market.resolution == MarketResolution::Unresolved {
    return Err(HitMyBetError::MarketIsNotResolved.into());
  }

  let prediction: UserPrediction =
    borsh::BorshDeserialize::deserialize(&mut user_prediction_info.data.borrow().as_ref())?;

  let win_per_vote: u64;
  let votes_to_claim: u64;
  match market.resolution {
    MarketResolution::Unresolved => {
      return Err(HitMyBetError::MarketIsNotResolved.into());
    }
    MarketResolution::Yes => {
      win_per_vote = market.balance_no / market.num_yes;
      votes_to_claim = prediction.num_votes_yes;
    }
    MarketResolution::No => {
      win_per_vote = market.balance_yes / market.num_no;
      votes_to_claim = prediction.num_votes_no;
    }
    MarketResolution::Tie => {
      win_per_vote = 0;
      votes_to_claim = prediction.num_votes_no + prediction.num_votes_yes;
    }
  };

  if votes_to_claim == 0 {
    msg!("lost prediction: {:?}", prediction);

    let dest_starting_lamports = claimer_info.lamports();
    **claimer_info.lamports.borrow_mut() = dest_starting_lamports
      .checked_add(user_prediction_info.lamports())
      .unwrap();
    **user_prediction_info.lamports.borrow_mut() = 0;

    user_prediction_info.assign(&solana_program::system_program::ID);
    user_prediction_info.realloc(0, false)?;

    return Ok(());
  } else {
    let lamports_to_claim = (win_per_vote * votes_to_claim) + (votes_to_claim * VOTE_PRICE);
    msg!(
      "win prediction: {:?},win={},refund={},total={},per_vote={},votes={}",
      prediction,
      win_per_vote * votes_to_claim,
      votes_to_claim * VOTE_PRICE,
      lamports_to_claim,
      win_per_vote,
      votes_to_claim
    );
    let dest_starting_lamports = claimer_info.lamports();
    **claimer_info.lamports.borrow_mut() = dest_starting_lamports
      .checked_add(user_prediction_info.lamports())
      .unwrap()
      .checked_add(lamports_to_claim)
      .unwrap();
    **user_prediction_info.lamports.borrow_mut() = 0;
    **prediction_market_info.lamports.borrow_mut() -= lamports_to_claim;

    user_prediction_info.assign(&solana_program::system_program::ID);
    user_prediction_info.realloc(0, false)?;

    // invoke_signed(
    //   &system_instruction::transfer(&prediction_market_info.key, claimer_info.key, lamports_to_claim),
    //   &[prediction_market_info.clone(), claimer_info.clone()],
    //   &[],
    // )?;
  }

  Ok(())
}

fn assert_market_owner(program_id: &Pubkey) -> ProgramResult {
  if !crate::check_id(program_id) {
    Err(HitMyBetError::InvalidMarketOwner.into())
  } else {
    Ok(())
  }
}

fn assert_system_program(program_id: &Pubkey) -> ProgramResult {
  if !solana_program::system_program::check_id(program_id) {
    Err(HitMyBetError::IncorrectProgramId.into())
  } else {
    Ok(())
  }
}

// fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
//   if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
//     msg!(&rent.minimum_balance(account_info.data_len()).to_string());
//     Err(HitMyBetError::NotRentExempt.into())
//   } else {
//     Ok(())
//   }
// }

// fn assert_uninitialized<T: Pack + IsInitialized>(
//   account_info: &AccountInfo,
// ) -> Result<T, ProgramError> {
//   let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
//   if account.is_initialized() {
//     Err(HitMyBetError::AlreadyInitialized.into())
//   } else {
//     Ok(account)
//   }
// }
