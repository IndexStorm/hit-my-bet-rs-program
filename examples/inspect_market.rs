use hit_my_bet::state::PredictionMarket;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey};

#[tokio::main]
async fn main() {
  let rpc_url = String::from("https://api.devnet.solana.com");
  let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

  let market_pubkey = pubkey!("A534Mmaa1LLw7TmTvdM2gPPvGgh8YZeKSeWkJRTSm7Uh");
  let market_data = client.get_account_data(&market_pubkey).await.unwrap();

  let market: PredictionMarket =
    borsh::BorshDeserialize::deserialize(&mut market_data.as_slice()).expect("decode market");

  println!("{:?}", market);
}
