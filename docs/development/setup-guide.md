# Development Setup Guide

## Overview

This guide walks you through setting up a local development environment for the Gotong Royong platform.

## Prerequisites

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| **Docker** | 20.10+ | Container runtime |
| **Docker Compose** | 2.0+ | Multi-container orchestration |
| **Node.js** (or Python/Rust) | 18+ | Application runtime |
| **Git** | 2.30+ | Version control |
| **PostgreSQL Client** | 14+ | Database CLI (optional) |
| **Redis CLI** | 7+ | Cache CLI (optional) |

### Install Prerequisites

**macOS** (using Homebrew):
```bash
brew install docker docker-compose node git postgresql redis
```

**Ubuntu**:
```bash
# Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# PostgreSQL client
sudo apt-get install -y postgresql-client

# Redis client
sudo apt-get install -y redis-tools
```

**Windows**:
- Install [Docker Desktop](https://www.docker.com/products/docker-desktop)
- Install [Node.js](https://nodejs.org/)
- Install [Git for Windows](https://git-scm.com/download/win)

## Quick Start (Docker Compose)

### 1. Clone Repository

```bash
git clone https://github.com/your-org/gotong-royong.git
cd gotong-royong
```

### 2. Create Environment File

```bash
cp .env.example .env.local
```

**Edit `.env.local`**:
```bash
# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/gotong_royong_dev

# Redis
REDIS_URL=redis://localhost:6379

# S3 (MinIO for local dev)
S3_ENDPOINT=http://localhost:9000
S3_BUCKET=gotong-royong-evidence-dev
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin

# Webhook
GOTONG_ROYONG_WEBHOOK_SECRET=dev_secret_32_chars_minimum_here

# Markov Engine
MARKOV_API_URL=http://localhost:3001

# JWT
JWT_SECRET=dev_jwt_secret_32_chars_minimum

# App
NODE_ENV=development
PORT=3000
LOG_LEVEL=debug
```

### 3. Start Services

```bash
docker-compose up -d
```

**Services Started**:
- PostgreSQL: `localhost:5432`
- Redis: `localhost:6379`
- MinIO: `localhost:9000` (API), `localhost:9001` (Console)

### 4. Install Dependencies

**Node.js**:
```bash
npm install
```

**Python**:
```bash
pip install -r requirements.txt
```

**Rust**:
```bash
cargo build
```

### 5. Run Database Migrations

**Node.js (Knex)**:
```bash
npx knex migrate:latest
```

**Python (Alembic)**:
```bash
alembic upgrade head
```

**Rust (Diesel)**:
```bash
diesel migration run
```

### 6. Seed Database (Optional)

```bash
npm run seed
# or
python scripts/seed_database.py
# or
cargo run --bin seed
```

### 7. Start Application

**Node.js**:
```bash
npm run dev
```

**Python**:
```bash
python app.py
# or with auto-reload
uvicorn app:app --reload
```

**Rust**:
```bash
cargo run
# or with auto-reload
cargo watch -x run
```

### 8. Verify Installation

Open browser: http://localhost:3000/health

**Expected Response**:
```json
{
  "status": "healthy",
  "checks": {
    "database": "ok",
    "redis": "ok",
    "s3": "ok"
  }
}
```

## Manual Setup (Without Docker)

### 1. Install PostgreSQL

**macOS**:
```bash
brew install postgresql@14
brew services start postgresql@14
```

**Ubuntu**:
```bash
sudo apt-get install postgresql-14
sudo systemctl start postgresql
```

**Create Database**:
```bash
createdb gotong_royong_dev
```

### 2. Install Redis

**macOS**:
```bash
brew install redis
brew services start redis
```

**Ubuntu**:
```bash
sudo apt-get install redis-server
sudo systemctl start redis
```

### 3. Install MinIO (S3-compatible storage)

**macOS/Linux**:
```bash
wget https://dl.min.io/server/minio/release/linux-amd64/minio
chmod +x minio
./minio server /tmp/minio-data --console-address ":9001"
```

**Access MinIO Console**: http://localhost:9001
- Username: `minioadmin`
- Password: `minioadmin`

**Create Bucket**:
```bash
# Install mc (MinIO client)
brew install minio/stable/mc

# Configure alias
mc alias set local http://localhost:9000 minioadmin minioadmin

# Create bucket
mc mb local/gotong-royong-evidence-dev
```

### 4. Continue with Steps 4-8 Above

## Development Workflow

### Running the Application

**Development mode** (with hot reload):
```bash
npm run dev
# or
python -m uvicorn app:app --reload
# or
cargo watch -x run
```

**Production mode** (for testing):
```bash
npm start
# or
gunicorn app:app
# or
cargo run --release
```

### Running Tests

**Unit Tests**:
```bash
npm test
# or
pytest
# or
cargo test
```

**Integration Tests**:
```bash
npm run test:integration
# or
pytest tests/integration
# or
cargo test --test integration
```

**Watch Mode**:
```bash
npm run test:watch
# or
pytest-watch
# or
cargo watch -x test
```

### Code Formatting

**Node.js**:
```bash
npm run format
npm run lint
```

**Python**:
```bash
black .
flake8 .
mypy .
```

**Rust**:
```bash
cargo fmt
cargo clippy
```

### Database Operations

**Create Migration**:
```bash
# Node.js (Knex)
npx knex migrate:make create_users_table

# Python (Alembic)
alembic revision -m "create users table"

# Rust (Diesel)
diesel migration generate create_users_table
```

**Run Migrations**:
```bash
npm run migrate
# or
alembic upgrade head
# or
diesel migration run
```

**Rollback Migration**:
```bash
npm run migrate:rollback
# or
alembic downgrade -1
# or
diesel migration revert
```

**Reset Database**:
```bash
npm run db:reset
# or
alembic downgrade base && alembic upgrade head
# or
diesel database reset
```

## Connecting to Services

### PostgreSQL

**Command Line**:
```bash
psql postgresql://postgres:postgres@localhost:5432/gotong_royong_dev
```

**GUI Tools**:
- [pgAdmin](https://www.pgadmin.org/)
- [DBeaver](https://dbeaver.io/)
- [TablePlus](https://tableplus.com/)

### Redis

**Command Line**:
```bash
redis-cli
```

**Commands**:
```bash
# List all keys
KEYS *

# Get value
GET reputation:user123

# Delete key
DEL reputation:user123

# Flush all data (careful!)
FLUSHALL
```

**GUI Tools**:
- [RedisInsight](https://redis.com/redis-enterprise/redis-insight/)
- [Another Redis Desktop Manager](https://github.com/qishibo/AnotherRedisDesktopManager)

### MinIO (S3)

**Console**: http://localhost:9001

**CLI**:
```bash
# List buckets
mc ls local

# List objects
mc ls local/gotong-royong-evidence-dev

# Upload file
mc cp photo.jpg local/gotong-royong-evidence-dev/photos/2026/02/10/

# Download file
mc cp local/gotong-royong-evidence-dev/photos/2026/02/10/photo.jpg ./
```

## Environment Variables Reference

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@host:5432/db` |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `S3_ENDPOINT` | S3 endpoint URL | `http://localhost:9000` |
| `S3_BUCKET` | S3 bucket name | `gotong-royong-evidence-dev` |
| `S3_ACCESS_KEY` | S3 access key | `minioadmin` |
| `S3_SECRET_KEY` | S3 secret key | `minioadmin` |
| `GOTONG_ROYONG_WEBHOOK_SECRET` | Webhook HMAC secret | Min 32 characters |
| `JWT_SECRET` | JWT signing secret | Min 32 characters |

### Optional Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | API server port | `3000` |
| `NODE_ENV` | Environment | `development` |
| `LOG_LEVEL` | Logging level | `info` |
| `MARKOV_API_URL` | Markov Engine URL | `http://localhost:3001` |

## Troubleshooting

### Port Already in Use

**Error**: `EADDRINUSE: address already in use :::3000`

**Solution**:
```bash
# Find process using port 3000
lsof -i :3000

# Kill process
kill -9 <PID>

# Or use different port
PORT=3001 npm run dev
```

### Database Connection Failed

**Error**: `connection refused`

**Solutions**:
```bash
# Check if PostgreSQL is running
docker ps
# or
brew services list
# or
sudo systemctl status postgresql

# Restart PostgreSQL
docker-compose restart db
# or
brew services restart postgresql
```

### Redis Connection Failed

**Error**: `Redis connection to localhost:6379 failed`

**Solutions**:
```bash
# Check if Redis is running
redis-cli ping

# Restart Redis
docker-compose restart redis
# or
brew services restart redis
```

### MinIO Connection Failed

**Error**: `S3 connection failed`

**Solutions**:
```bash
# Check if MinIO is running
curl http://localhost:9000/minio/health/live

# Restart MinIO
docker-compose restart minio
```

### Migration Failed

**Error**: `migration failed: relation already exists`

**Solutions**:
```bash
# Rollback and retry
npm run migrate:rollback
npm run migrate

# Or reset database
npm run db:reset
```

### Module Not Found

**Error**: `Cannot find module 'express'`

**Solution**:
```bash
# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Or clear npm cache
npm cache clean --force
npm install
```

## IDE Setup

### Visual Studio Code

**Recommended Extensions**:
- ESLint
- Prettier
- PostgreSQL (Chris Kolkman)
- REST Client
- Docker
- GitLens

**Settings** (`.vscode/settings.json`):
```json
{
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "eslint.validate": [
    "javascript",
    "typescript"
  ]
}
```

**Launch Configuration** (`.vscode/launch.json`):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "node",
      "request": "launch",
      "name": "Debug API",
      "skipFiles": ["<node_internals>/**"],
      "program": "${workspaceFolder}/src/index.js",
      "envFile": "${workspaceFolder}/.env.local"
    }
  ]
}
```

### IntelliJ IDEA / WebStorm

**Recommended Plugins**:
- Database Tools
- Docker
- .env files support

**Run Configuration**:
1. Run → Edit Configurations
2. Add → Node.js
3. Name: "Development Server"
4. JavaScript file: `src/index.js`
5. Environment variables: Load from `.env.local`

## Next Steps

After setup:
1. Review [Local Development](local-development.md) for development workflow
2. Review [Testing Integration](testing-integration.md) for testing strategies
3. Review [API Documentation](../api/webhook-spec.md) for API details
4. Join team Slack channel for questions

## Getting Help

- **Documentation**: Review docs in `/docs` directory
- **Issues**: Check GitHub Issues
- **Slack**: #gotong-royong-dev channel
- **Email**: dev@gotong-royong.app

## References

- [Local Development](local-development.md) - Development workflow
- [Testing Integration](testing-integration.md) - Testing guide
- [Database Schema](../database/schema-requirements.md) - Database design
- [Architecture Overview](../architecture/system-overview.md) - System architecture
