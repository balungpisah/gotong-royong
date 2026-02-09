# Database Migrations

## Overview

This document outlines the migration strategy for managing database schema changes in the Gotong Royong platform.

## Migration Tools

### Recommended Tools by Stack

| Stack | Tool | Why |
|-------|------|-----|
| Node.js | Knex.js, Sequelize, TypeORM | Mature, TypeScript support |
| Python | Alembic (SQLAlchemy), Django Migrations | Auto-generation, version control |
| Rust | Diesel CLI, sqlx-cli | Type-safe, compile-time checks |
| Go | golang-migrate, goose | Simple, no ORM required |

## Migration Principles

### 1. Version Control

Every schema change MUST:
- Be tracked in version control (Git)
- Have a unique version number
- Include both `up` and `down` migrations
- Be tested in development before production

### 2. Idempotency

Migrations MUST be idempotent (safe to run multiple times):

**❌ Not Idempotent**:
```sql
-- Will fail on second run
CREATE TABLE users (...);
```

**✅ Idempotent**:
```sql
-- Safe to run multiple times
CREATE TABLE IF NOT EXISTS users (...);
```

### 3. Backward Compatibility

New migrations SHOULD be backward compatible when possible:
- Add columns with DEFAULT values
- Use `ALTER TABLE ADD COLUMN IF NOT EXISTS`
- Avoid dropping columns immediately (deprecate first)

### 4. Data Migrations

Separate data migrations from schema migrations:
- Schema migration: Change structure
- Data migration: Transform existing data

## Migration Workflow

### Development Workflow

```
1. Developer creates migration file
   ↓
2. Run migration locally
   ↓
3. Test application against new schema
   ↓
4. Commit migration file
   ↓
5. Code review
   ↓
6. Merge to main
   ↓
7. Deploy to staging
   ↓
8. Run migration on staging
   ↓
9. Test on staging
   ↓
10. Deploy to production
    ↓
11. Run migration on production
```

### File Naming Convention

**Format**: `{timestamp}_{description}.{ext}`

**Examples**:
- `20260210120000_create_users_table.sql`
- `20260210130000_add_markov_user_id_to_users.sql`
- `20260210140000_create_contributions_table.sql`

**Timestamp Format**: `YYYYMMDDHHmmss`

## Migration Examples

### Example 1: Create Initial Tables

**File**: `migrations/20260210120000_create_users_table.sql`

```sql
-- Up Migration
CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  username VARCHAR(50) NOT NULL UNIQUE,
  email VARCHAR(255) NOT NULL UNIQUE,
  password_hash VARCHAR(255) NOT NULL,
  markov_user_id VARCHAR(100) UNIQUE,
  bio TEXT,
  location VARCHAR(255),
  avatar_url TEXT,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_markov_id ON users(markov_user_id);

-- Down Migration
-- DROP TABLE IF EXISTS users CASCADE;
```

### Example 2: Add New Column

**File**: `migrations/20260210130000_add_bio_to_users.sql`

```sql
-- Up Migration
ALTER TABLE users
ADD COLUMN IF NOT EXISTS bio TEXT;

-- Down Migration
-- ALTER TABLE users DROP COLUMN IF EXISTS bio;
```

### Example 3: Create Junction Table

**File**: `migrations/20260210140000_create_task_skills_table.sql`

```sql
-- Up Migration
CREATE TABLE IF NOT EXISTS task_skills (
  task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  skill_id UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
  created_at TIMESTAMP DEFAULT NOW(),
  PRIMARY KEY (task_id, skill_id)
);

CREATE INDEX IF NOT EXISTS idx_task_skills_skill ON task_skills(skill_id);

-- Down Migration
-- DROP TABLE IF EXISTS task_skills;
```

### Example 4: Add Constraint

**File**: `migrations/20260210150000_add_check_constraint_vouches.sql`

