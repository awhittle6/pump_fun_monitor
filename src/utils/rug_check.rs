use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;
use std::{collections::HashMap, env, error::Error, str::FromStr};

// API Response Structures
#[derive(Debug, Deserialize)]
struct MoralisResponse {
    possible_spam: bool,
    holder_distribution: HolderDistribution,
}

#[derive(Debug, Deserialize)]
struct HolderDistribution {
    top10: f64,
    top25: f64,
    total: u64,
}

/// Updated response for parsing GoPlus Security API responses.
///
/// The GoPlus API returns a JSON object with `code`, `message`, and
/// a `result` field (a map from token mint addresses to their details).
#[derive(Debug, Deserialize)]
pub struct GoPlusResponse {
    pub code: u32,
    pub message: String,
    pub result: HashMap<String, GoPlusTokenSecurity>,
}

/// Detailed security information for a single token, as returned by GoPlus.
/// Fields not guaranteed to be present in every response (such as `creators` or `dex`)
/// are marked as `Option` with a default.
#[derive(Debug, Deserialize)]
pub struct GoPlusTokenSecurity {
    pub balance_mutable_authority: AuthorityStatus,
    pub closable: AuthorityStatus,
    #[serde(rename = "default_account_state")]
    pub default_account_state: String,
    pub default_account_state_upgradable: AuthorityStatus,
    pub freezable: AuthorityStatus,
    pub mintable: AuthorityStatus,
    #[serde(rename = "non_transferable")]
    pub non_transferable: String,
    #[serde(rename = "total_supply")]
    pub total_supply: String,
    pub transfer_fee: Value,
    pub transfer_fee_upgradable: AuthorityStatus,
    pub transfer_hook: Vec<Value>,
    pub transfer_hook_upgradable: AuthorityStatus,
    pub trusted_token: i32,
    // Additional fields available in some responses
    #[serde(default)]
    pub creators: Option<Vec<Value>>,
    #[serde(default)]
    pub dex: Option<Vec<DexInfo>>,
    #[serde(default)]
    pub holders: Option<Vec<Holder>>,
    #[serde(rename = "lp_holders", default)]
    pub lp_holders: Option<Vec<Holder>>,
    #[serde(default)]
    pub metadata: Option<Metadata>,
    #[serde(rename = "metadata_mutable", default)]
    pub metadata_mutable: Option<MetadataMutable>,
}

/// Represents the authority and status for a specific token capability.
#[derive(Debug, Deserialize)]
pub struct AuthorityStatus {
    pub authority: Vec<String>,
    pub status: String,
}

/// Details for a decentralized exchange (DEX) listed in the GoPlus API response.
#[derive(Debug, Deserialize)]
pub struct DexInfo {
    pub day: PriceData,
    pub dex_name: String,
    pub fee_rate: String,
    pub id: String,
    pub lp_amount: Option<String>,
    pub month: PriceData,
    pub open_time: String,
    pub price: String,
    pub tvl: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub week: PriceData,
}

/// Price and volume data over a specific time period.
#[derive(Debug, Deserialize)]
pub struct PriceData {
    pub price_max: String,
    pub price_min: String,
    pub volume: String,
}

/// Information about a token holder.
#[derive(Debug, Deserialize)]
pub struct Holder {
    pub account: String,
    pub balance: String,
    pub is_locked: i32,
    pub locked_detail: Vec<Value>,
    pub percent: String,
    pub tag: String,
    pub token_account: String,
}

/// Metadata details (if available) for a token.
#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub description: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

