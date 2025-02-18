use {
    anyhow::Result, solana_client::nonblocking::rpc_client::RpcClient, solana_sdk::pubkey::Pubkey, spl_token::instruction::TokenInstruction, sqlx::PgPool, std::sync::Arc, yellowstone_grpc_proto::
        prelude::Message
    ,
};


pub async fn process_new_token_launch(_token_mint: Pubkey, _rpc_client: Arc<RpcClient>) -> Result<()> {
    // let token_info = tokio::task::spawn_blocking({
    //      let rpc_client = rpc_client.clone();
    //      move || {
    //          // Call the synchronous version of fetch_token_details (which returns a Result<TokenInfo, anyhow::Error>)
    //          fetch_token_details(rpc_client, token_mint)
    //      }
    //  })
    //  .await??;
    //  store_token_info(&db_pool, &token_info).await?;
    //  println!("Stored details for token: {}", token_info.mint_address);
     Ok(())
 }
 
pub fn matches_pump_fun_pattern(message: &Message) -> bool {
     let instructions = &message.instructions;
     
     if instructions.len() < 5 {
         return false;
     }
     // 1. Check for InitializeMultisig { m: 138 }
     if let Ok(TokenInstruction::InitializeMultisig { .. }) =
         TokenInstruction::unpack(&instructions[0].data)
     {
     } else {
         return false;
     }
     // 2. Check for Transfer { amount: 2456227 }
     if let Ok(TokenInstruction::Transfer { .. }) =
         TokenInstruction::unpack(&instructions[1].data)
     {
 
     } else {
         return false;
     }
     // 3. Check that unpacking the third instruction fails with Custom(12)
     if let Err(e) = TokenInstruction::unpack(&instructions[2].data) {
         if format!("{:?}", e) != "Custom(12)" {
             return false;
         }
     } else {
         return false;
     }
     // 4. Check for InitializeAccount
     if let Ok(TokenInstruction::InitializeAccount) =
         TokenInstruction::unpack(&instructions[3].data)
     {
         // OK.
     } else {
         return false;
     }
     // 5. Check that unpacking the fifth instruction fails with Custom(12)
     if let Err(e) = TokenInstruction::unpack(&instructions[4].data) {
         if format!("{:?}", e) != "Custom(12)" {
             return false;
         }
     } else {
         return false;
     }
     true
 }
 
 
 fn _print_account_keys(data: Vec<Vec<u8>>) {
     for account in data {
         if let Ok(_pubkey) = Pubkey::try_from(account.as_slice()){
             println!("Account key:")
         }
     }
 }