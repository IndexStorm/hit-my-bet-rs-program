use hit_my_bet::error::HitMyBetError;
use hit_my_bet::instruction::init_prediction_market;
use hit_my_bet::entrypoint::process_instruction;
use hit_my_bet::state::{PredictionMarket, PROGRAM_VERSION};
use solana_program::clock::UnixTimestamp;
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, ProgramTest, ProgramTestBanksClientExt};
use solana_sdk::instruction::InstructionError;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::{Transaction, TransactionError};

const MARKET_ID: &[u8; 16] = b"market_id_16_chr";

#[tokio::test]
async fn test_success() {
  const OPEN_UNTIL: UnixTimestamp = 1;

  let (market_pubkey, bump_seed) = Pubkey::find_program_address(
    &[PredictionMarket::SEED_PREFIX.as_bytes(), MARKET_ID],
    &hit_my_bet::ID,
  );
  let mut test = ProgramTest::new(
    "hit_my_bet",
    hit_my_bet::ID,
    processor!(process_instruction),
  );

  test.set_compute_max_units(1_000);

  let (banks_client, payer, recent_blockhash) = test.start().await;
  let resolver_kp = Keypair::new();
  let instruction = init_prediction_market(
    hit_my_bet::ID,
    payer.pubkey(),
    market_pubkey,
    resolver_kp.pubkey(),
    MARKET_ID.clone(),
    OPEN_UNTIL,
  );
  let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
  transaction.sign(&[&payer, &resolver_kp], recent_blockhash);

  let transaction_result = banks_client.process_transaction(transaction).await;
  assert!(transaction_result.is_ok());

  let account: Result<_, _> = banks_client.get_account(market_pubkey).await;
  assert!(account.is_ok());
  assert!(account.as_ref().unwrap().is_some());
  let account_data = account.unwrap().unwrap().data;
  let market: borsh::io::Result<PredictionMarket> =
    borsh::BorshDeserialize::deserialize(&mut account_data.as_slice());
  assert!(market.is_ok());
  assert_eq!(
    market.unwrap(),
    PredictionMarket {
      version: PROGRAM_VERSION,
      bump_seed,
      resolver: resolver_kp.pubkey(),
      open_until: OPEN_UNTIL,
      ..Default::default()
    }
  );
}

// #[tokio::test]
// async fn test_not_enough_balance() {
//   const OPEN_UNTIL: UnixTimestamp = 1;
//
//   let (market_pubkey, _) = Pubkey::find_program_address(
//     &[PredictionMarket::SEED_PREFIX.as_bytes(), MARKET_ID],
//     &hit_my_bet::ID,
//   );
//   let mut test = ProgramTest::new(
//     "hit_my_bet",
//     hit_my_bet::ID,
//     processor!(process_instruction),
//   );
//
//   test.set_compute_max_units(1_000);
//
//   let (mut banks_client, mint_kp, mut recent_blockhash) = test.start().await;
//   let payer = Keypair::new();
//
//   let ix = solana_program::system_instruction::transfer(
//     &mint_kp.pubkey(),
//     &payer.pubkey(),
//     solana_program::rent::Rent::default().minimum_balance(0),
//   );
//   let tx = Transaction::new_signed_with_payer(
//     &[ix],
//     Some(&mint_kp.pubkey()),
//     &[&mint_kp],
//     recent_blockhash,
//   );
//   assert!(banks_client.process_transaction(tx).await.is_ok());
//   tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
//
//   let resolver_kp = Keypair::new();
//   let instruction = init_prediction_market(
//     hit_my_bet::ID,
//     payer.pubkey(),
//     resolver_kp.pubkey(),
//     market_pubkey,
//     MARKET_ID.clone(),
//     OPEN_UNTIL,
//   );
//   let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
//   recent_blockhash = banks_client
//     .get_new_latest_blockhash(&recent_blockhash)
//     .await
//     .unwrap();
//   transaction.sign(&[&payer, &resolver_kp], recent_blockhash);
//
//   let transaction_result = banks_client.process_transaction(transaction).await;
//   assert!(transaction_result.is_err());
// }

#[tokio::test]
async fn test_already_initialized() {
  let (market_pubkey, _) = Pubkey::find_program_address(
    &[PredictionMarket::SEED_PREFIX.as_bytes(), MARKET_ID],
    &hit_my_bet::ID,
  );
  let mut test = ProgramTest::new(
    "hit_my_bet",
    hit_my_bet::ID,
    processor!(process_instruction),
  );

  test.set_compute_max_units(1_000);

  let (mut banks_client, payer, mut recent_blockhash) = test.start().await;
  let resolver_kp = Keypair::new();
  let instruction = init_prediction_market(
    hit_my_bet::ID,
    payer.pubkey(),
    market_pubkey,
    resolver_kp.pubkey(),
    MARKET_ID.clone(),
    UnixTimestamp::from(1),
  );
  {
    let mut transaction =
      Transaction::new_with_payer(&[instruction.clone()], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &resolver_kp], recent_blockhash);

    let transaction_result = banks_client.process_transaction(transaction.clone()).await;
    assert!(transaction_result.is_ok());
  }
  {
    recent_blockhash = banks_client
      .get_new_latest_blockhash(&recent_blockhash)
      .await
      .unwrap();
    let mut transaction =
      Transaction::new_with_payer(&[instruction.clone()], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &resolver_kp], recent_blockhash);

    let transaction_result = banks_client.process_transaction(transaction).await;
    assert_eq!(
      transaction_result.unwrap_err().unwrap(),
      TransactionError::InstructionError(
        0,
        InstructionError::Custom(HitMyBetError::AlreadyInitialized.into())
      )
    );
  }
}
