use {
    crate::utils::rug_check::check_liquidity_pools, solana_client::nonblocking::rpc_client::RpcClient, solana_sdk::{commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey}, std::{mem, sync::Arc, time::Duration}
    
};




pub struct SwqosRpcClient{
    inner: Arc<RpcClient>,

}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::AnyBitPattern)]
struct BondingCurveState {
    virtual_sol_reserve: u64,   // SOL liquidity in lamports
    virtual_token_reserve: u64, // Token supply in base units
    progress_percent: u8,       // Bonding curve completion %
    last_updated: i64,          // UNIX timestamp of last update
}

impl SwqosRpcClient {
    pub fn new(rpc_endpoint: &str) -> Self {
        let client = RpcClient::new_with_timeout(rpc_endpoint.to_string(),
        Duration::from_millis(400)
        );
        SwqosRpcClient {
            inner: Arc::new(client),
        }
    }

    pub async fn validate_token(&self, mint: &Pubkey) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // let mint_account = self.inner.get_account(&mint).await?;
        // let mint_state = spl_token::state::Mint::unpack(&mint_account.data)
        //     .map_err(|e| {
        //         eprintln!("Error unpacking mint state: {:?}", e);
        //         Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        //     })?;
        let holders = self.inner.get_token_largest_accounts(mint).await?;
        // Assume that the bonding curve is the largest holder
        if let  Some(bc) = holders.get(0) {
            if let Some(tokens) = bc.amount.ui_amount {
                let prog = 100.0 as f64 - (tokens - 206900000 as f64) * (100.0 as f64)/(793100000 as f64) as f64;
                let ilv = (32190005730.0 as f64) * ((1073000191.0 as f64 - tokens) / (tokens * 1073000191.0 as f64)) as f64;
                // println!("ILV: {:?}", ilv);
                if prog > 5.0 {
                    return Ok(true);
                }
                // println!("Bonding curve progress: {:?} %", (prog * 100.0).round() / 100.0);
            }
        }
        // println!("Hodlers: {:?}", holders);
        // println!("Mint: {mint:?}");

        Ok(false)
    }
    
    pub async fn get_largest_token_holders(&self, mint: &Pubkey) -> Result<(), Box<dyn std::error::Error>> {
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