/// Metadata mutability details (if available) for a token.
#[derive(Debug, Deserialize)]
pub struct MetadataMutable {
    pub metadata_upgrade_authority: Vec<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub enum RugStatus {
    Rug,
    NotRug,
    InsufficientData,
}

#[derive(Debug, Serialize)]
pub struct RugCheckResult {
    pub token_status: RugStatus,
    pub risk_factors: Vec<String>,
    pub confidence: f64,
    pub metadata: TokenMetadata,
    pub liquidity: LiquidityAnalysis,
}

#[derive(Debug, Serialize)]
pub struct TokenMetadata {
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
}

#[derive(Debug, Serialize)]
pub struct LiquidityAnalysis {
    pub pool_locked: bool,
    pub liquidity_amount: f64,
    pub creator_holdings: f64,
    pub burn_status: bool,
}


/// Check liquidity pool status using Raydium protocol analysis
async fn check_liquidity_pools(client: &RpcClient, mint: &Pubkey) -> Result<LiquidityAnalysis, Box<dyn Error>> {
    // Implementation would use get_account_data and parse Raydium pool state
    // Placeholder implementation:
    Ok(LiquidityAnalysis {
        pool_locked: false,
        liquidity_amount: 0.0,
        creator_holdings: 0.0,
        burn_status: false,
    })
}

/// Query Moralis API for spam detection
async fn check_moralis_api(mint_address: &str) -> Result<MoralisResponse, Box<dyn Error>> {
    let client = Client::new();
    let response = client
        .get("https://deep-index.moralis.io/api/v2/spl/token/metadata")
        .query(&[("chain", "sol"), ("addresses", mint_address)])
        .header("X-API-Key", "")
        .send()?
        .json()?;
    
    Ok(response)
}

async fn sol_sniffer_call(mint_address: &str) -> Result<SolSnifferResponse, Box<dyn Error + Send + Sync>> {
    let mint_address_owned = mint_address.to_owned();
    let api_key = env::var("TOKEN_SNIFFER_KEY").expect("Missing TOKEN_SNIFFER_KEY environment variable");
    let url = format!("https://solsniffer.com/api/v2/token/{m}", m = mint_address_owned);
    let result = task::spawn_blocking(move || {
        let client = Client::new();
        let response: SolSnifferResponse = client
        .get(url)
        .header("X-API-KEY", api_key.to_string())
        .send()?
        .json()?;
        Ok::<_, Box<dyn Error + Send + Sync>>(response)
    }).await??;
 Ok(result)   
}



pub async fn rugcheck_api(mint_address: &str) -> Result<RugCheckApiResponse, Box<dyn Error + Send + Sync>> {
    let mint_address_owned = mint_address.to_owned();
    let url = format!("https://api.rugcheck.xyz/v1/tokens/{n}/report/summary", n = mint_address_owned);
    let result: RugCheckApiResponse = task::spawn_blocking(move || {
        let client = Client::new();
        let response = client
        .get(url)
        .header("accept", "application/json")
        .send()?
        .json()?;
    Ok::<_, Box<dyn Error + Send + Sync>>(response)
    }).await??;
    Ok(result)
}
/// Query GoPlus Security API using the async reqwest client.
pub async fn check_goplus_api(mint_address: &str) -> Result<GoPlusResponse, Box<dyn Error + Send + Sync>> {
    let mint_address_owned = mint_address.to_owned();
    let result = task::spawn_blocking(move || {
        let client = Client::new();
        let response: GoPlusResponse = client
            .get("https://api.gopluslabs.io/api/v1/solana/token_security")
            .query(&[("contract_addresses", &mint_address_owned)])
            .send()?
            .json()?;
        Ok::<_, Box<dyn Error + Send + Sync>>(response)
    }).await??;
    Ok(result)
}

/// Comprehensive rug check for Solana SPL tokens
pub async fn check_solana_rug(mint_address: &str) -> Result<RugStatus, Box<dyn Error>> {
    // let client = RpcClient::new_with_commitment(RPC_URL, CommitmentConfig::confirmed());
    // let mint_pubkey = Pubkey::from_str(mint_address)?;
    
    // // 1. On-chain metadata analysis
    // let mint_data = client.get_account_data(&mint_pubkey)?;
    // let mint_info = Mint::unpack(&mint_data)?;
    // let metadata = TokenMetadata {
    //     mint_authority: mint_info.mint_authority.map(|pk| pk.to_string()).into(),
    //     freeze_authority: mint_info.freeze_authority.map(|pk| pk.to_string()).into(),
    //     supply: mint_info.supply,
    //     decimals: mint_info.decimals,
    //     is_initialized: mint_info.is_initialized,
    // };
    


    // let liquidity = check_liquidity_pools(&client, &mint_pubkey).await?;

    // let goplus_result = check_goplus_api(mint_address).await.expect("Error in goplus api request");



    // 4. Risk factor aggregation
    


    let rugcheck_result = rugcheck_api(mint_address).await.expect("Error in rug check API");
    println!("Rug check result: {:?}", rugcheck_result);
    println!("For mint: {:?}", mint_address);
    if let Some(num) = rugcheck_result.score {
        if num > 400 {
            println!("Rug status failed for {:?}", mint_address);
            return  Ok(RugStatus::Rug);
        }
    }
    // let solsniffer_response = sol_sniffer_call(mint_address).await.expect("Error calling sol sniffer api");

    // // Mint authority checks
    // if metadata.mint_authority.is_some() {
    //     risk_factors.push("Mutable mint authority".into());
    //     confidence += 25.0;
    // }

    // // Freeze authority checks
    // if metadata.freeze_authority.is_some() {
    //     risk_factors.push("Mutable freeze authority".into());
    //     confidence += 15.0;
    // }

    // Moralis API evaluation
    // if moralis_result.possible_spam {
    //     risk_factors.push("Moralis spam flag".into());
    //     confidence += 30.0;
    // }

 
    // // GoPlus API evaluation
    // if goplus_result.result.is_empty() {
    //     risk_factors.push("No security information found via GoPlus".into());
    //     confidence += 10.0;
    // }

    // // Liquidity checks
    // if !liquidity.pool_locked {
    //     risk_factors.push("Unlocked liquidity pool".into());
    //     confidence += 35.0;
    // }

    // Determine token status based on liquidity and overall confidence.
    // Here we use a minimal liquidity threshold (e.g. 1.0) to decide if there is enough trading data.
    // const MINIMUM_LIQUIDITY: f64 = 5.0;
    // let token_status = if liquidity.liquidity_amount < MINIMUM_LIQUIDITY {
    //     RugStatus::InsufficientData
    // } else if confidence >= 50.0 {
    //     RugStatus::Rug
    // } else {
    //     RugStatus::NotRug
    // };

    Ok(RugStatus::InsufficientData)
}

#[derive(Debug, Deserialize)]
pub struct SolSnifferResponse {
    #[serde(rename = "tokenData")]
    pub token_data: TokenData,
    #[serde(rename = "tokenInfo")]
    pub token_info: TokenInfo,
}

#[derive(Debug, Deserialize)]
pub struct TokenData {
    #[serde(rename = "indicatorData")]
    pub indicator_data: IndicatorData,
    #[serde(rename = "tokenOverview")]
    pub token_overview: TokenOverview,
    pub address: String,
    #[serde(rename = "deployTime")]
    pub deploy_time: String,
    pub externals: String,
    #[serde(rename = "liquidityList")]
    pub liquidity_list: Vec<LiquidityEntry>,
    #[serde(rename = "marketCap")]
    pub market_cap: f64,
    #[serde(rename = "ownersList")]
    pub owners_list: Vec<Owner>,
    pub score: i32,
    #[serde(rename = "tokenImg")]
    pub token_img: String,
    #[serde(rename = "tokenName")]
    pub token_name: String,
    #[serde(rename = "tokenSymbol")]
    pub token_symbol: String,
    #[serde(rename = "auditRisk")]
    pub audit_risk: AuditRisk,
}

#[derive(Debug, Deserialize)]
pub struct IndicatorData {
    pub high: IndicatorRating,
    pub moderate: IndicatorRating,
    pub low: IndicatorRating,
    pub specific: IndicatorRating,
}

#[derive(Debug, Deserialize)]
pub struct IndicatorRating {
    pub count: i32,
    pub details: String, // Contains a JSON string with more details.
}

#[derive(Debug, Deserialize)]
pub struct TokenOverview {
    pub deployer: String,
    pub mint: String,
    pub address: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Deserialize)]