```sql
-- Up Migration
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'check_not_self_vouch'
  ) THEN
    ALTER TABLE vouches
    ADD CONSTRAINT check_not_self_vouch CHECK (voucher_id != vouchee_id);
  END IF;
END $$;

-- Down Migration
-- ALTER TABLE vouches DROP CONSTRAINT IF EXISTS check_not_self_vouch;
```

### Example 5: Data Migration

**File**: `migrations/20260210160000_populate_markov_user_ids.sql`

```sql
-- Up Migration
-- Populate markov_user_id for existing users
UPDATE users
SET markov_user_id = CONCAT('gotong_royong:', id::TEXT)
WHERE markov_user_id IS NULL;

-- Down Migration
-- UPDATE users SET markov_user_id = NULL;
```

### Example 6: Add Index for Performance

**File**: `migrations/20260210170000_add_index_contributions_user_status.sql`

```sql
-- Up Migration
CREATE INDEX IF NOT EXISTS idx_contributions_user_status
ON contributions(user_id, verification_status);

-- Down Migration
-- DROP INDEX IF EXISTS idx_contributions_user_status;
```

## Migration Tools Setup

### Knex.js (Node.js)

**Install**:
```bash
npm install knex pg
npm install --save-dev @types/knex
```

**Configure** (`knexfile.js`):
```javascript
module.exports = {
  development: {
    client: 'postgresql',
    connection: {
      host: process.env.DB_HOST || 'localhost',
      database: process.env.DB_NAME || 'gotong_royong_dev',
      user: process.env.DB_USER || 'postgres',
      password: process.env.DB_PASSWORD || 'postgres',
    },
    migrations: {
      directory: './migrations',
      tableName: 'knex_migrations',
    },
  },
  production: {
    client: 'postgresql',
    connection: process.env.DATABASE_URL,
    migrations: {
      directory: './migrations',
      tableName: 'knex_migrations',
    },
  },
};
```

**Create Migration**:
```bash
npx knex migrate:make create_users_table
```

**Run Migrations**:
```bash
npx knex migrate:latest
```

**Rollback**:
```bash
npx knex migrate:rollback
```

**Example Migration** (`migrations/20260210120000_create_users_table.js`):
```javascript
exports.up = function(knex) {
  return knex.schema.createTable('users', function(table) {
    table.uuid('id').primary().defaultTo(knex.raw('gen_random_uuid()'));
    table.string('username', 50).notNullable().unique();
    table.string('email', 255).notNullable().unique();
    table.string('password_hash', 255).notNullable();
    table.string('markov_user_id', 100).unique();
    table.text('bio');
    table.string('location', 255);
    table.text('avatar_url');
    table.timestamps(true, true);

    table.index('username');
    table.index('email');
    table.index('markov_user_id');
  });
};

exports.down = function(knex) {
  return knex.schema.dropTableIfExists('users');
};
```

### Alembic (Python)

**Install**:
```bash
pip install alembic psycopg2-binary
```

**Initialize**:
```bash
alembic init migrations
```

**Configure** (`alembic.ini`):
```ini
sqlalchemy.url = postgresql://postgres:postgres@localhost/gotong_royong_dev
```

**Create Migration**:
```bash
alembic revision -m "create users table"
```

**Run Migrations**:
```bash
alembic upgrade head
```

**Rollback**:
```bash
alembic downgrade -1
```

**Example Migration** (`migrations/versions/001_create_users_table.py`):
```python
from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects.postgresql import UUID

def upgrade():
    op.create_table(
        'users',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, server_default=sa.text('gen_random_uuid()')),
        sa.Column('username', sa.String(50), nullable=False, unique=True),
        sa.Column('email', sa.String(255), nullable=False, unique=True),
        sa.Column('password_hash', sa.String(255), nullable=False),
        sa.Column('markov_user_id', sa.String(100), unique=True),
        sa.Column('bio', sa.Text()),
        sa.Column('location', sa.String(255)),
        sa.Column('avatar_url', sa.Text()),
        sa.Column('created_at', sa.TIMESTAMP, server_default=sa.func.now()),
        sa.Column('updated_at', sa.TIMESTAMP, server_default=sa.func.now(), onupdate=sa.func.now()),
    )

    op.create_index('idx_users_username', 'users', ['username'])
    op.create_index('idx_users_email', 'users', ['email'])
    op.create_index('idx_users_markov_id', 'users', ['markov_user_id'])

def downgrade():
    op.drop_table('users')
```

