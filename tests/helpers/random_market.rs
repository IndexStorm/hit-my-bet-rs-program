pub fn random_market_id() -> [u8; 16] {
  let mut market_id: [u8; 16] = Default::default();
  let nanos1 = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos()
    .to_le_bytes();
  market_id[0..8].copy_from_slice(&nanos1[0..8]);
  let nanos2 = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos()
    .to_le_bytes();
  market_id[8..16].copy_from_slice(&nanos2[0..8]);
  market_id
}
