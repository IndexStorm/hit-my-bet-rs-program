use hit_my_bet::entrypoint::process_instruction;
use hit_my_bet::instruction::{init_prediction_market, make_prediction};
use hit_my_bet::state::{PredictionMarket, UserPrediction, UserVote, VOTE_PRICE};
use solana_program::clock::UnixTimestamp;
use solana_program::hash::Hash;
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, BanksClient, ProgramTest, ProgramTestBanksClientExt};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

const MARKET_ID: &[u8; 16] = b"market_id_16_chr";

#[tokio::test]
async fn test_success() {
  const NUM_VOTES: u16 = 5;

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

  let (prediction_pubkey, _) = Pubkey::find_program_address(
    &[
      UserPrediction::SEED_PREFIX.as_bytes(),
      market_pubkey.as_ref(),
      payer.pubkey().as_ref(),
    ],
    &hit_my_bet::ID,
  );

  let timestamp = get_unix_timestamp(&banks_client).await;

  let resolver_kp = Keypair::new();
  let init_market_tx = init_prediction_market_tx(
    payer.pubkey(),
    resolver_kp.pubkey(),
    market_pubkey,
    MARKET_ID.clone(),
    timestamp + 60,
    &[&payer, &resolver_kp],
    recent_blockhash,
  );

  let transaction_result = banks_client.process_transaction(init_market_tx).await;
  assert!(transaction_result.is_ok());

  let balance_before = get_balance(&banks_client, market_pubkey).await;

  let instruction = make_prediction(
    hit_my_bet::ID,
    payer.pubkey(),
    market_pubkey,
    prediction_pubkey,
    UserVote::Yes,
    NUM_VOTES,
  );
  let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
  recent_blockhash = banks_client
    .get_new_latest_blockhash(&recent_blockhash)
    .await
    .unwrap();
  transaction.sign(&[&payer], recent_blockhash);

  let transaction_result = banks_client.process_transaction(transaction).await;
  assert!(transaction_result.is_ok());

  _ = banks_client
    .get_new_latest_blockhash(&recent_blockhash)
    .await
    .unwrap();
  let balance_diff = get_balance(&banks_client, market_pubkey).await - balance_before;
  assert_eq!(balance_diff, VOTE_PRICE * (u64::from(NUM_VOTES)));

  let account: Result<_, _> = banks_client.get_account(market_pubkey).await;
  assert!(account.is_ok());
  assert!(account.as_ref().unwrap().is_some());
  let account_data = account.unwrap().unwrap().data;
  let market: PredictionMarket =
    borsh::BorshDeserialize::deserialize(&mut account_data.as_slice()).unwrap();
  assert_eq!(market.num_yes, u64::from(NUM_VOTES));
  assert_eq!(market.balance_yes, VOTE_PRICE * (u64::from(NUM_VOTES)));

  let account: Result<_, _> = banks_client.get_account(prediction_pubkey).await;
  assert!(account.is_ok());
  assert!(account.as_ref().unwrap().is_some());
  let account_data = account.unwrap().unwrap().data;
  let prediction: UserPrediction =
    borsh::BorshDeserialize::deserialize(&mut account_data.as_slice()).unwrap();
  assert_eq!(prediction.num_votes_yes, u64::from(NUM_VOTES));
}

async fn get_balance(client: &BanksClient, address: Pubkey) -> u64 {
  client.get_balance(address).await.unwrap()
}

async fn get_unix_timestamp(client: &BanksClient) -> UnixTimestamp {
  let clock = client
    .get_account(solana_program::clock::sysvar::ID)
    .await
    .unwrap()
    .unwrap();
  UnixTimestamp::from_le_bytes(clock.data[32..].as_ref().try_into().unwrap())
}

fn init_prediction_market_tx(
  payer: Pubkey,
  resolver: Pubkey,
  market_pubkey: Pubkey,
  market_id: [u8; 16],
  open_until: UnixTimestamp,
  signers: &[&Keypair],
  blockhash: Hash,
) -> Transaction {
  let instruction = init_prediction_market(
    hit_my_bet::ID,
    payer,
    market_pubkey,
    resolver,
    market_id,
    open_until,
  );
  let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer));
  transaction.sign(signers, blockhash);
  transaction
}