### Diesel (Rust)

**Install**:
```bash
cargo install diesel_cli --no-default-features --features postgres
```

**Setup**:
```bash
diesel setup
```

**Create Migration**:
```bash
diesel migration generate create_users_table
```

**Run Migrations**:
```bash
diesel migration run
```

**Rollback**:
```bash
diesel migration revert
```

**Example Migration** (`migrations/2026-02-10-120000_create_users_table/up.sql`):
```sql
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  username VARCHAR(50) NOT NULL UNIQUE,
  email VARCHAR(255) NOT NULL UNIQUE,
  password_hash VARCHAR(255) NOT NULL,
  markov_user_id VARCHAR(100) UNIQUE,
  bio TEXT,
  location VARCHAR(255),
  avatar_url TEXT,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_markov_id ON users(markov_user_id);
```

**Down Migration** (`migrations/2026-02-10-120000_create_users_table/down.sql`):
```sql
DROP TABLE IF EXISTS users CASCADE;
```

## Production Migration Strategy

### Pre-Deployment Checklist

- [ ] Migration tested in development
- [ ] Migration tested in staging
- [ ] Backup created before migration
- [ ] Rollback plan documented
- [ ] Downtime window scheduled (if needed)
- [ ] Team notified
- [ ] Monitoring alerts configured

### Zero-Downtime Migrations

For large tables, use these strategies:

#### 1. Add Column with Default

```sql
-- Step 1: Add nullable column
ALTER TABLE contributions ADD COLUMN verification_status VARCHAR(20);

-- Step 2: Populate in batches (background job)
UPDATE contributions
SET verification_status = 'pending'
WHERE verification_status IS NULL
LIMIT 1000;

-- Step 3: Add NOT NULL constraint (after all rows populated)
ALTER TABLE contributions
ALTER COLUMN verification_status SET NOT NULL;

-- Step 4: Add default
ALTER TABLE contributions
ALTER COLUMN verification_status SET DEFAULT 'pending';
```

#### 2. Create Index Concurrently (PostgreSQL)

```sql
-- Non-blocking index creation
CREATE INDEX CONCURRENTLY idx_contributions_user_status
ON contributions(user_id, verification_status);
```

#### 3. Rename Column Safely

```sql
-- Step 1: Add new column
ALTER TABLE users ADD COLUMN markov_user_id VARCHAR(100);

-- Step 2: Copy data
UPDATE users SET markov_user_id = old_markov_id;

-- Step 3: Deploy code that reads from both columns
-- (Application handles both column names)

-- Step 4: Drop old column (after traffic fully migrated)
ALTER TABLE users DROP COLUMN old_markov_id;
```

### Rollback Strategy

**Automatic Rollback**:
```bash
# If migration fails, automatically rollback
migrate_up || migrate_down
```

**Manual Rollback**:
```bash
# Identify last successful migration
SELECT * FROM knex_migrations ORDER BY id DESC LIMIT 1;

# Rollback one step
knex migrate:rollback

# Or rollback to specific version
knex migrate:down 20260210120000_create_users_table.js
```

## Migration Tracking

### Metadata Table

Migration tools create a metadata table to track applied migrations:

**Knex.js**: `knex_migrations`
```sql
CREATE TABLE knex_migrations (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255),
  batch INT,
  migration_time TIMESTAMP DEFAULT NOW()
);
```

**Alembic**: `alembic_version`
```sql
CREATE TABLE alembic_version (
  version_num VARCHAR(32) PRIMARY KEY
);
```

**Diesel**: `__diesel_schema_migrations`
```sql
CREATE TABLE __diesel_schema_migrations (
  version VARCHAR(50) PRIMARY KEY,
  run_on TIMESTAMP DEFAULT NOW()
);
```

