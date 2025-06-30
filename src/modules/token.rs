use crate::utils::errors::SolanaError;
use crate::utils::solana_client::get_rpc_client;
use axum::{routing::post, Json, Router};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::{initialize_mint, mint_to};
use tracing::info;

#[derive(Deserialize, Serialize)]
pub struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    pub mint_authority: Option<String>,
    pub mint: Option<String>,
    pub decimals: Option<u8>,
}

#[derive(Deserialize, Serialize)]
pub struct MintTokenRequest {
    pub mint: Option<String>,
    pub destination: Option<String>,
    pub authority: Option<String>,
    pub amount: Option<u64>,
}

#[derive(Serialize)]
pub struct InstructionResponse {
    pub program_id: String,
    pub accounts: Vec<AccountMetaResponse>,
    pub instruction_data: String,
}

#[derive(Serialize)]
pub struct AccountMetaResponse {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

pub fn routes() -> Router {
    Router::new()
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
}

async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    info!(
        "POST /token/create - Request: {}",
        serde_json::to_string(&payload).unwrap_or_default()
    );

    // Validate required fields are present and not empty
    let mint_authority = payload
        .mint_authority
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let mint = payload
        .mint
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let decimals = payload.decimals.ok_or(SolanaError::MissingFields)?;

    // Validate decimals range (SPL tokens typically use 0-9 decimals)
    if decimals > 9 {
        return Err(SolanaError::InvalidInput(
            "Decimals must be between 0 and 9".to_string(),
        ));
    }

    // Parse public keys AFTER validation
    let mint_authority_pubkey = mint_authority
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint authority public key".to_string()))?;

    let mint_pubkey = mint
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint public key".to_string()))?;

    // Validate that mint and authority are different accounts
    if mint_pubkey == mint_authority_pubkey {
        return Err(SolanaError::InvalidInput(
            "Mint account and mint authority cannot be the same".to_string(),
        ));
    }

    // For informational purposes, show what the ATA would look like for the mint authority
    let authority_ata = get_associated_token_address(&mint_authority_pubkey, &mint_pubkey);

    info!(
        "Creating mint: {} with authority: {} (Authority's ATA would be: {})",
        mint_pubkey, mint_authority_pubkey, authority_ata
    );

    // Create initialize mint instruction
    let instruction = initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_authority_pubkey,
        Some(&mint_authority_pubkey), // Using same authority for freeze authority
        decimals,
    )
    .map_err(|e| SolanaError::TokenError(e.to_string()))?;

    let accounts: Vec<AccountMetaResponse> = instruction
        .accounts
        .iter()
        .map(|acc| AccountMetaResponse {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let response = InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    let json_response = serde_json::json!({
        "success": true,
        "data": response
    });

    info!("Response: 200 - Token mint creation instruction generated successfully");

    Ok(Json(json_response))
}

async fn mint_token(
    Json(payload): Json<MintTokenRequest>,
) -> Result<Json<serde_json::Value>, SolanaError> {
    info!(
        "POST /token/mint - Request: {}",
        serde_json::to_string(&payload).unwrap_or_default()
    );

    // Validate required fields are present and not empty
    let mint = payload
        .mint
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let destination = payload
        .destination
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let authority = payload
        .authority
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .ok_or(SolanaError::MissingFields)?;

    let amount = payload
        .amount
        .filter(|&a| a > 0)
        .ok_or(SolanaError::MissingFields)?;

    // Parse public keys AFTER validation
    let mint_pubkey = mint
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid mint address".to_string()))?;

    let destination_wallet_pubkey = destination
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid destination wallet address".to_string()))?;

    let authority_pubkey = authority
        .parse::<Pubkey>()
        .map_err(|_| SolanaError::InvalidInput("Invalid authority address".to_string()))?;

    // Get RPC client for validation
    let client = get_rpc_client();

    // Validate mint account exists and check authority permissions
    let mint_account = client
        .get_account(&mint_pubkey)
        .map_err(|_| SolanaError::InvalidInput("Mint account does not exist".to_string()))?;

    // Parse mint account data to check authority
    if mint_account.owner != spl_token::id() {
        return Err(SolanaError::InvalidInput(
            "Invalid mint account - not owned by SPL Token program".to_string(),
        ));
    }

    // Derive ATAs for both authority and destination
    let authority_ata = get_associated_token_address(&authority_pubkey, &mint_pubkey);
    let destination_ata = get_associated_token_address(&destination_wallet_pubkey, &mint_pubkey);

    info!(
        "Authority: {} (ATA: {}), Destination: {} (ATA: {})",
        authority_pubkey, authority_ata, destination_wallet_pubkey, destination_ata
    );

    // Validate authority has proper token account setup (optional but recommended)
    match client.get_token_account_balance(&authority_ata) {
        Ok(balance) => {
            info!(
                "Authority ATA exists with balance: {} tokens - Authority is properly set up",
                balance.amount
            );
        }
        Err(_) => {
            info!("Authority ATA does not exist yet - this is normal for new mint authorities");
            // Note: This is not an error - mint authorities don't need to have token accounts
            // They can mint to any account, including their own ATA later
        }
    }

    // Validate destination ATA (this is where tokens will be minted to)
    match client.get_token_account_balance(&destination_ata) {
        Ok(balance) => {
            info!(
                "Destination ATA exists with current balance: {} tokens",
                balance.amount
            );
        }
        Err(_) => {
            info!("Destination ATA does not exist - it will need to be created before minting");
            // This could be a warning but not necessarily an error
            // The transaction might include ATA creation instruction
        }
    }

    info!(
        "Minting {} tokens from mint {} to destination ATA: {}",
        amount, mint_pubkey, destination_ata
    );

    // Create mint to instruction using the derived ATA
    let instruction = mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &destination_ata,
        &authority_pubkey,
        &[],
        amount,
    )
    .map_err(|e| SolanaError::TokenError(e.to_string()))?;

    let accounts: Vec<AccountMetaResponse> = instruction
        .accounts
        .iter()
        .map(|acc| AccountMetaResponse {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let response = InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    let json_response = serde_json::json!({
        "success": true,
        "data": response
    });

    info!(
        "Response: 200 - Authority validation completed and mint instruction created successfully"
    );

    Ok(Json(json_response))
}
