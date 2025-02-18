use chrono::Utc;
use sqlx::types::BigDecimal;

#[derive(Debug)]
pub struct TokenInfo {
    pub mint_address: String,
    pub token_symbol: Option<String>,
    pub launch_timestamp: chrono::DateTime<Utc>,
    pub supply: Option<BigDecimal>,
    pub decimals: Option<BigDecimal>,
    pub market_cap: Option<BigDecimal>,
    pub rug_check_status: String,
    pub rug_check_confidence_score: Option<BigDecimal>,
    pub created_at: Option<chrono::DateTime<Utc>>,
}

impl Default for TokenInfo {
    fn default() -> Self {
        TokenInfo {
            mint_address: "".to_string(),
            token_symbol: None,
            launch_timestamp: Utc::now(),
            supply: None,
            decimals: None,
            market_cap: None,
            rug_check_status: "failed".to_string(),
            rug_check_confidence_score: None,
            created_at: None,
        }
    }
}