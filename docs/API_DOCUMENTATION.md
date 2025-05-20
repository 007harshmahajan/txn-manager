# Transaction Manager API Documentation

## Overview

This document provides detailed information about the API endpoints available in the Transaction Manager service. The service is a RESTful API built with Axum that allows users to manage their accounts and transactions.

## Authentication

All endpoints except for `/api/v1/users/register` and `/api/v1/users/login` require authentication via JWT Bearer token.

**Header format:**
```
Authorization: Bearer <your_jwt_token>
```

If authentication is invalid or missing, the API will respond with a `401 Unauthorized` or `403 Forbidden` status code.

## Base URL

```
http://localhost:8080/api/v1
```

## Response Format

All API responses follow a consistent JSON structure:

```json
{
  "status": "success",
  "message": "A human-readable message",
  "data": {
    // Response data (if applicable)
  }
}
```

For error responses:

```json
{
  "error": "ERROR_CODE",
  "message": "A human-readable error message"
}
```

## Common Error Codes

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | BAD_REQUEST | Invalid input data |
| 401 | UNAUTHORIZED | Missing or invalid authentication |
| 403 | FORBIDDEN | Insufficient permissions |
| 404 | NOT_FOUND | Resource not found |
| 409 | CONFLICT | Resource already exists (e.g., username) |
| 500 | INTERNAL_SERVER_ERROR | Server-side error |

## Endpoints

### User Management

#### Register a New User

```
POST /users/register
```

Register a new user and create a default account.

**Request:**
```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "securepassword",
  "first_name": "John",
  "last_name": "Doe"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "User registered successfully",
  "data": {
    "id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
    "username": "johndoe",
    "email": "john@example.com",
    "first_name": "John",
    "last_name": "Doe"
  }
}
```

#### Login

```
POST /users/login
```

Authenticate a user and receive a JWT token.

**Request:**
```json
{
  "username": "johndoe",
  "password": "securepassword"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Login successful",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
      "username": "johndoe",
      "email": "john@example.com",
      "first_name": "John", 
      "last_name": "Doe"
    }
  }
}
```

#### Get Current User Profile

```
GET /users/me
```

Retrieve the authenticated user's profile.

**Response:**
```json
{
  "status": "success",
  "message": "User profile retrieved",
  "data": {
    "id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
    "username": "johndoe",
    "email": "john@example.com",
    "first_name": "John",
    "last_name": "Doe"
  }
}
```

#### Update User Profile

```
PUT /users/profile
```

Update the authenticated user's profile information.

**Request:**
```json
{
  "first_name": "Johnny",
  "last_name": "Doe"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Profile updated successfully", 
  "data": {
    "id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
    "username": "johndoe",
    "email": "john@example.com",
    "first_name": "Johnny", 
    "last_name": "Doe"
  }
}
```

### Account Management

#### Get User Accounts

```
GET /accounts
```

Retrieve all accounts belonging to the authenticated user.

**Response:**
```json
{
  "status": "success",
  "message": "Accounts retrieved successfully",
  "data": [
    {
      "id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
      "user_id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
      "balance": "1000.0000",
      "currency": "USD",
      "created_at": "2023-03-01T12:00:00Z"
    }
  ]
}
```

#### Get Account Details

```
GET /accounts/:id
```

Retrieve details for a specific account.

**Response:**
```json
{
  "status": "success",
  "message": "Account retrieved successfully",
  "data": {
    "id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
    "user_id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab", 
    "balance": "1000.0000",
    "currency": "USD",
    "created_at": "2023-03-01T12:00:00Z"
  }
}
```

#### Create New Account

```
POST /accounts
```

Create a new account for the authenticated user.

**Request:**
```json
{
  "currency": "EUR"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Account created successfully",
  "data": {
    "id": "c3d4e5f6-a7b8-9012-cdef-3456789abcde",
    "user_id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
    "balance": "0.0000",
    "currency": "EUR",
    "created_at": "2023-03-02T14:30:00Z"
  }
}
```

### Transaction Management

#### Get Transaction Details

```
GET /transactions/:id
```

Retrieve details for a specific transaction.

**Response:**
```json
{
  "status": "success",
  "message": "Transaction retrieved successfully",
  "data": {
    "id": "d4e5f6a7-b8c9-0123-defg-456789abcdef",
    "sender_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
    "receiver_account_id": "c3d4e5f6-a7b8-9012-cdef-3456789abcde",
    "amount": "100.0000",
    "currency": "USD",
    "transaction_type": "TRANSFER",
    "status": "COMPLETED",
    "description": "Payment for services",
    "created_at": "2023-03-03T11:45:00Z"
  }
}
```

#### Create Generic Transaction

```
POST /transactions
```

Create a new transaction with custom type.

