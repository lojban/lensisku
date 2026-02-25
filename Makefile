# Makefile for Lojban Lens Search Development Environment

# Variables
DC = docker compose
DC_FILE = docker-compose.dev.yml

# Colors for pretty output
CYAN = \033[0;36m
NC = \033[0m # No Color

# Load environment variables from .env file
include .env
export

# Targets
.PHONY: up down logs ps clean build build-prod help psql

# Default target
help:
	@echo "$(CYAN)Lojban Lens Search Development Environment$(NC)"
	@echo "Usage:"
	@echo "  make up      - Start the development environment"
	@echo "  make down    - Stop the development environment"
	@echo "  make logs    - View logs from all containers"
	@echo "  make ps      - List running containers"
	@echo "  make clean   - Remove all containers and volumes"
	@echo "  make build       - Rebuild dev Docker images (db, redis)"
	@echo "  make build-prod  - Build production app image (faster Rust: uses BuildKit cache)"
	@echo "  make psql    - Enter PostgreSQL container and connect to the database"
	@echo "  make redis   - Enter Redis container and connect to the storage"

up:
	@echo "$(CYAN)Starting development environment...$(NC)"
	$(DC) -f $(DC_FILE) up -d

down:
	@echo "$(CYAN)Stopping development environment...$(NC)"
	$(DC) -f $(DC_FILE) down

flush:
	@echo "$(CYAN)Stopping development environment...$(NC)"
	$(DC) -f $(DC_FILE) down -v

logs:
	@echo "$(CYAN)Viewing logs...$(NC)"
	$(DC) -f $(DC_FILE) logs -f

ps:
	@echo "$(CYAN)Listing containers...$(NC)"
	$(DC) -f $(DC_FILE) ps

clean:
	@echo "$(CYAN)Cleaning up development environment...$(NC)"
	$(DC) -f $(DC_FILE) down -v --remove-orphans

build:
	@echo "$(CYAN)Rebuilding Docker images...$(NC)"
	DOCKER_BUILDKIT=1 $(DC) -f $(DC_FILE) build --no-cache

# Production image (use BuildKit for faster Rust compile via cache mounts)
build-prod:
	@echo "$(CYAN)Building production Docker image...$(NC)"
	DOCKER_BUILDKIT=1 docker build -t lenisku:latest .

# Check for errors in the code
check:
	CXXFLAGS="-std=c++17" cargo check

# Run backend development server
back:
	make up
	@echo "$(CYAN)Waiting for services to be ready...$(NC)"
	sleep 2
	@echo "$(CYAN)Starting backend development server...$(NC)"
	/bin/bash -c "set -a && source .env && set +a && bash ./watch.sh"

# Run frontend development server
front:
	@echo "$(CYAN)Starting frontend development server...$(NC)"
	cd frontend && pnpm dev

# Enter PostgreSQL container and connect to the database
psql:
	@echo "$(CYAN)Entering PostgreSQL container and connecting to $(DB_NAME)...$(NC)"
	$(DC) -f $(DC_FILE) exec db psql -U $(DB_USER) -d $(DB_NAME)

redis:
	@echo "$(CYAN)Entering Redis container...$(NC)"
	$(DC) -f $(DC_FILE) exec redis redis-cli