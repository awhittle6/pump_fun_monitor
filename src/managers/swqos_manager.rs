use {
    solana_client::nonblocking::rpc_client::RpcClient, 
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey}, 
    std::{fmt::Result, sync::Arc}
};

pub struct SwqosRpcClient{
    inner: Arc<RpcClient>,
}


impl SwqosRpcClient {
    pub fn new(rpc_endpoint: String) -> SwqosRpcClient {
        let client = RpcClient::new_socket_with_commitment(rpc_endpoint.parse().unwrap(), CommitmentConfig::confirmed());
        SwqosRpcClient {
            inner: Arc::new(client),
        }
    }

    pub async fn get_largest_token_holders(&self, mint: &Pubkey) -> Result {
        let total_supply = self.inner.
        get_token_supply_with_commitment(mint, CommitmentConfig::confirmed())
        .await
        .unwrap()
        .value;
        let holder_data = self.inner
        .get_token_largest_accounts_with_commitment(mint, CommitmentConfig::confirmed())
        .await.unwrap().value;

        let mut top_ten_pct : f64 = 0.0;
        for (i, val) in holder_data.iter().enumerate() {
            if i > 10 {
                break;
            }
            top_ten_pct += val.amount.ui_amount.unwrap() / total_supply.ui_amount.unwrap()
        }
        println!("The top ten holders hold {top_ten_pct}%");
        Ok(())
    }
}