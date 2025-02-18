use {
    crate::{models::token::TokenInfo, utils::rug_check::{check_solana_rug, RugCheckResult}}, anyhow::Result, sqlx::PgPool, std::sync::Arc
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
    pub async fn delete_token_by_mint(&self, mint_address: &str) {
        match sqlx::query!(
            r#"
            DELETE FROM tokens
            WHERE mint_address = $1
            "#,
            mint_address
        )
        .execute(&*self.db_pool)
        .await {
            Ok(_) => println!("Successfully deleted token with mint address: {}", mint_address),
            Err(e) => eprintln!(
                "Failed to delete token with mint address {}: {:?}",
                mint_address, e
            ),
        }
    }

    pub async fn process_all_tokens(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tokens = sqlx::query_as!(
            MintAddress,
            r#"SELECT mint_address FROM tokens"#
        )
        .fetch_all(&*self.db_pool)
        .await?;

        for mint in tokens {
            let mint_address = mint.mint_address.clone();
            match check_solana_rug(&mint_address).await {
                Ok(result) => {
                    println!("Rug check result: {:?}, mint address: {:?}", result, &mint_address);
                    if result.confidence > 50.0 {
                        if let _t = self.delete_token_by_mint(&mint_address){
                            eprintln!("Error deleting token");
                        }
                    }
                },
                Err(e) => eprintln!("Failed to check solana rug for mint address {}: {:?}", mint_address, e),
            }
        }
        Ok(())
    }
    

    
    pub async fn store_token_info(&self, token_info: &TokenInfo) -> Result<()> {

        sqlx::query_as!(P,
            r#"
            INSERT INTO tokens (
                mint_address,
                token_symbol,
                launch_timestamp,
                supply,
                decimals,
                market_cap,
                rug_check_status,
                rug_check_confidence_score,
                created_at,
                updated_at
            )
            VALUES (
                $1, 
                $2, 
                $3, 
                $4, 
                $5, 
                $6, 
                $7, 
                $8, 
                COALESCE($9, NOW()), 
                NOW()
            )
            ON CONFLICT (mint_address) DO UPDATE SET
                token_symbol = EXCLUDED.token_symbol,
                launch_timestamp = EXCLUDED.launch_timestamp,
                supply = EXCLUDED.supply,
                decimals = EXCLUDED.decimals,
                market_cap = EXCLUDED.market_cap,
                rug_check_status = EXCLUDED.rug_check_status,
                rug_check_confidence_score = EXCLUDED.rug_check_confidence_score,
                updated_at = NOW()
            "#,
            token_info.mint_address,
            token_info.token_symbol,
            token_info.launch_timestamp,
            token_info.supply,
            token_info.decimals,
            token_info.market_cap,
            token_info.rug_check_status,
            token_info.rug_check_confidence_score,
            token_info.created_at,
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to store token info: {:?}", e))?;
        Ok(())
        
    }
    
    pub async fn new(db_uri: &str) -> Result<Arc<DbManager>> {
        let db_pool = Arc::new(PgPool::connect(db_uri).await?);
        Ok(Arc::new(DbManager { db_pool }))
    }
}


