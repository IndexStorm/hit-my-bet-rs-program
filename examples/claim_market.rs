use hit_my_bet::instruction::claim_market;
use hit_my_bet::state::{PredictionMarket, UserPrediction};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::SeedDerivable;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
  commitment_config::CommitmentConfig,
  signature::{Keypair, Signer},
};

const PROGRAM_ID: Pubkey = hit_my_bet::ID;

#[tokio::main]
async fn main() {
  let rpc_url = String::from("http://127.0.0.1:8899");
  let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

  let mut claimer_seed: [u8; 32] = Default::default();
  claimer_seed[0] = 0x2;
  claimer_seed[1] = 0x1;
  let claimer = Keypair::from_seed(&claimer_seed).expect("claimer");
  let claimer_pubkey = claimer.pubkey();
  println!("claimer: {claimer_pubkey:?}");

  let market_id: &[u8; 16] = b"v0.1.1_market_id";

  let (market_pubkey, _) = Pubkey::find_program_address(
    &[PredictionMarket::SEED_PREFIX.as_bytes(), market_id],
    &PROGRAM_ID,
  );
  let (prediction_pubkey, _) = Pubkey::find_program_address(
    &[
      UserPrediction::SEED_PREFIX.as_bytes(),
      market_pubkey.as_ref(),
      claimer_pubkey.as_ref(),
    ],
    &PROGRAM_ID,
  );

  let instruction = claim_market(PROGRAM_ID, claimer_pubkey, market_pubkey, prediction_pubkey);
  let mut transaction = Transaction::new_with_payer(&[instruction], Some(&claimer_pubkey));
  transaction.sign(&[&claimer], client.get_latest_blockhash().await.unwrap());
  match client.send_and_confirm_transaction(&transaction).await {
    Ok(signature) => println!("Transaction Signature: {}", signature),
    Err(err) => eprintln!("Error sending transaction: {}", err),
  }
}
