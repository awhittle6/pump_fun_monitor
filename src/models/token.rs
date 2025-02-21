use chrono::Utc;
use sqlx::types::BigDecimal;

#[derive(Debug)]
pub struct TokenInfo {
    pub mint_address: String,
    pub creator_address: Option<String>,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub symbol: Option<String>,
    pub bonding_address: Option<String>,
    
}

impl Default for TokenInfo {
    fn default() -> Self {
        TokenInfo {
            mint_address: "".to_string(),
            created_at: Some(Utc::now()),
            symbol: None,
            bonding_address: None,
            creator_address: None,
        }
    }
}


// Expanded models.rs

#[derive(Debug, Clone)]
pub struct TokenAnalysis {
    pub mint_address: String,
    pub bonding_curve_progress: f32,
    pub ilv: f32, // Initial Liquidity Velocity
    pub social_replies: u32,
    pub metadata_score: f32,
    pub creator_age: i64, // Block time delta
    pub sell_pressure: f32,
    pub temporal_features: Vec<f32>,
    pub risk_score: f32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TokenMetrics {
    /// The token's mint address; used to look up the token id.
    pub mint_address: String,
    pub bonding_percent: f64,
    pub ilv: f64,
    pub social_replies: i32,
    pub safety_score: f64,
    pub liquidity: f64,
    pub holders: i32,
    pub volume: Option<BigDecimal>,
    pub buy_volume: Option<BigDecimal>,
    pub sell_volume: Option<BigDecimal>,
}

#[derive(Debug, sqlx::FromRow)] 
pub struct ModelTrainingData {
    pub id: i64,
    pub features: Vec<f32>,
    pub outcome: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
