
// use {
//     anyhow::Result, bs58, chrono::Utc, futures::{sink::SinkExt, stream::StreamExt}, log::error, solana_client::nonblocking::rpc_client::RpcClient, solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey}, spl_token::instruction::TokenInstruction, sqlx::PgPool, std::{sync::Arc, time::Duration}, tokio::sync::Mutex, tonic::{metadata::errors::InvalidMetadataValue, transport::Endpoint}, tonic_health::pb::health_client::HealthClient, yellowstone_grpc_client::{GeyserGrpcClient, InterceptorXToken}, yellowstone_grpc_proto::{
//         geyser::{
//             geyser_client::GeyserClient, subscribe_update::UpdateOneof, SubscribeRequest, SubscribeUpdateTransaction
//         },
//         prelude::{Message, SubscribeRequestPing},
//     },
//     crate::models::token::TokenInfo,
// };

// fn fetch_token_details(rpc_client: Arc<RpcClient>, token_mint: Pubkey) -> Result<TokenInfo> {
//     let handle = tokio::runtime::Handle::current();

//     println!("Looking to get token supply from: {}", token_mint);
//     let supply_resp = handle.block_on(rpc_client.get_token_supply_with_commitment(&token_mint, CommitmentConfig::confirmed()))?;
//     let supply_ui = supply_resp.value.ui_amount.unwrap_or(0.0);
//     let token_accounts = handle.block_on(rpc_client.get_program_accounts(
//         &token_mint,
//     ))?;
//     println!("Supply_ui: {:?}, token accounts: {:?}", supply_ui, token_accounts);

//     Ok(TokenInfo{
//         mint_address: token_mint.to_string(),
//         launch_timestamp: Utc::now(),
//         ..Default::default()
//     })
// }


// async fn store_token_info(pool: &PgPool, token_info: &TokenInfo) -> Result<()> {
//     sqlx::query!(
//         r#"
//         INSERT INTO tokens (
//             mint_address,
//             token_symbol,
//             launch_timestamp,
//             supply,
//             decimals,
//             market_cap,
//             rug_check_status,
//             rug_check_confidence_score,
//             created_at,
//             updated_at
//         )
//         VALUES (
//             $1, 
//             $2, 
//             $3, 
//             $4, 
//             $5, 
//             $6, 
//             $7, 
//             $8, 
//             COALESCE($9, NOW()), 
//             NOW()
//         )
//         ON CONFLICT (mint_address) DO UPDATE SET
//             token_symbol = EXCLUDED.token_symbol,
//             launch_timestamp = EXCLUDED.launch_timestamp,
//             supply = EXCLUDED.supply,
//             decimals = EXCLUDED.decimals,
//             market_cap = EXCLUDED.market_cap,
//             rug_check_status = EXCLUDED.rug_check_status,
//             rug_check_confidence_score = EXCLUDED.rug_check_confidence_score,
//             updated_at = NOW()
//         "#,
//         token_info.mint_address,
//         token_info.token_symbol,
//         token_info.launch_timestamp,
//         token_info.supply,
//         token_info.decimals,
//         token_info.market_cap,
//         token_info.rug_check_status,
//         token_info.rug_check_confidence_score,
//         token_info.created_at,
//     )
//     .execute(pool)
//     .await
//     .map_err(|e| anyhow::anyhow!("Failed to store token info: {:?}", e))?;
//     Ok(())
// }


// pub struct GrpcStreamManager {
//     client: GeyserGrpcClient<InterceptorXToken>,
//     is_connected: bool,
//     reconnect_attempts: u32,
//     max_reconnect_attempts: u32,
//     reconnect_interval: Duration,
//     rpc_client: Arc<RpcClient>,
//     db_pool: Arc<PgPool>,
// }




// impl GrpcStreamManager {
//     /// Handles transaction update messages from the gRPC stream
//     /// This function can be customized based on your requirements:
//     /// - Store transactions in a database
//     /// - Trigger specific actions based on transaction contents
//     /// - Filter for specific types of transactions
//     /// - Transform data into your required format
//     /// 
//     /// # Arguments
//     /// * `transaction_update` - The transaction update containing all details
//     pub fn handle_transaction_update(&self, transaction_update: &SubscribeUpdateTransaction) {
//         if let Some(transaction) = &transaction_update.transaction {
        
