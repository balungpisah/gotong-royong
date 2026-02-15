# Local Development

## Stack Lock Notice

Canonical implementation stack is Rust 2024 + Axum + SurrealDB `v3.0.0-beta.4`.

Use these as source of truth for active setup/runtime decisions:
- `setup-guide.md`
- `../backend-research.md`
- `../research/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`

This document still contains pre-lock polyglot workflow examples and should be treated as historical guidance until fully rewritten.

## Overview

This guide covers the day-to-day development workflow for the Gotong Royong platform, including code organization, development practices, debugging, and common tasks.

## Development Workflow

### Daily Workflow

```
1. Pull latest changes
   ↓
2. Create feature branch
   ↓
3. Make changes
   ↓
4. Run tests
   ↓
5. Commit changes
   ↓
6. Push to remote
   ↓
7. Create pull request
   ↓
8. Code review
   ↓
9. Merge to main
```

### Starting Your Day

```bash
# 1. Pull latest changes
git checkout main
git pull origin main

# 2. Create feature branch
git checkout -b feature/add-evidence-validation

# 3. Start services
docker-compose up -d

# 4. Start development server
npm run dev
```

## Code Organization

### Project Structure

```
gotong-royong/
├── src/                    # Source code
│   ├── api/               # API routes
│   │   ├── auth.js
│   │   ├── tasks.js
│   │   ├── contributions.js
│   │   └── evidence.js
│   ├── services/          # Business logic
│   │   ├── user.js
│   │   ├── task.js
│   │   ├── contribution.js
│   │   └── webhook.js
│   ├── repositories/      # Database access
│   │   ├── user.js
│   │   ├── task.js
│   │   └── contribution.js
│   ├── middleware/        # Express middleware
│   │   ├── auth.js
│   │   ├── validation.js
│   │   └── error-handler.js
│   ├── utils/             # Utilities
│   │   ├── validation.js
│   │   ├── crypto.js
│   │   └── logger.js
│   ├── config/            # Configuration
│   │   └── database.js
│   └── index.js           # Entry point
├── tests/                 # Tests
│   ├── unit/
│   ├── integration/
│   └── e2e/
├── migrations/            # Database migrations
├── docs/                  # Documentation
├── scripts/               # Utility scripts
├── .env.example           # Example environment file
├── package.json
└── README.md
```

### Naming Conventions

**Files**:
- Use kebab-case: `user-service.js`, `auth-middleware.js`
- Test files: `user-service.test.js`

**Variables**:
- camelCase for variables: `userId`, `webhookSecret`
- UPPER_CASE for constants: `MAX_FILE_SIZE`, `JWT_EXPIRY`
- PascalCase for classes: `UserService`, `TaskRepository`

**Functions**:
- Use descriptive verbs: `createUser()`, `validateEmail()`, `sendWebhook()`
- Async functions: `async fetchUserById()`, `async processWebhook()`

**Database**:
- snake_case for tables: `users`, `task_assignments`, `evidence`
- snake_case for columns: `user_id`, `created_at`, `media_hash`

## Development Practices

### Code Style

**Use ESLint/Prettier**:
```bash
npm run lint
npm run format
```

**Configuration** (.eslintrc.js):
```javascript
module.exports = {
  extends: ['eslint:recommended', 'prettier'],
  env: {
    node: true,
    es2021: true,
  },
  parserOptions: {
    ecmaVersion: 2021,
  },
  rules: {
    'no-console': 'warn',
    'no-unused-vars': 'error',
    'prefer-const': 'error',
  },
};
```

### Git Commit Messages

**Format**:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Code style (formatting)
- `refactor`: Code refactoring
- `test`: Tests
- `chore`: Build/tooling

**Examples**:
```bash
feat(api): add evidence upload endpoint

Implements presigned URL generation for direct S3 uploads.
Includes validation for file type and size.

Closes #123
```

```bash
fix(webhook): handle signature verification edge case

Fixed issue where whitespace in JSON payload caused
signature mismatch.

Fixes #456
```

### Branch Strategy

**Main Branches**:
- `main`: Production-ready code
- `develop`: Integration branch (optional)

**Feature Branches**:
- Format: `feature/<description>`
- Example: `feature/add-gps-validation`

**Bug Fix Branches**:
- Format: `fix/<description>`
- Example: `fix/webhook-retry-logic`

**Hotfix Branches**:
- Format: `hotfix/<description>`
- Example: `hotfix/security-patch`

## Debugging

### VS Code Debugger

**Launch Configuration** (.vscode/launch.json):
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
      "envFile": "${workspaceFolder}/.env.local",
      "console": "integratedTerminal"
    },
    {
      "type": "node",
      "request": "launch",
      "name": "Debug Tests",
      "program": "${workspaceFolder}/node_modules/.bin/jest",
      "args": ["--runInBand"],
      "console": "integratedTerminal"
    }
  ]
}
```

**Set Breakpoints**:
1. Click left margin in editor
2. Start debugger (F5)
3. Trigger breakpoint
4. Step through code

### Console Debugging

**Use structured logging**:
```javascript
// ❌ Bad
console.log('User:', user);

