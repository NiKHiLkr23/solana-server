#!/bin/bash

# Solana Server Test Script
SERVER_URL="http://localhost:3001"

echo "🚀 Testing Solana Server..."

# 1. Health Check
echo "1️⃣ Health Check:"
curl -s $SERVER_URL/health
echo -e "\n"

# 2. Create Account 1
echo "2️⃣ Creating Account 1:"
ACCOUNT1=$(curl -s -X POST $SERVER_URL/account/create \
  -H "Content-Type: application/json" \
  -d '{"save_private_key": true}')
echo $ACCOUNT1 | jq
echo -e "\n"

# Extract keys
PUBKEY1=$(echo $ACCOUNT1 | jq -r '.public_key')
PRIVKEY1=$(echo $ACCOUNT1 | jq -r '.private_key')

# 3. Create Account 2
echo "3️⃣ Creating Account 2:"
ACCOUNT2=$(curl -s -X POST $SERVER_URL/account/create \
  -H "Content-Type: application/json" \
  -d '{"save_private_key": true}')
echo $ACCOUNT2 | jq
echo -e "\n"

PUBKEY2=$(echo $ACCOUNT2 | jq -r '.public_key')

# 4. Request Airdrop for Account 1
echo "4️⃣ Requesting Airdrop for Account 1:"
curl -s -X POST $SERVER_URL/airdrop \
  -H "Content-Type: application/json" \
  -d "{\"public_key\": \"$PUBKEY1\", \"amount_sol\": 2.0}" | jq
echo -e "\n"

# Wait a bit for airdrop to process
echo "⏳ Waiting 5 seconds for airdrop to process..."
sleep 5

# 5. Check Account 1 Balance
echo "5️⃣ Checking Account 1 Balance:"
curl -s $SERVER_URL/account/$PUBKEY1 | jq
echo -e "\n"

# 6. Transfer SOL from Account 1 to Account 2
echo "6️⃣ Transferring 0.5 SOL from Account 1 to Account 2:"
curl -s -X POST $SERVER_URL/transfer \
  -H "Content-Type: application/json" \
  -d "{\"from_private_key\": \"$PRIVKEY1\", \"to_public_key\": \"$PUBKEY2\", \"amount_sol\": 0.5}" | jq
echo -e "\n"

# Wait a bit for transfer to process
echo "⏳ Waiting 5 seconds for transfer to process..."
sleep 5

# 7. Check both account balances
echo "7️⃣ Final Account Balances:"
echo "Account 1:"
curl -s $SERVER_URL/account/$PUBKEY1 | jq
echo "Account 2:"
curl -s $SERVER_URL/account/$PUBKEY2 | jq

echo "✅ Test Complete!" 