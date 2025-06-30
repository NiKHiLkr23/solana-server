# Solana Server

A simple Rust-based REST API server for basic Solana operations on devnet. This server provides endpoints for account creation, airdrops, and transfers.

## Features

- ✅ Create new Solana keypairs/accounts
- ✅ Request SOL airdrops on devnet
- ✅ Transfer SOL between accounts
- ✅ Get account information and balances
- ✅ Built with Axum for high performance
- ✅ Comprehensive error handling
- ✅ JSON API responses

## Prerequisites

- Rust 1.70+ 
- Cargo

## Installation

1. Clone or copy this project
2. Navigate to the project directory:
   ```bash
   cd solana-server
   ```
3. Copy the environment file:
   ```bash
   cp .env.example .env
   ```
4. Install dependencies and run:
   ```bash
   cargo run
   ```

The server will start on `http://localhost:3000` by default.

## API Endpoints

### Health Check
```
GET /health
```
Returns server health status.

### Account Operations

#### Create Account
```
POST /account/create
Content-Type: application/json

{
  "save_private_key": true  // optional, defaults to false
}
```

Response:
```json
{
  "public_key": "8rKwPkMc...",
  "private_key": "base64_encoded_key", // only if save_private_key is true
  "message": "Account created successfully..."
}
```

#### Get Account Info
```
GET /account/{public_key}
```

Response:
```json
{
  "public_key": "8rKwPkMc...",
  "balance_sol": 1.5,
  "balance_lamports": 1500000000,
  "executable": false,
  "owner": "11111111111111111111111111111112",
  "rent_epoch": 361
}
```

### Airdrop Operations

#### Request Airdrop
```
POST /airdrop
Content-Type: application/json

{
  "public_key": "8rKwPkMc...",
  "amount_sol": 1.0  // Max 5.0 SOL on devnet
}
```

Response:
```json
{
  "transaction_signature": "2Kh7s8...",
  "public_key": "8rKwPkMc...",
  "amount_sol": 1.0,
  "amount_lamports": 1000000000,
  "message": "Successfully airdropped 1 SOL to account"
}
```

### Transfer Operations

#### Transfer SOL
```
POST /transfer
Content-Type: application/json

{
  "from_private_key": "base64_encoded_private_key",
  "to_public_key": "recipient_public_key",
  "amount_sol": 0.5
}
```

Response:
```json
{
  "transaction_signature": "3Bh9s7...",
  "from_public_key": "sender_public_key",
  "to_public_key": "recipient_public_key",
  "amount_sol": 0.5,
  "amount_lamports": 500000000,
  "message": "Successfully transferred 0.5 SOL"
}
```

## Usage Examples

### 1. Create an Account
```bash
curl -X POST http://localhost:3000/account/create \
  -H "Content-Type: application/json" \
  -d '{"save_private_key": true}'
```

### 2. Request Airdrop
```bash
curl -X POST http://localhost:3000/airdrop \
  -H "Content-Type: application/json" \
  -d '{
    "public_key": "YOUR_PUBLIC_KEY",
    "amount_sol": 2.0
  }'
```

### 3. Check Balance
```bash
curl http://localhost:3000/account/YOUR_PUBLIC_KEY
```

### 4. Transfer SOL
```bash
curl -X POST http://localhost:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{
    "from_private_key": "YOUR_BASE64_PRIVATE_KEY",
    "to_public_key": "RECIPIENT_PUBLIC_KEY", 
    "amount_sol": 0.1
  }'
```

## Environment Variables

- `PORT`: Server port (default: 3000)
- `ENV`: Environment mode (LOCAL/PRODUCTION)
- `SOLANA_RPC_URL`: Solana RPC endpoint (default: devnet)

## Error Handling

The API returns structured error responses:

```json
{
  "error": "Error description",
  "status": 400
}
```

Common error codes:
- `400`: Bad Request (invalid input, insufficient funds)
- `502`: Bad Gateway (Solana RPC issues)
- `500`: Internal Server Error

## Development

Run in development mode with auto-reload:
```bash
cargo watch -x run
```

Run tests:
```bash
cargo test
```

## Security Notes

⚠️ **Important**: This is a development/testing server for devnet only. Never use this in production or with mainnet without proper security measures:

- Private keys are handled in memory and requests
- No authentication or rate limiting
- No input sanitization beyond basic validation
- Suitable for development and testing only

## License

This is a boilerplate project for educational and development purposes. 