// ✅ Good
logger.debug('User retrieved', {
  user_id: user.id,
  username: user.username,
});
```

**Debug specific module**:
```bash
DEBUG=services:webhook npm run dev
```

### Database Debugging

**Enable query logging**:
```javascript
// knexfile.js
module.exports = {
  development: {
    client: 'postgresql',
    connection: process.env.DATABASE_URL,
    debug: true,  // Log all queries
  },
};
```

**Inspect query**:
```javascript
const query = db('users').where({ email: 'alice@example.com' });
console.log(query.toString());  // Print SQL query
```

**Analyze slow queries**:
```sql
EXPLAIN ANALYZE SELECT * FROM contributions WHERE user_id = 'user123';
```

### HTTP Debugging

**Use REST Client (VS Code Extension)**:

**Create** `requests.http`:
```http
### Register User
POST http://localhost:3000/api/auth/register
Content-Type: application/json

{
  "username": "alice",
  "email": "alice@example.com",
  "password": "SecurePassword123!"
}

### Login
POST http://localhost:3000/api/auth/login
Content-Type: application/json

{
  "email": "alice@example.com",
  "password": "SecurePassword123!"
}

### Get Tasks (authenticated)
GET http://localhost:3000/api/tasks
Authorization: Bearer {{access_token}}
```

**Or use curl**:
```bash
# Register
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","password":"SecurePassword123!"}'

# Login
TOKEN=$(curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"SecurePassword123!"}' \
  | jq -r '.access_token')

# Get tasks
curl http://localhost:3000/api/tasks \
  -H "Authorization: Bearer $TOKEN"
```

## Common Tasks

### Adding a New API Endpoint

**1. Define Route**:
```javascript
// src/api/tasks.js
const express = require('express');
const router = express.Router();
const TaskService = require('../services/task');
const { authenticate } = require('../middleware/auth');

router.post('/', authenticate, async (req, res, next) => {
  try {
    const task = await TaskService.create(req.user.id, req.body);
    res.status(201).json(task);
  } catch (error) {
    next(error);
  }
});

module.exports = router;
```

**2. Implement Service**:
```javascript
// src/services/task.js
const TaskRepository = require('../repositories/task');

class TaskService {
  static async create(userId, data) {
    // Validate input
    if (!data.title) {
      throw new Error('Title is required');
    }

    // Create task
    return TaskRepository.create({
      creator_id: userId,
      title: data.title,
      description: data.description,
      status: 'open',
    });
  }
}

module.exports = TaskService;
```

**3. Implement Repository**:
```javascript
// src/repositories/task.js
const db = require('../config/database');

class TaskRepository {
  static async create(data) {
    const [task] = await db('tasks')
      .insert(data)
      .returning('*');
    return task;
  }
}

module.exports = TaskRepository;
```

**4. Write Tests**:
```javascript
// tests/api/tasks.test.js
const request = require('supertest');
const app = require('../../src/app');

describe('POST /api/tasks', () => {
  it('creates task', async () => {
    const response = await request(app)
      .post('/api/tasks')
      .set('Authorization', `Bearer ${token}`)
      .send({
        title: 'Test Task',
        description: 'Test Description',
      })
      .expect(201);

    expect(response.body).toHaveProperty('id');
    expect(response.body.title).toBe('Test Task');
  });
});
```

### Adding a Database Migration

**1. Create Migration**:
```bash
npx knex migrate:make add_location_to_tasks
```

**2. Edit Migration**:
```javascript
// migrations/20260210120000_add_location_to_tasks.js
exports.up = function(knex) {
  return knex.schema.table('tasks', function(table) {
    table.string('location', 255);
    table.decimal('latitude', 10, 8);
    table.decimal('longitude', 11, 8);
  });
};

exports.down = function(knex) {
  return knex.schema.table('tasks', function(table) {
    table.dropColumn('location');
    table.dropColumn('latitude');
    table.dropColumn('longitude');
  });
};
```

**3. Run Migration**:
```bash
npm run migrate
```

**4. Update Model**:
```javascript
// Add to TaskRepository
static async findNearby(lat, lon, radiusKm = 10) {
  // Calculate bounding box
  // Query tasks within box
}
```

### Adding Environment Variable

**1. Add to .env.example**:
```bash
# Feature flags
ENABLE_GPS_VERIFICATION=true
```

**2. Add to config**:
```javascript
// src/config/index.js
module.exports = {
  features: {
    gpsVerification: process.env.ENABLE_GPS_VERIFICATION === 'true',
  },
};
```

**3. Use in code**:
```javascript
const config = require('./config');

if (config.features.gpsVerification) {
  // Enable GPS verification
}
```

### Running Background Jobs Locally

**Using Bull Queue**:
```javascript
// src/queues/webhook.js
const Queue = require('bull');
const webhookQueue = new Queue('webhook-events', process.env.REDIS_URL);

webhookQueue.process(async (job) => {
  console.log('Processing webhook:', job.data.event_type);
  await deliverWebhook(job.data);
});

module.exports = webhookQueue;
```

**Start Worker**:
```bash
node src/workers/webhook-worker.js
```

**Monitor Queue**:
```bash
# Install Bull Board
npm install bull-board

