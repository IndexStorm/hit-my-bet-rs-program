use crate::error::HitMyBetError;
use crate::processor;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

solana_program::entrypoint_no_alloc!(process_instruction);

pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  instruction_data: &[u8],
) -> ProgramResult {
  assert_hit_my_bet_program_id(program_id)?;
  processor::process_instruction(program_id, accounts, instruction_data).map(|_| Ok(()))?
}

fn assert_hit_my_bet_program_id(program_id: &Pubkey) -> ProgramResult {
  if !crate::check_id(program_id) {
    Err(HitMyBetError::IncorrectProgramId.into())
  } else {
    Ok(())
  }
}
