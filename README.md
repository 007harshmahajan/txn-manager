# Dodo Payments - Transaction Manager

A backend service for managing transactions and user accounts in a simplified version of Dodo Payments' system.

## Features

- User Management: Registration, authentication, and profile management
- Transaction Management: Create, view, and list transactions
- Account Management: Balance tracking, multiple accounts per user
- JWT Authentication: Secure API access
- PostgreSQL Database: Reliable data storage

## Tech Stack

- [Rust](https://www.rust-lang.org/): Primary programming language
- [Axum](https://github.com/tokio-rs/axum): Web framework
- [SQLx](https://github.com/launchbadge/sqlx): Database interactions
- [PostgreSQL](https://www.postgresql.org/): Relational database
- [Docker](https://www.docker.com/): Containerization

## Setup and Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.75 or later)
- [PostgreSQL](https://www.postgresql.org/download/) (14 or later)
- [Docker](https://docs.docker.com/get-docker/) (optional, for containerized deployment)

### Local Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/txn-manager.git
   cd txn-manager
   ```

2. Create a `.env` file based on the example:
   ```bash
   cp .env.example .env
   # Edit the .env file with your configuration
   ```

3. Set up the database:
   ```bash
   # Create a PostgreSQL database named 'txn_manager'
   createdb txn_manager
   ```

4. Run the application:
   ```bash
   cargo run
   ```

### Using Docker

1. Build and start the containers:
   ```bash
   docker-compose up -d
   ```

2. The application will be available at http://localhost:8080

## API Documentation

### Authentication

All endpoints except for registration and login require authentication via JWT Bearer token.

Include the token in the Authorization header:
```
Authorization: Bearer <your_token>
```

### User Endpoints

#### Register a new user

```
POST /api/v1/users/register
```

Request body:
```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "securepassword",
  "first_name": "John",
  "last_name": "Doe"
}
```

Response:
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
POST /api/v1/users/login
```

Request body:
```json
{
  "username": "johndoe",
  "password": "securepassword"
}
```

Response:
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

#### Get Current User

```
GET /api/v1/users/me
```

Response:
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

#### Update Profile

```
PUT /api/v1/users/profile
```

Request body:
```json
{
  "first_name": "Johnny",
  "last_name": "Doe"
}
```

Response:
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

### Account Endpoints

#### Get User Accounts

```
GET /api/v1/accounts
```

Response:
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
GET /api/v1/accounts/:id
```

Response:
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
POST /api/v1/accounts
```

Request body:
```json
{
  "currency": "EUR"
}
```

Response:
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

### Transaction Endpoints

#### Create Transaction

```
POST /api/v1/transactions
```

Request body:
```json
{
  "transaction_type": "DEPOSIT",
  "receiver_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
  "amount": "500.00",
  "currency": "USD",
  "description": "Initial deposit"
}
```

Response:
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
POST /api/v1/transactions/transfer
```

Request body:
```json
{
  "sender_account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
  "receiver_account_id": "c3d4e5f6-a7b8-9012-cdef-3456789abcde",
  "amount": "100.00",
  "description": "Payment for services"
}
```

Response:
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
POST /api/v1/transactions/deposit
```

Request body:
```json
{
  "account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
  "amount": "200.00",
  "description": "Monthly deposit"
}
```

Response:
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
POST /api/v1/transactions/withdrawal
```

Request body:
```json
{
  "account_id": "b2c3d4e5-f6a7-8901-bcde-23456789abcd",
  "amount": "50.00",
  "description": "ATM withdrawal"
}
```

Response:
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

#### Get Transaction Details

```
GET /api/v1/transactions/:id
```

Response:
```json
{
  "status": "success",
  "message": "Transaction retrieved successfully",
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

#### Get Account Transactions

```
GET /api/v1/transactions/account/:id?limit=10&offset=0
```

Response:
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

## Error Handling

The API returns appropriate HTTP status codes and structured error responses:

```json
{
  "error": "BAD_REQUEST",
  "message": "Insufficient funds"
}
```

Common error codes:
- 400 Bad Request: Invalid input data
- 401 Unauthorized: Missing or invalid authentication
- 403 Forbidden: Insufficient permissions
- 404 Not Found: Resource not found
- 409 Conflict: Resource already exists (e.g., username)
- 500 Internal Server Error: Server-side error

## Testing

### Running Unit Tests

```bash
cargo test
```

### Running Integration Tests

```bash
cargo test -- --test-threads=1 --ignored
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request 