# Access UI: http://localhost:3000/admin/queues
```

## Hot Reloading

### Node.js (nodemon)

**Install**:
```bash
npm install --save-dev nodemon
```

**Configuration** (nodemon.json):
```json
{
  "watch": ["src"],
  "ext": "js,json",
  "ignore": ["src/**/*.test.js"],
  "exec": "node src/index.js"
}
```

**Run**:
```bash
npm run dev
# or
npx nodemon src/index.js
```

### Python (uvicorn)

```bash
uvicorn app:app --reload
```

### Rust (cargo-watch)

```bash
cargo install cargo-watch
cargo watch -x run
```

## Performance Profiling

### Node.js Profiling

**CPU Profiling**:
```bash
node --prof src/index.js
# Generate load
node --prof-process isolate-*.log > profile.txt
```

**Memory Profiling**:
```bash
node --inspect src/index.js
# Open chrome://inspect
# Take heap snapshot
```

**Using clinic.js**:
```bash
npm install -g clinic
clinic doctor -- node src/index.js
```

## Database Management

### Common Operations

**View Tables**:
```sql
\dt
```

**Describe Table**:
```sql
\d users
```

**Count Rows**:
```sql
SELECT COUNT(*) FROM users;
```

**Recent Contributions**:
```sql
SELECT * FROM contributions
ORDER BY created_at DESC
LIMIT 10;
```

**Clear Cache**:
```bash
redis-cli FLUSHALL
```

## Troubleshooting Development Issues

### Issue: Port Already in Use

**Solution**:
```bash
# Find process
lsof -i :3000

# Kill process
kill -9 <PID>
```

### Issue: Database Connection Error

**Solutions**:
```bash
# Check PostgreSQL is running
docker-compose ps

# Restart database
docker-compose restart db

# Check connection
psql $DATABASE_URL -c "SELECT 1"
```

### Issue: Tests Failing Locally

**Solutions**:
```bash
# Clear test database
npm run db:reset

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Clear Jest cache
npx jest --clearCache
```

### Issue: Memory Leak

**Solutions**:
```bash
# Run with memory limit
node --max-old-space-size=4096 src/index.js

# Profile memory
clinic heapprofiler -- node src/index.js
```

## Code Review Checklist

**Before Creating PR**:
- [ ] Tests pass locally
- [ ] Code is formatted (Prettier)
- [ ] No linter errors (ESLint)
- [ ] Commits are well-formatted
- [ ] No console.log statements
- [ ] No commented-out code
- [ ] Environment variables documented
- [ ] README updated if needed

**Reviewing PR**:
- [ ] Code follows style guide
- [ ] Tests cover new functionality
- [ ] No security vulnerabilities
- [ ] Database migrations are reversible
- [ ] API changes are documented
- [ ] Error handling is appropriate

## Development Tips

### 1. Use Environment-Specific Configs

```javascript
const config = {
  development: {
    logLevel: 'debug',
    corsOrigins: ['http://localhost:3001'],
  },
  production: {
    logLevel: 'info',
    corsOrigins: ['https://app.gotong-royong.app'],
  },
};

module.exports = config[process.env.NODE_ENV || 'development'];
```

### 2. Use Feature Flags

```javascript
const features = {
  newEvidenceFlow: process.env.FEATURE_NEW_EVIDENCE === 'true',
};

if (features.newEvidenceFlow) {
  // Use new flow
} else {
  // Use old flow
}
```

### 3. Log Important Events

```javascript
logger.info('User registered', {
  user_id: user.id,
  username: user.username,
});

logger.error('Webhook delivery failed', {
  event_id: event.id,
  error: error.message,
  attempts: retryCount,
});
```

### 4. Handle Errors Gracefully

```javascript
app.use((error, req, res, next) => {
  logger.error('Unhandled error', {
    error: error.message,
    stack: error.stack,
    url: req.url,
  });

  res.status(error.status || 500).json({
    error: process.env.NODE_ENV === 'production'
      ? 'Internal server error'
      : error.message,
  });
});
```

### 5. Use Database Transactions

```javascript
async function createContributionWithEvidence(contributionData, evidenceData) {
  const trx = await db.transaction();

  try {
    const contribution = await trx('contributions').insert(contributionData).returning('*');
    await trx('evidence').insert({ ...evidenceData, contribution_id: contribution[0].id });
    await trx.commit();
    return contribution[0];
  } catch (error) {
    await trx.rollback();
    throw error;
  }
}
```

## Next Steps

After reading this guide:
1. Complete [Setup Guide](setup-guide.md) if not done
2. Review [Testing Integration](testing-integration.md) for testing practices
3. Review [API Documentation](../api/webhook-spec.md) for API details
4. Join daily standup meetings
5. Pick up a task from the backlog

## References

- [Setup Guide](setup-guide.md) - Initial setup
- [Testing Integration](testing-integration.md) - Testing practices
- [Architecture Overview](../architecture/system-overview.md) - System design
- [Database Schema](../database/schema-requirements.md) - Database design
- [API Specifications](../api/webhook-spec.md) - API documentation
