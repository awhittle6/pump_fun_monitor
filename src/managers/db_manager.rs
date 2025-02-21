use {
    super::swqos_manager::SwqosRpcClient, crate::models::token::{TokenInfo, TokenMetrics}, anyhow::{Ok, Result}, futures::TryFutureExt, solana_sdk::pubkey::Pubkey, sqlx::PgPool, std::{env, result::Result::Err, str::FromStr, sync::Arc}
};

pub struct DbManager {
    pub db_pool: Arc<PgPool>
}


#[derive(Debug)]
pub struct MintAddress {
    pub mint_address: String,
}


impl DbManager {

    /// Deletes a token from the tokens table based on its mint address.
    /// Instead of returning an error, any issues are logged.
    /// 
    /// 
    /// 
    // pub async fn delete_token_by_mint(&self, mint_address: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    //     sqlx::query!(
    //         r#"
    //         DELETE FROM tokens
    //         WHERE mint_address = $1
    //         "#,
    //         mint_address
    //     )
    //     .execute(&*self.db_pool)
    //     .await?;
    //     Ok(())
    // }

    pub async fn process_all_tokens(&self) -> anyhow::Result<()> {
        let tokens = sqlx::query_as!(
            MintAddress,
            r#"SELECT mint_address FROM tokens"#
        )
        .fetch_all(&*self.db_pool)
        .await?;
        if tokens.len() < 1 {
            return  Ok(());
        }
        let rpc_endpoint = env::var("RPC_ENDPOINT").expect("Missing RPC endpoint");
        let rpc_manager = SwqosRpcClient::new(&rpc_endpoint);
        for mint in tokens {
            let mint_address = mint.mint_address.clone();
            // let  rug_status = check_solana_rug(&mint_address).await.expect("msg");
            let pubkey = Pubkey::from_str(&mint_address).unwrap();
            match rpc_manager.validate_token(&pubkey).await {
                std::result::Result::Ok(is_valid) => {
                    if !is_valid {
                        println!("🚨Deleting token : {mint_address:?} from table");
                        sqlx::query!(
                            r#"
                            DELETE FROM tokens
                            WHERE mint_address = $1
                            "#,
                            mint_address
                        )
                        .execute(&*self.db_pool).await?;
                    }
                },
                Err(_) => {}
            };
        }
        Ok(())
    }
    

    
    pub async fn store_token_info(&self, token_info: &TokenInfo) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO tokens (
                mint_address,
                creator_address,
                created_at,
                symbol,
                bonding_address
            )
            VALUES (
                $1, 
                $2, 
                $3, 
                $4, 
                $5
            )
            ON CONFLICT (mint_address) DO UPDATE SET
                creator_address = EXCLUDED.creator_address,
                symbol = EXCLUDED.symbol,
                bonding_address = EXCLUDED.bonding_address
            "#,
            token_info.mint_address,
            token_info.creator_address,
            token_info.created_at,
            token_info.symbol,
            token_info.bonding_address,
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to store token info: {:?}", e))?;
        Ok(())
        
    }


    pub async fn store_token_metrics(&self, token_metric: &TokenMetrics) -> Result<()> {
        let token = sqlx::query!(
            r#"
            SELECT id FROM tokens
            WHERE mint_address = $1
            "#,
            token_metric.mint_address
        )
        .fetch_one(&*self.db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch token: {:?}", e))?;

        sqlx::query!(
            r#"
            INSERT INTO token_metrics (
                token_id,
                bonding_percent,
                ilv,
                social_replies,
                safety_score,
                liquidity,
                holders,
                volume,
                buy_volume,
                sell_volume
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            token.id,
            token_metric.bonding_percent,
            token_metric.ilv,
            token_metric.social_replies,
            token_metric.safety_score,
            token_metric.liquidity,
            token_metric.holders,
            token_metric.volume,
            token_metric.buy_volume,
            token_metric.sell_volume
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to store token metrics: {:?}", e))?;
        Ok(())
    }
    
    pub async fn new(db_uri: &str) -> Result<Arc<DbManager>> {
        let db_pool = Arc::new(PgPool::connect(db_uri).await?);
        Ok(Arc::new(DbManager { db_pool }))
    }
}


