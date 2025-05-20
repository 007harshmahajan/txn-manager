#!/bin/bash

# API Testing Script for Transaction Manager
BASE_URL="http://localhost:8080/api/v1"
TOKEN=""
ACCOUNT_ID=""
SECOND_ACCOUNT_ID=""

echo "Testing Transaction Manager API..."

# 1. Register a user
echo "1. Registering a user..."
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/users/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "apitester",
    "email": "api@test.com",
    "password": "testpassword123",
    "first_name": "API",
    "last_name": "Tester"
  }')
echo "$REGISTER_RESPONSE"

# 2. Login and get token
echo "2. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/users/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "apitester",
    "password": "testpassword123"
  }')
echo "$LOGIN_RESPONSE"

# Extract token
TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)
echo "Token: $TOKEN"

# 3. Get user profile
echo "3. Getting user profile..."
curl -s -X GET "$BASE_URL/users/me" \
  -H "Authorization: Bearer $TOKEN"

# 4. Get user accounts
echo "4. Getting user accounts..."
ACCOUNTS_RESPONSE=$(curl -s -X GET "$BASE_URL/accounts" \
  -H "Authorization: Bearer $TOKEN")
echo "$ACCOUNTS_RESPONSE"

# Extract account ID
ACCOUNT_ID=$(echo "$ACCOUNTS_RESPONSE" | grep -o '"id":"[^"]*' | head -1 | cut -d'"' -f4)
echo "Account ID: $ACCOUNT_ID"

# 5. Create a new account
echo "5. Creating a new account..."
NEW_ACCOUNT_RESPONSE=$(curl -s -X POST "$BASE_URL/accounts" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "currency": "EUR"
  }')
echo "$NEW_ACCOUNT_RESPONSE"

# Extract second account ID
SECOND_ACCOUNT_ID=$(echo "$NEW_ACCOUNT_RESPONSE" | grep -o '"id":"[^"]*' | head -1 | cut -d'"' -f4)
echo "Second Account ID: $SECOND_ACCOUNT_ID"

# 6. Deposit to account
echo "6. Creating a deposit..."
curl -s -X POST "$BASE_URL/transactions/deposit" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"account_id\": \"$ACCOUNT_ID\",
    \"amount\": \"100.00\",
    \"description\": \"Initial deposit\"
  }"

# 7. Make a transfer
echo "7. Creating a transfer..."
curl -s -X POST "$BASE_URL/transactions/transfer" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"sender_account_id\": \"$ACCOUNT_ID\",
    \"receiver_account_id\": \"$SECOND_ACCOUNT_ID\",
    \"amount\": \"25.00\",
    \"description\": \"Test transfer\"
  }"

# 8. Get account transactions
echo "8. Getting account transactions..."
curl -s -X GET "$BASE_URL/transactions/account/$ACCOUNT_ID" \
  -H "Authorization: Bearer $TOKEN"

echo "API testing complete!" 