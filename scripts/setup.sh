#!/bin/bash

# Setup script for Transaction Manager
set -e

# Print colorful messages
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=======================================${NC}"
echo -e "${BLUE}   Transaction Manager Setup Script    ${NC}"
echo -e "${BLUE}=======================================${NC}"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${YELLOW}Docker is not installed. Please install Docker first:${NC}"
    echo -e "https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null && ! command -v docker compose &> /dev/null; then
    echo -e "${YELLOW}Docker Compose is not installed. Please install Docker Compose:${NC}"
    echo -e "https://docs.docker.com/compose/install/"
    exit 1
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo -e "${BLUE}Creating .env file...${NC}"
    cp .env.example .env
    # Generate a random JWT secret
    if command -v openssl &> /dev/null; then
        JWT_SECRET=$(openssl rand -hex 32)
        sed -i "s/your_jwt_secret_key_here_change_in_production/$JWT_SECRET/g" .env
        echo -e "${GREEN}Generated secure JWT secret${NC}"
    else
        echo -e "${YELLOW}OpenSSL not found - using default JWT secret. Please change this in production!${NC}"
    fi
fi

# Start the services using Docker Compose
echo -e "${BLUE}Starting services with Docker Compose...${NC}"

# Use 'docker compose' or 'docker-compose' depending on which is available
if command -v docker compose &> /dev/null; then
    docker compose down
    docker compose build
    docker compose up -d
elif command -v docker-compose &> /dev/null; then
    docker-compose down
    docker-compose build
    docker-compose up -d
fi

echo -e "${BLUE}Waiting for services to be ready...${NC}"
# Wait for the application to be ready (max 60 seconds)
MAX_ATTEMPTS=30
ATTEMPT=0
while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if curl -s http://localhost:8080/ &> /dev/null; then
        echo -e "${GREEN}Services are up and running!${NC}"
        break
    fi
    ATTEMPT=$((ATTEMPT+1))
    echo -e "Waiting for services to start... ($ATTEMPT/$MAX_ATTEMPTS)"
    sleep 2
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
    echo -e "${YELLOW}Timed out waiting for services to start. Please check logs with 'docker-compose logs'${NC}"
    exit 1
fi

# Print usage information
echo -e "${GREEN}=======================================${NC}"
echo -e "${GREEN}   Transaction Manager is now running!  ${NC}"
echo -e "${GREEN}=======================================${NC}"
echo -e ""
echo -e "API is available at: ${BLUE}http://localhost:8080/${NC}"
echo -e ""
echo -e "Default user credentials for testing:"
echo -e "Username: ${BLUE}admin${NC}"
echo -e "Password: ${BLUE}password${NC}"
echo -e ""
echo -e "Database is available at:"
echo -e "Host: ${BLUE}localhost${NC}"
echo -e "Port: ${BLUE}5433${NC}"
echo -e "Username: ${BLUE}postgres${NC}"
echo -e "Password: ${BLUE}postgres${NC}"
echo -e "Database: ${BLUE}txn_manager${NC}"
echo -e ""
echo -e "Useful commands:"
echo -e "- View logs: ${BLUE}docker-compose logs -f${NC}"
echo -e "- Stop services: ${BLUE}docker-compose down${NC}"
echo -e "- Restart services: ${BLUE}docker-compose restart${NC}"
echo -e ""
echo -e "For more information, see the README.md file." 