**Request:**
```json
{
  "transaction_type": "DEPOSIT",
  "receiver_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
  "amount": "500.00",
  "currency": "USD",
  "description": "Initial deposit"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Transaction created successfully",
  "data": {
    "id": "d4e5f6a7-b8c9-0123-defg-456789abcdef", 
    "sender_account_id": null,
    "receiver_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
    "amount": "500.0000",
    "currency": "USD", 
    "transaction_type": "DEPOSIT",
    "status": "COMPLETED",
    "description": "Initial deposit",
    "created_at": "2023-03-03T10:15:00Z"
  }
}
```

#### Transfer Money

```
POST /transactions/transfer
```

Transfer money between two accounts.

**Request:**
```json
{
  "sender_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
  "receiver_account_id": "c3d4e5f6-a7b8-9012-cdef-3456789abcde",
  "amount": "100.00",
  "description": "Payment for services"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Transfer successful",
  "data": {
    "id": "e5f6a7b8-c9d0-1234-efgh-56789abcdefg",
    "sender_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd", 
    "receiver_account_id": "c3d4e5f6-a7b8-9012-cdef-3456789abcde",
    "amount": "100.0000",
    "currency": "USD",
    "transaction_type": "TRANSFER",
    "status": "COMPLETED",
    "description": "Payment for services",
    "created_at": "2023-03-03T11:45:00Z" 
  }
}
```

#### Deposit Money

```
POST /transactions/deposit
```

Deposit money into an account.

**Request:**
```json
{
  "account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
  "amount": "200.00",
  "description": "Monthly deposit"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Deposit successful",
  "data": {
    "id": "f6a7b8c9-d0e1-2345-fghi-6789abcdefgh",
    "sender_account_id": null, 
    "receiver_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
    "amount": "200.0000",
    "currency": "USD",
    "transaction_type": "DEPOSIT",
    "status": "COMPLETED",
    "description": "Monthly deposit",
    "created_at": "2023-03-04T09:30:00Z"
  }
}
```

#### Withdraw Money

```
POST /transactions/withdrawal
```

Withdraw money from an account.

**Request:**
```json
{
  "account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
  "amount": "50.00",
  "description": "ATM withdrawal"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Withdrawal successful",
  "data": {
    "id": "a7b8c9d0-e1f2-3456-ghij-789abcdefghi",
    "sender_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
    "receiver_account_id": null,
    "amount": "50.0000",
    "currency": "USD", 
    "transaction_type": "WITHDRAWAL",
    "status": "COMPLETED",
    "description": "ATM withdrawal",
    "created_at": "2023-03-05T15:20:00Z"
  }
}
```

#### Get Account Transactions

```
GET /transactions/account/:id
```

Retrieve all transactions for a specific account.

**Query Parameters:**
- `limit` (optional): Maximum number of transactions to return (default: 100)
- `offset` (optional): Number of transactions to skip (default: 0)

**Example:** `/transactions/account/b2c3d4e5-f6a7-8901-bcde-23456789abcd?limit=10&offset=0`

**Response:**
```json
{
  "status": "success",
  "message": "Transactions retrieved successfully",
  "data": [
    {
      "id": "a7b8c9d0-e1f2-3456-ghij-789abcdefghi",
      "sender_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
      "receiver_account_id": null,
      "amount": "50.0000",
      "currency": "USD",
      "transaction_type": "WITHDRAWAL",
      "status": "COMPLETED",
      "description": "ATM withdrawal",
      "created_at": "2023-03-05T15:20:00Z"
    },
    {
      "id": "f6a7b8c9-d0e1-2345-fghi-6789abcdefgh",
      "sender_account_id": null,
      "receiver_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
      "amount": "200.0000",
      "currency": "USD",
      "transaction_type": "DEPOSIT",
      "status": "COMPLETED",
      "description": "Monthly deposit", 
      "created_at": "2023-03-04T09:30:00Z"
    }
  ]
}
```

## Data Models

### User

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Unique identifier |
| username | String | Unique username (3-50 chars) |
| email | String | Unique email address |
| first_name | String (optional) | User's first name |
| last_name | String (optional) | User's last name |
| created_at | DateTime | When the user was created |

### Account

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Unique identifier |
| user_id | UUID | Reference to owner user |
| balance | Decimal | Current account balance |
| currency | String | 3-letter currency code (e.g., "USD") |
| created_at | DateTime | When the account was created |

### Transaction

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Unique identifier |
| sender_account_id | UUID (optional) | Reference to sender account (null for deposits) |
| receiver_account_id | UUID (optional) | Reference to receiver account (null for withdrawals) |
| amount | Decimal | Transaction amount (always positive) |
| currency | String | 3-letter currency code |
| transaction_type | String | TRANSFER, DEPOSIT, or WITHDRAWAL |
| status | String | PENDING, COMPLETED, or FAILED |
| description | String (optional) | Transaction description |
| created_at | DateTime | When the transaction was created |

## Error Handling

The API uses appropriate HTTP status codes and consistent error responses. All error responses include:

1. An appropriate HTTP status code
2. A standardized error code
3. A human-readable error message

Example error response:

```json
{
  "error": "INSUFFICIENT_FUNDS",
  "message": "Insufficient funds to complete this transaction"
}
``` 