use hit_my_bet::instruction::resolve_market;
use hit_my_bet::state::{MarketResolution, PredictionMarket};
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

  let mut resolver_seed: [u8; 32] = Default::default();
  resolver_seed[0] = 0x1;
  resolver_seed[1] = 0x2;
  let resolver = Keypair::from_seed(&resolver_seed).expect("resolver");
  let resolver_pubkey = resolver.pubkey();
  println!("resolver: {resolver_pubkey:?}");

  let market_id: &[u8; 16] = b"v0.1.1_market_id";

  let (market_pubkey, _) = Pubkey::find_program_address(
    &[PredictionMarket::SEED_PREFIX.as_bytes(), market_id],
    &PROGRAM_ID,
  );

  let instruction = resolve_market(
    PROGRAM_ID,
    resolver_pubkey,
    market_pubkey,
    MarketResolution::Yes,
  );
  let mut transaction = Transaction::new_with_payer(&[instruction], Some(&resolver_pubkey));
  transaction.sign(&[&resolver], client.get_latest_blockhash().await.unwrap());
  match client.send_and_confirm_transaction(&transaction).await {
    Ok(signature) => println!("Transaction Signature: {}", signature),
    Err(err) => eprintln!("Error sending transaction: {}", err),
  }
}
