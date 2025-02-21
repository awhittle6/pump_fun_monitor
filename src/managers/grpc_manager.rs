
use {
    crate::{models::token::TokenInfo, utils::token_filter::matches_pump_fun_pattern}, anyhow::Result, bs58, futures::{sink::SinkExt, stream::StreamExt}, log::error, solana_client::nonblocking::rpc_client::RpcClient, solana_sdk::pubkey::Pubkey, std::{str::FromStr, sync::Arc, time::Duration}, tokio::sync::{mpsc, Mutex}, tonic::{metadata::errors::InvalidMetadataValue, transport::Endpoint}, tonic_health::pb::health_client::HealthClient, yellowstone_grpc_client::{GeyserGrpcClient, InterceptorXToken}, yellowstone_grpc_proto::{
        geyser::{
            geyser_client::GeyserClient, subscribe_update::UpdateOneof, SubscribeRequest, SubscribeUpdateTransaction
        },
        prelude::SubscribeRequestPing,
    }
};


pub struct GrpcStreamManager {
    client: GeyserGrpcClient<InterceptorXToken>,
    is_connected: bool,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    reconnect_interval: Duration,
    event_sender: mpsc::Sender<TokenInfo>,
}



impl GrpcStreamManager {
    /// Handles transaction update messages from the gRPC stream
    /// This function can be customized based on your requirements:
    /// - Store transactions in a database
    /// - Trigger specific actions based on transaction contents
    /// - Filter for specific types of transactions
    /// - Transform data into your required format
    /// 
    /// # Arguments
    /// * `transaction_update` - The transaction update containing all details
    /// 
    // pub fn handle_account_update(&self, account_update: &Pubkey)
    pub fn handle_transaction_update(&self, transaction_update: &SubscribeUpdateTransaction) {
        if let Some(transaction) = &transaction_update.transaction {
        
            if let Some(transaction_message) = &transaction.transaction {
                if let Some(message) = &transaction_message.message {
                    if matches_pump_fun_pattern(&message) {                       
                        // println!("Pump.fun launch detected. Transaction {}", bs58::encode(&transaction.signature.as_slice()).into_string());
                        if let Some(token_key) = message.account_keys.get(0 as usize){
                            if let Ok(pubkey) = Pubkey::try_from(token_key.as_slice()) {
                                // println!("  Developer account: {}", pubkey.to_string());
                            }
                        };
                        if let Some(token_key) = message.account_keys.get(1 as usize){
                            // let mut keys : Vec<String> = vec![];
                            // for account in message.account_keys.clone() {
                            //     let pkey = Pubkey::try_from(account.as_slice()).unwrap();
                            //     keys.push(pkey.to_string());
                            // }
                            // println!("Account keys: {:?}", keys);
                            if let Ok(pubkey) = Pubkey::try_from(token_key.as_slice()) {
                                let event_sender = self.event_sender.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = event_sender.send(TokenInfo { mint_address: pubkey.to_string(), ..Default::default()}).await {
                                        // eprintln!("Failed to send token info: {:?}", e);
                                    }
                                });
                                // println!("  Pump.fun token: {}\n", pubkey.to_string());
                            }
                        };
                    }
                }
            }
        }
    }

    /// Creates a new GrpcStreamManager instance
    /// 
    /// # Arguments
    /// * `endpoint` - The gRPC endpoint URL
    /// * `x_token` - Authentication token for the endpoint
    pub async fn new(endpoint: &str, x_token: &str, event_sender: mpsc::Sender<TokenInfo>) -> Result<Arc<Mutex<GrpcStreamManager>>> {
        let interceptor = InterceptorXToken {
            x_token: Some(x_token.parse().map_err(|e: InvalidMetadataValue| anyhow::Error::from(e))?),
            x_request_snapshot: true,
        };

        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(10))
            .connect()
            .await
            .map_err(|e| anyhow::Error::from(e))?;

        let client: GeyserGrpcClient<InterceptorXToken> = GeyserGrpcClient::new(
            HealthClient::with_interceptor(channel.clone(), interceptor.clone()),
            GeyserClient::with_interceptor(channel, interceptor),
        );
        Ok(Arc::new(Mutex::new(GrpcStreamManager {
            client,
            is_connected: false,
            reconnect_attempts: 0,
            max_reconnect_attempts: 10,
            reconnect_interval: Duration::from_secs(5),
            event_sender
        })))
    }

    /// Establishes connection and handles the subscription stream
    /// 
    /// # Arguments
    /// * `request` - The subscription request containing transaction filters and other parameters
    pub async fn connect(&mut self, request: SubscribeRequest) -> Result<()> {
        let request = request.clone();
        let (mut subscribe_tx, mut stream) = self.client.subscribe_with_request(Some(request.clone())).await?;

        self.is_connected = true;
        self.reconnect_attempts = 0;

        while let Some(message) = stream.next().await {
            match message {
                Ok(msg) => {
                    match msg.update_oneof {
                        Some(UpdateOneof::Transaction(transaction)) => {
                            self.handle_transaction_update(&transaction);
                        }
                        Some(UpdateOneof::Ping(_)) => {
                            subscribe_tx
                                .send(SubscribeRequest {
                                    ping: Some(SubscribeRequestPing { id: 1 }),
                                    ..Default::default()
                                })
                                .await?;
                        }
                        Some(UpdateOneof::Pong(_)) => {} // Ignore pong responses
                        _ => {
                            println!("Other update received: {:?}", msg);
                        }
                    }
                }
                Err(err) => {
                    error!("Error: {:?}", err);
                    self.is_connected = false;
                    Box::pin(self.reconnect(request.clone())).await?;
                    break;
                }
            }
        }

        Ok(())
    }

    /// Attempts to reconnect when the connection is lost
    /// 
    /// # Arguments
    /// * `request` - The original subscription request to reestablish the connection
    pub async fn reconnect(&mut self, request: SubscribeRequest) -> Result<()> {
        if self.reconnect_attempts >= self.max_reconnect_attempts {
            println!("Max reconnection attempts reached");
            return Ok(());
        }

        self.reconnect_attempts += 1;
        println!("Reconnecting... Attempt {}", self.reconnect_attempts);

        let backoff = self.reconnect_interval * std::cmp::min(self.reconnect_attempts, 5);
        tokio::time::sleep(backoff).await;

        Box::pin(self.connect(request)).await
    }
}