# API Sample Requests

This document provides sample API requests that you can use to test the Transaction Manager API.

## Using curl

Below are curl commands for the main API endpoints. Replace `localhost:8080` with your server address if running elsewhere.

### User Management

#### Register a New User

```bash
curl -X POST http://localhost:8080/api/v1/users/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "securepassword123",
    "first_name": "Test",
    "last_name": "User"
  }'
```

#### Login

```bash
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "securepassword123"
  }'
```

*The login response will contain a JWT token that you'll need for authenticated requests. Replace `{YOUR_TOKEN}` in the following requests with that token.*

#### Get Current User Profile

```bash
curl -X GET http://localhost:8080/api/v1/users/me \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

#### Update User Profile

```bash
curl -X PUT http://localhost:8080/api/v1/users/profile \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -d '{
    "first_name": "Updated",
    "last_name": "Name"
  }'
```

### Account Management

#### Get User Accounts

```bash
curl -X GET http://localhost:8080/api/v1/accounts \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

#### Get Specific Account

```bash
curl -X GET http://localhost:8080/api/v1/accounts/{ACCOUNT_ID} \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

#### Create New Account

```bash
curl -X POST http://localhost:8080/api/v1/accounts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -d '{
    "currency": "EUR"
  }'
```

### Transaction Management

#### Get Transaction Details

```bash
curl -X GET http://localhost:8080/api/v1/transactions/{TRANSACTION_ID} \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

#### Get Account Transactions

```bash
curl -X GET http://localhost:8080/api/v1/transactions/account/{ACCOUNT_ID}?limit=10&offset=0 \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

#### Create a Deposit

```bash
curl -X POST http://localhost:8080/api/v1/transactions/deposit \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -d '{
    "account_id": "{ACCOUNT_ID}",
    "amount": "100.00",
    "description": "Test deposit"
  }'
```

#### Create a Withdrawal

```bash
curl -X POST http://localhost:8080/api/v1/transactions/withdrawal \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -d '{
    "account_id": "{ACCOUNT_ID}",
    "amount": "50.00",
    "description": "Test withdrawal"
  }'
```

#### Create a Transfer

```bash
curl -X POST http://localhost:8080/api/v1/transactions/transfer \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -d '{
    "sender_account_id": "{SENDER_ACCOUNT_ID}",
    "receiver_account_id": "{RECEIVER_ACCOUNT_ID}",
    "amount": "25.00",
    "description": "Test transfer"
  }'
```

## Postman Collection

To make testing easier, you can download and import our Postman collection:

[Download Transaction Manager API Postman Collection](https://example.com/txn-manager-postman.json)

### Importing the Postman Collection

1. Open Postman
2. Click "Import" in the upper left
3. Upload the downloaded file or drag and drop it
4. Once imported, you'll see a new collection "Transaction Manager API"

### Using the Postman Collection

1. The collection includes environment variables for the API URL and authentication token
2. Run the "Login" request first to obtain a token
3. The token will be automatically set for subsequent requests
4. Organized folders group related endpoints for easier navigation

## API Testing Script

For automated testing of the API endpoints in sequence, you can use the following shell script:

```bash
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
```

Save this script as `test_api.sh` in the project root, make it executable with `chmod +x test_api.sh`, and run it to test the API endpoints in sequence. 