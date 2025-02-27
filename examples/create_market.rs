use hit_my_bet::instruction::init_prediction_market;
use hit_my_bet::state::PredictionMarket;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::clock::UnixTimestamp;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::signer::SeedDerivable;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
  commitment_config::CommitmentConfig,
  signature::{Keypair, Signer},
};
use std::ops::Add;
use std::time::Duration;

const PROGRAM_ID: Pubkey = hit_my_bet::ID;

#[tokio::main]
async fn main() {
  let rpc_url = String::from("http://127.0.0.1:8899");
  let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

  // let creator = Keypair::new();
  let mut creator_seed: [u8; 32] = Default::default();
  creator_seed[0] = 0x0;
  creator_seed[1] = 0x1;
  let creator = Keypair::from_seed(&creator_seed).expect("creator");
  let creator_pubkey = creator.pubkey();
  println!("creator: {creator_pubkey:?}");
  airdrop(&client, &creator_pubkey, LAMPORTS_PER_SOL).await;

  let mut resolver_seed: [u8; 32] = Default::default();
  resolver_seed[0] = 0x1;
  resolver_seed[1] = 0x2;
  let resolver = Keypair::from_seed(&resolver_seed).expect("resolver");
  let resolver_pubkey = resolver.pubkey();
  println!("resolver: {resolver_pubkey:?}");
  airdrop(&client, &resolver_pubkey, LAMPORTS_PER_SOL).await;

  let market_id: &[u8; 16] = b"v0.1.1_market_id";

  let (market_pubkey, _) = Pubkey::find_program_address(
    &[PredictionMarket::SEED_PREFIX.as_bytes(), market_id],
    &PROGRAM_ID,
  );

  let timestamp = std::time::SystemTime::now()
    .add(Duration::from_secs(3600))
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();
  let instruction = init_prediction_market(
    PROGRAM_ID,
    creator_pubkey,
    market_pubkey,
    resolver_pubkey,
    market_id.clone(),
    UnixTimestamp::try_from(timestamp).expect("unix"),
  );
  let mut transaction = Transaction::new_with_payer(&[instruction], Some(&creator_pubkey));
  transaction.sign(
    &[&creator, &resolver],
    client.get_latest_blockhash().await.unwrap(),
  );
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
