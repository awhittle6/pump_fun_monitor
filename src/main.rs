mod models;
mod managers;
mod utils;
use {
    anyhow::Result, dotenv::dotenv, managers::{db_manager::DbManager, grpc_manager::GrpcStreamManager}, solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig}, solana_sdk::{commitment_config::CommitmentConfig, signature::Signature}, solana_transaction_status::UiTransactionEncoding, std::{collections::HashMap, env, str::FromStr}, tokio::sync::mpsc, yellowstone_grpc_proto::{
        geyser::{
            SubscribeRequest, SubscribeRequestFilterTransactions
        },
        prelude::CommitmentLevel,
    }
};





async fn _lookup_transaction(
    rpc_url: &str,
    signature: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new(rpc_url.to_string());
    if let Ok(sig) = Signature::from_str(signature) {
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Base58),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),

        };
        let transaction = client.get_transaction_with_config(&sig, config)?;
        
        let decoded = transaction.transaction.transaction.decode();
        println!("Decoded: {:?}", decoded);
    }
    Ok(())
}




#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    dotenv().ok();
    let grpc_endpoint = env::var("GRPC_ENDPOINT").expect("Missing GRPC Endpoint variable");
    let database_uri = env::var("DATABASE_URL").expect("Missing DB_URL environment variable");
    let db_manager = DbManager::new(&database_uri).await?;
    let (tx, mut rx) = mpsc::channel::<models::token::TokenInfo>(100);
    let manager = GrpcStreamManager::new(
        &grpc_endpoint,
        "",
        tx
    ).await?;

    // Create subscription request for token program transactions
    let request = SubscribeRequest {
        transactions: HashMap::from_iter(vec![(
            "transactions".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: vec!["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()],
                account_exclude: vec![],
                account_required: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()],
            },
        )]),
        commitment: Some(CommitmentLevel::Confirmed as i32),
        ..Default::default()
    };

    println!("Starting subscription for Pump.fun launches");


    let token_monitor = {
        let db_manager = db_manager.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = db_manager.process_all_tokens().await {
                    println!("Error processing all tokens: {:?}", e);
                }
                // sleep(Duration::from_secs(4)).await;
                
            }
        })
    };

    let db_consumer = {
        let db_manager = db_manager.clone();
        tokio::spawn(async move {
            while let Some(token_info) = rx.recv().await {
                if let Err(e) = db_manager.store_token_info(&token_info).await{
                    eprintln!("Error storing token info: {:?}", e);
                } else {
                    println!("DB stored token info for mint: {:?}", token_info);
                }
            }
        })
    };
    
    tokio::join!(token_monitor);
    Ok(())
}