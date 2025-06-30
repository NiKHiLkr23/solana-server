use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::env;

pub fn get_rpc_client() -> RpcClient {
    let rpc_url =
        env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed())
}