## Testing Migrations

### Unit Tests

Test migrations with Docker:

```bash
# Start test database
docker run -d --name test-db -p 5433:5432 \
  -e POSTGRES_PASSWORD=test \
  postgres:14

# Run migrations
DATABASE_URL=postgresql://postgres:test@localhost:5433/test_db \
  knex migrate:latest

# Test application
npm test

# Cleanup
docker stop test-db && docker rm test-db
```

### Integration Tests

```javascript
describe('Migrations', () => {
  beforeAll(async () => {
    await knex.migrate.latest();
  });

  afterAll(async () => {
    await knex.migrate.rollback();
    await knex.destroy();
  });

  it('creates users table', async () => {
    const exists = await knex.schema.hasTable('users');
    expect(exists).toBe(true);
  });

  it('creates expected columns', async () => {
    const columns = await knex('users').columnInfo();
    expect(columns.username).toBeDefined();
    expect(columns.email).toBeDefined();
    expect(columns.markov_user_id).toBeDefined();
  });

  it('creates indexes', async () => {
    const indexes = await knex.raw(`
      SELECT indexname FROM pg_indexes
      WHERE tablename = 'users'
    `);
    const indexNames = indexes.rows.map(r => r.indexname);
    expect(indexNames).toContain('idx_users_username');
    expect(indexNames).toContain('idx_users_email');
  });
});
```

## Common Migration Patterns

### Pattern 1: Add Enum Column

```sql
-- Create enum type
CREATE TYPE contribution_type AS ENUM ('task_completion', 'code_review', 'documentation');

-- Add column
ALTER TABLE contributions
ADD COLUMN contribution_type contribution_type DEFAULT 'task_completion';
```

### Pattern 2: Split Table

```sql
-- Original: users table with profile fields
-- Goal: Split into users and profiles tables

-- Step 1: Create new profiles table
CREATE TABLE profiles (
  user_id UUID PRIMARY KEY REFERENCES users(id),
  bio TEXT,
  location VARCHAR(255),
  avatar_url TEXT
);

-- Step 2: Migrate data
INSERT INTO profiles (user_id, bio, location, avatar_url)
SELECT id, bio, location, avatar_url FROM users;

-- Step 3: Drop columns from users (after app updated)
ALTER TABLE users DROP COLUMN bio;
ALTER TABLE users DROP COLUMN location;
ALTER TABLE users DROP COLUMN avatar_url;
```

### Pattern 3: Change Column Type

```sql
-- Change user_id from VARCHAR to UUID

-- Step 1: Add new column
ALTER TABLE contributions ADD COLUMN user_uuid UUID;

-- Step 2: Populate new column
UPDATE contributions
SET user_uuid = user_id::UUID;

-- Step 3: Drop old column, rename new
ALTER TABLE contributions DROP COLUMN user_id;
ALTER TABLE contributions RENAME COLUMN user_uuid TO user_id;

-- Step 4: Re-create indexes
CREATE INDEX idx_contributions_user ON contributions(user_id);
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/migrations.yml
name: Test Migrations

on: [push, pull_request]

jobs:
  test-migrations:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v2

      - name: Setup Node.js
        uses: actions/setup-node@v2
        with:
          node-version: '18'

      - name: Install dependencies
        run: npm install

      - name: Run migrations
        env:
          DATABASE_URL: postgresql://postgres:test@localhost:5432/test_db
        run: npx knex migrate:latest

      - name: Run tests
        run: npm test
```

## Monitoring

### Migration Metrics

Track these metrics:
- Migration execution time
- Migration success/failure rate
- Rollback frequency
- Database downtime during migrations

### Alerts

**Critical**:
- Migration failed in production
- Rollback executed

**Warning**:
- Migration took >5 minutes
- Migration executed outside maintenance window

## References

- [Schema Requirements](schema-requirements.md) - Complete database schema
- [Infrastructure](../deployment/infrastructure.md) - Deployment architecture
- [Monitoring](../deployment/monitoring.md) - Observability setup