pub struct LiquidityEntry {
    pub pumpfun: PumpfunLiquidity,
}

#[derive(Debug, Deserialize)]
pub struct PumpfunLiquidity {
    pub address: String,
    pub amount: f64,
    #[serde(rename = "lpPair")]
    pub lp_pair: String,
}

#[derive(Debug, Deserialize)]
pub struct Owner {
    pub address: String,
    pub amount: String,
    pub percentage: String,
}

#[derive(Debug, Deserialize)]
pub struct AuditRisk {
    #[serde(rename = "mintDisabled")]
    pub mint_disabled: bool,
    #[serde(rename = "freezeDisabled")]
    pub freeze_disabled: bool,
    #[serde(rename = "lpBurned")]
    pub lp_burned: bool,
    #[serde(rename = "top10Holders")]
    pub top10_holders: bool,
}

#[derive(Debug, Deserialize)]
pub struct TokenInfo {
    pub price: String,
    #[serde(rename = "supplyAmount")]
    pub supply_amount: u64,
    #[serde(rename = "mktCap")]
    pub mkt_cap: f64,
}

#[derive(Debug, Deserialize)]
pub struct RugCheckApiResponse {
    #[serde(rename = "tokenProgram")]
    pub token_program: Option<String>,
    #[serde(rename = "tokenType")]
    pub token_type: Option<String>,
    pub risks: Option<Vec<RugRisk>>,
    pub score: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RugRisk {
    pub name: String,
    pub value: String,
    pub description: String,
    pub score: u32,
    pub level: String,
}
