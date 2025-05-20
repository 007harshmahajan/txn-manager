# Transaction Manager

A backend service for managing transactions and user accounts in a financial system.

## Quick Start

To quickly start the Transaction Manager, simply run:

```bash
# Clone the repository
git clone https://github.com/yourusername/txn-manager.git
cd txn-manager

# Run the setup script (requires Docker and Docker Compose)
./setup.sh
```

That's it! The application will be available at http://localhost:8080.

To stop the application:
```bash
docker-compose down
```

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

## Database Schema

The system uses a PostgreSQL database with three main tables:
- **Users**: Stores user information and authentication details
- **Accounts**: Tracks financial accounts belonging to users
- **Transactions**: Records all financial transactions between accounts

For detailed schema information, see [Database Schema](docs/DATABASE_SCHEMA.md).

## Tools

- **pgAdmin**: Access the database management interface at http://localhost:5050
  - Email: admin@example.com
  - Password: admin
- **API Testing**: Run the included test script: `scripts/test_api.sh`

## Documentation

Detailed documentation is available in the `docs` directory:

- [Setup Instructions](docs/SETUP_INSTRUCTIONS.md): Alternative setup methods and troubleshooting
- [API Documentation](docs/API_DOCUMENTATION.md): Detailed API endpoints and usage
- [API Samples](docs/API_SAMPLES.md): Sample requests and testing examples
- [Database Schema](docs/DATABASE_SCHEMA.md): Detailed database structure and relationships
- [Performance Information](docs/PERFORMANCE.md): Performance analysis and optimizations
- [Building from Source](docs/BUILDING.md): Comprehensive build instructions

## License

This project is licensed under the MIT License - see the LICENSE file for details.
