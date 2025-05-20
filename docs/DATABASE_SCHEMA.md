# Database Schema

This document details the database schema for the Transaction Manager application.

## Overview

The Transaction Manager uses a PostgreSQL database with three main tables:
- **Users**: Stores user information and authentication details
- **Accounts**: Tracks financial accounts belonging to users
- **Transactions**: Records all financial transactions between accounts

## Entity-Relationship Diagram

```
┌─────────────┐       ┌─────────────┐       ┌─────────────┐
│   Users     │       │  Accounts   │       │Transactions │
├─────────────┤       ├─────────────┤       ├─────────────┤
│ id          │       │ id          │       │ id          │
│ username    │       │ user_id     │─────┐ │ sender_id   │─┐
│ email       │       │ balance     │     │ │ receiver_id │ │
│ password    │  1:N  │ currency    │  1:N│ │ amount      │ │
│ first_name  ├───────┤ created_at  │     └─┤ currency    │ │
│ last_name   │       │ updated_at  │       │ type        │ │
│ created_at  │       └─────────────┘       │ status      │ │
│ updated_at  │                             │ description │ │
└─────────────┘                             │ created_at  │ │
                                            │ updated_at  │ │
                                            └──────┬──────┘ │
                                                   │        │
                                                   └────────┘
```

## Tables

### Users Table

Stores user information and authentication details.

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(50),
    last_name VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

#### Fields:
- **id**: UUID primary key
- **username**: Unique username (3-50 characters)
- **email**: Unique email address
- **password_hash**: Bcrypt-hashed password
- **first_name**: Optional first name
- **last_name**: Optional last name
- **created_at**: Timestamp of user creation
- **updated_at**: Timestamp of last update

### Accounts Table

Stores account information, with each account belonging to a user.

```sql
CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    balance DECIMAL(19, 4) NOT NULL DEFAULT 0.0,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT balance_non_negative CHECK (balance >= 0)
);

CREATE INDEX idx_accounts_user ON accounts(user_id);
```

#### Fields:
- **id**: UUID primary key
- **user_id**: Foreign key to the users table
- **balance**: Account balance with 4 decimal places
- **currency**: 3-letter currency code (e.g., "USD")
- **created_at**: Timestamp of account creation
- **updated_at**: Timestamp of last update

#### Constraints:
- **balance_non_negative**: Ensures balance cannot be negative
- **Foreign key**: Cascading delete if user is deleted

#### Indices:
- **idx_accounts_user**: Index on user_id for faster account lookup by user

### Transactions Table

Records all financial transactions between accounts.

```sql
CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    sender_account_id UUID REFERENCES accounts(id),
    receiver_account_id UUID REFERENCES accounts(id),
    amount DECIMAL(19, 4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    transaction_type VARCHAR(10) NOT NULL CHECK (transaction_type IN ('TRANSFER', 'DEPOSIT', 'WITHDRAWAL')),
    status VARCHAR(10) NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'COMPLETED', 'FAILED')),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT amount_positive CHECK (amount > 0),
    CONSTRAINT transaction_not_self CHECK (
        (transaction_type = 'TRANSFER' AND sender_account_id IS NOT NULL AND receiver_account_id IS NOT NULL AND sender_account_id != receiver_account_id) OR
        (transaction_type = 'DEPOSIT' AND sender_account_id IS NULL AND receiver_account_id IS NOT NULL) OR
        (transaction_type = 'WITHDRAWAL' AND sender_account_id IS NOT NULL AND receiver_account_id IS NULL)
    )
);

CREATE INDEX idx_transactions_sender ON transactions(sender_account_id);
CREATE INDEX idx_transactions_receiver ON transactions(receiver_account_id);
```

#### Fields:
- **id**: UUID primary key
- **sender_account_id**: Foreign key to the sender's account (null for deposits)
- **receiver_account_id**: Foreign key to the receiver's account (null for withdrawals)
- **amount**: Transaction amount with 4 decimal places
- **currency**: 3-letter currency code
- **transaction_type**: Type of transaction ('TRANSFER', 'DEPOSIT', 'WITHDRAWAL')
- **status**: Transaction status ('PENDING', 'COMPLETED', 'FAILED')
- **description**: Optional transaction description
- **created_at**: Timestamp of transaction creation
- **updated_at**: Timestamp of last update

#### Constraints:
- **amount_positive**: Ensures amount is always positive
- **transaction_not_self**: Complex constraint ensuring:
  - Transfers have both sender and receiver (different accounts)
  - Deposits have only receiver
  - Withdrawals have only sender

#### Indices:
- **idx_transactions_sender**: Index on sender_account_id
- **idx_transactions_receiver**: Index on receiver_account_id

## Relationships

1. **User-to-Account**: One-to-many relationship
   - One user can have multiple accounts
   - Each account belongs to exactly one user
   - Enforced by foreign key constraint on accounts.user_id

2. **Account-to-Transaction**: One-to-many relationship
   - One account can be involved in multiple transactions
   - Each transaction involves one or two accounts
   - Enforced by foreign key constraints on transactions.sender_account_id and transactions.receiver_account_id

## Database Migration

The schema is managed through SQLx migrations. The initial migration script is located in `/migrations/20240101000001_initial_schema.sql`.

## Considerations

- **Decimal Precision**: Financial values use DECIMAL(19, 4) for high precision without floating-point issues
- **Transactions**: Database transactions are used for all financial operations to ensure consistency
- **Indices**: Strategic indices improve query performance, especially for account and transaction lookups
- **Constraints**: Business rules are enforced at the database level through constraints 