//             if let Some(transaction_message) = &transaction.transaction {
//                 if let Some(message) = &transaction_message.message {
//                     if matches_pump_fun_pattern(&message) {                       
//                         println!("Pump.fun launch detected. Transaction {}", bs58::encode(&transaction.signature.as_slice()).into_string());
//                         if let Some(token_key) = message.account_keys.get(0 as usize){
//                             if let Ok(pubkey) = Pubkey::try_from(token_key.as_slice()) {
//                                 println!("  Developer account: {}", pubkey.to_string());
//                             }
//                         };
//                         if let Some(token_key) = message.account_keys.get(1 as usize){
//                             if let Ok(pubkey) = Pubkey::try_from(token_key.as_slice()) {
//                                 let rpc = self.rpc_client.clone();
//                                 let db_pool = self.db_pool.clone();
//                                 tokio::spawn(async move {
//                                     if let Err(e) = process_new_token_launch(pubkey, rpc, db_pool).await {
//                                         eprintln!("Error processing token launch: {:?}", e);
//                                     }
//                                 });
//                                 println!("  Pump.fun token: {}\n", pubkey.to_string());
//                             }
//                         };
                        
//                     }
//                 }
//             }
//         }
//     }

//     /// Creates a new GrpcStreamManager instance
//     /// 
//     /// # Arguments
//     /// * `endpoint` - The gRPC endpoint URL
//     /// * `x_token` - Authentication token for the endpoint
//     pub async fn new(endpoint: &str, x_token: &str, rpc_endpoint: &str, db_url: &str) -> Result<Arc<Mutex<GrpcStreamManager>>> {
//         let interceptor = InterceptorXToken {
//             x_token: Some(x_token.parse().map_err(|e: InvalidMetadataValue| anyhow::Error::from(e))?),
//             x_request_snapshot: true,
//         };

//         let channel = Endpoint::from_shared(endpoint.to_string())?
//             .connect_timeout(Duration::from_secs(10))
//             .timeout(Duration::from_secs(10))
//             .connect()
//             .await
//             .map_err(|e| anyhow::Error::from(e))?;

//         let client: GeyserGrpcClient<InterceptorXToken> = GeyserGrpcClient::new(
//             HealthClient::with_interceptor(channel.clone(), interceptor.clone()),
//             GeyserClient::with_interceptor(channel, interceptor),
//         );
//         let rpc_client = Arc::new(RpcClient::new(rpc_endpoint.to_string()));
//         let db_pool = Arc::new(PgPool::connect(&db_url).await.expect("Unable to connect to db"));
//         Ok(Arc::new(Mutex::new(GrpcStreamManager {
//             client,
//             is_connected: false,
//             reconnect_attempts: 0,
//             max_reconnect_attempts: 10,
//             reconnect_interval: Duration::from_secs(5),
//             rpc_client,
//             db_pool
//         })))
//     }

//     /// Establishes connection and handles the subscription stream
//     /// 
//     /// # Arguments
//     /// * `request` - The subscription request containing transaction filters and other parameters
//     pub async fn connect(&mut self, request: SubscribeRequest) -> Result<()> {
//         let request = request.clone();
//         let (mut subscribe_tx, mut stream) = self.client.subscribe_with_request(Some(request.clone())).await?;

//         self.is_connected = true;
//         self.reconnect_attempts = 0;

//         while let Some(message) = stream.next().await {
//             match message {
//                 Ok(msg) => {
//                     match msg.update_oneof {
//                         Some(UpdateOneof::Transaction(transaction)) => {
//                             self.handle_transaction_update(&transaction);
//                         }
//                         Some(UpdateOneof::Ping(_)) => {
//                             subscribe_tx
//                                 .send(SubscribeRequest {
//                                     ping: Some(SubscribeRequestPing { id: 1 }),
//                                     ..Default::default()
//                                 })
//                                 .await?;
//                         }
//                         Some(UpdateOneof::Pong(_)) => {} // Ignore pong responses
//                         _ => {
//                             println!("Other update received: {:?}", msg);
//                         }
//                     }
//                 }
//                 Err(err) => {
//                     error!("Error: {:?}", err);
//                     self.is_connected = false;
//                     Box::pin(self.reconnect(request.clone())).await?;
//                     break;
//                 }
//             }
//         }

//         Ok(())
//     }

//     /// Attempts to reconnect when the connection is lost
//     /// 
//     /// # Arguments
//     /// * `request` - The original subscription request to reestablish the connection
//     pub async fn reconnect(&mut self, request: SubscribeRequest) -> Result<()> {
//         if self.reconnect_attempts >= self.max_reconnect_attempts {
//             println!("Max reconnection attempts reached");
//             return Ok(());
//         }

//         self.reconnect_attempts += 1;
//         println!("Reconnecting... Attempt {}", self.reconnect_attempts);

//         let backoff = self.reconnect_interval * std::cmp::min(self.reconnect_attempts, 5);
//         tokio::time::sleep(backoff).await;

//         Box::pin(self.connect(request)).await
//     }
// }
