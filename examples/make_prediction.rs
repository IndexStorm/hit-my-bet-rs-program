use hit_my_bet::instruction::make_prediction;
use hit_my_bet::state::{PredictionMarket, UserPrediction, UserVote};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::signer::SeedDerivable;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
  commitment_config::CommitmentConfig,
  signature::{Keypair, Signer},
};
use std::time::Duration;

const PROGRAM_ID: Pubkey = hit_my_bet::ID;

#[tokio::main]
async fn main() {
  let rpc_url = String::from("http://127.0.0.1:8899");
  let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

  let mut voter_seed: [u8; 32] = Default::default();
  voter_seed[0] = 0x2;
  voter_seed[1] = 0x1;
  let voter = Keypair::from_seed(&voter_seed).expect("voter");
  let voter_pubkey = voter.pubkey();
  println!("voter: {voter_pubkey:?}");
  airdrop(&client, &voter_pubkey, LAMPORTS_PER_SOL).await;

  let market_id: &[u8; 16] = b"v0.1.1_market_id";

  let (market_pubkey, _) = Pubkey::find_program_address(
    &[PredictionMarket::SEED_PREFIX.as_bytes(), market_id],
    &PROGRAM_ID,
  );
  let (prediction_pubkey, _) = Pubkey::find_program_address(
    &[
      UserPrediction::SEED_PREFIX.as_bytes(),
      market_pubkey.as_ref(),
      voter_pubkey.as_ref(),
    ],
    &PROGRAM_ID,
  );

  let instruction = make_prediction(
    PROGRAM_ID,
    voter_pubkey,
    market_pubkey,
    prediction_pubkey,
    UserVote::Yes,
    5,
  );
  let mut transaction = Transaction::new_with_payer(&[instruction], Some(&voter_pubkey));
  transaction.sign(&[&voter], client.get_latest_blockhash().await.unwrap());
  match client.send_and_confirm_transaction(&transaction).await {
    Ok(signature) => println!("Transaction Signature: {}", signature),
    Err(err) => eprintln!("Error sending transaction: {}", err),
  }

  let instruction = make_prediction(
    PROGRAM_ID,
    voter_pubkey,
    market_pubkey,
    prediction_pubkey,
    UserVote::No,
    3,
  );
  let mut transaction = Transaction::new_with_payer(&[instruction], Some(&voter_pubkey));
  transaction.sign(&[&voter], client.get_latest_blockhash().await.unwrap());
  match client.send_and_confirm_transaction(&transaction).await {
    Ok(signature) => println!("Transaction Signature: {}", signature),
    Err(err) => eprintln!("Error sending transaction: {}", err),
  }
}

async fn airdrop(client: &RpcClient, address: &Pubkey, amount: u64) -> Signature {
  let signature = client
    .request_airdrop(&address, amount)
    .await
    .expect("airdrop");
  for _ in 0..15 {
    let confirmed = client.confirm_transaction(&signature).await.unwrap();
    if confirmed {
      break;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
  }
  signature
}
