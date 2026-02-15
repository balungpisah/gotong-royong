# Testing Integration

## Stack Lock Notice

Canonical implementation stack is Rust 2024 + Axum + SurrealDB `v3.0.0-beta.4`.

Use these as source of truth for active stack/runtime decisions:
- `setup-guide.md`
- `../backend-research.md`
- `../research/adr/ADR-001-rust-axum-surrealdb-stack-lock.md`

This document includes pre-lock multi-language testing examples. They are useful as patterns, but implementation planning should prioritize Rust/SurrealDB-aligned tests.

## Overview

This document describes the testing strategy for the Gotong Royong platform, including unit tests, integration tests, E2E tests, and testing the Markov Engine integration.

## Test Pyramid

```
         /\
        /  \  E2E Tests (5%)
       /____\
      /      \
     / Integr-\ Integration Tests (25%)
    /  ation  \
   /___________\
  /             \
 /   Unit Tests  \ Unit Tests (70%)
/_________________\
```

**Test Distribution**:
- **Unit Tests**: 70% - Fast, isolated, test individual functions
- **Integration Tests**: 25% - Test component interactions
- **E2E Tests**: 5% - Test complete user flows

## Testing Stack

### Recommended Tools

| Stack | Unit | Integration | E2E | Mocking |
|-------|------|-------------|-----|---------|
| **Node.js** | Jest | Jest + Supertest | Playwright/Cypress | nock |
| **Python** | pytest | pytest | Playwright | responses |
| **Rust** | cargo test | cargo test | - | mockito |

## Unit Tests

### Writing Unit Tests

**Purpose**: Test individual functions in isolation

**Example (Node.js with Jest)**:

```javascript
// src/utils/validation.js
function isValidEmail(email) {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

function isValidPassword(password) {
  return password.length >= 12;
}

module.exports = { isValidEmail, isValidPassword };
```

**Test File**:
```javascript
// src/utils/validation.test.js
const { isValidEmail, isValidPassword } = require('./validation');

describe('Email validation', () => {
  it('accepts valid email', () => {
    expect(isValidEmail('user@example.com')).toBe(true);
  });

  it('rejects email without @', () => {
    expect(isValidEmail('userexample.com')).toBe(false);
  });

  it('rejects email without domain', () => {
    expect(isValidEmail('user@')).toBe(false);
  });
});

describe('Password validation', () => {
  it('accepts password with 12 characters', () => {
    expect(isValidPassword('SecurePass123!')).toBe(true);
  });

  it('rejects password with 11 characters', () => {
    expect(isValidPassword('ShortPass1!')).toBe(false);
  });
});
```

**Run Tests**:
```bash
npm test
```

### Test Coverage

**Target**: 80% code coverage

**Generate Coverage Report**:
```bash
npm run test:coverage
```

**Coverage Configuration** (jest.config.js):
```javascript
module.exports = {
  collectCoverageFrom: [
    'src/**/*.js',
    '!src/**/*.test.js',
    '!src/index.js',
  ],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80,
    },
  },
};
```

## Integration Tests

### Database Integration Tests

**Purpose**: Test database interactions

**Setup**:
```javascript
// tests/setup.js
const { Pool } = require('pg');

let db;

beforeAll(async () => {
  db = new Pool({
    connectionString: process.env.TEST_DATABASE_URL,
  });

  // Run migrations
  await runMigrations(db);
});

afterAll(async () => {
  await db.end();
});

beforeEach(async () => {
  // Clean database before each test
  await db.query('TRUNCATE users, contributions, evidence CASCADE');
});

module.exports = { db };
```

**Test File**:
```javascript
// tests/repositories/user.test.js
const { db } = require('../setup');
const UserRepository = require('../../src/repositories/user');

describe('UserRepository', () => {
  let userRepo;

  beforeEach(() => {
    userRepo = new UserRepository(db);
  });

  it('creates user', async () => {
    const user = await userRepo.create({
      username: 'alice',
      email: 'alice@example.com',
      password_hash: 'hashed_password',
    });

    expect(user.id).toBeDefined();
    expect(user.username).toBe('alice');
  });

  it('finds user by email', async () => {
    await userRepo.create({
      username: 'alice',
      email: 'alice@example.com',
      password_hash: 'hashed_password',
    });

    const user = await userRepo.findByEmail('alice@example.com');
    expect(user).toBeDefined();
    expect(user.username).toBe('alice');
  });

  it('returns null for non-existent user', async () => {
    const user = await userRepo.findByEmail('nonexistent@example.com');
    expect(user).toBeNull();
  });
});
```

### API Integration Tests

**Purpose**: Test API endpoints

**Example (Node.js with Supertest)**:

```javascript
// tests/api/auth.test.js
const request = require('supertest');
const app = require('../../src/app');
const { db } = require('../setup');

describe('POST /api/auth/register', () => {
  it('registers new user', async () => {
    const response = await request(app)
      .post('/api/auth/register')
      .send({
        username: 'alice',
        email: 'alice@example.com',
        password: 'SecurePassword123!',
      })
      .expect(201);

    expect(response.body).toHaveProperty('user_id');
    expect(response.body).toHaveProperty('access_token');
  });

  it('rejects duplicate email', async () => {
    await request(app)
      .post('/api/auth/register')
      .send({
        username: 'alice',
        email: 'alice@example.com',
        password: 'SecurePassword123!',
      });

    const response = await request(app)
      .post('/api/auth/register')
      .send({
        username: 'bob',
        email: 'alice@example.com',  // Duplicate
        password: 'SecurePassword123!',
      })
      .expect(400);

    expect(response.body.error).toContain('already exists');
  });

  it('rejects weak password', async () => {
    const response = await request(app)
      .post('/api/auth/register')
      .send({
        username: 'alice',
        email: 'alice@example.com',
        password: 'weak',  // Too short
      })
      .expect(400);

    expect(response.body.error).toContain('password');
  });
});
```

## Testing Markov Engine Integration

### Mock Markov Server

**Purpose**: Test webhook integration without running Markov Engine

**Setup**:
```javascript
// tests/mocks/markov-server.js
const nock = require('nock');

function mockMarkovWebhook(status = 200, response = { processed: 1 }) {
  return nock('http://localhost:3001')
    .post('/v1/platforms/gotong_royong/webhook')
    .reply(status, response);
}

function mockMarkovReputationQuery(userId, reputation = 2550) {
  return nock('http://localhost:3001')
    .get(`/v1/users/gotong_royong:${userId}/reputation`)
    .reply(200, {
      user_id: `gotong_royong:${userId}`,
      reputation_score: reputation,
      tier: 'advanced',
    });
}

module.exports = { mockMarkovWebhook, mockMarkovReputationQuery };
```

**Test File**:
```javascript
// tests/integration/webhook.test.js
const { mockMarkovWebhook } = require('../mocks/markov-server');
const { publishWebhook } = require('../../src/services/webhook');

describe('Webhook Publishing', () => {
  it('sends contribution_created event', async () => {
    const mock = mockMarkovWebhook(200, { processed: 1 });

    await publishWebhook({
      event_type: 'contribution_created',
      actor: { user_id: 'user123', username: 'alice' },
      subject: {
        contribution_type: 'task_completion',
        title: 'Test Task',
      },
    });

    expect(mock.isDone()).toBe(true);
  });

  it('retries on 500 error', async () => {
    const mock1 = mockMarkovWebhook(500);
    const mock2 = mockMarkovWebhook(200, { processed: 1 });

    await publishWebhook({
      event_type: 'contribution_created',
      actor: { user_id: 'user123', username: 'alice' },
      subject: {
        contribution_type: 'task_completion',
        title: 'Test Task',
      },
    });

    expect(mock1.isDone()).toBe(true);
    expect(mock2.isDone()).toBe(true);
  });

  it('fails after max retries', async () => {
    const mock1 = mockMarkovWebhook(500);
    const mock2 = mockMarkovWebhook(500);
    const mock3 = mockMarkovWebhook(500);

    await expect(
      publishWebhook({
        event_type: 'contribution_created',
        actor: { user_id: 'user123', username: 'alice' },
        subject: {
          contribution_type: 'task_completion',
          title: 'Test Task',
        },
      })
    ).rejects.toThrow();

    expect(mock1.isDone()).toBe(true);
    expect(mock2.isDone()).toBe(true);
    expect(mock3.isDone()).toBe(true);
  });
});
```

### Testing with Real Markov Engine

**Purpose**: End-to-end integration testing

**Setup**:
1. Start Markov Engine locally
2. Configure webhook secret
3. Run integration tests

**Docker Compose for Testing**:
```yaml
# docker-compose.test.yml
version: '3.8'

services:
  gotong-royong-api:
    build: .
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@test-db:5432/test_db
      - MARKOV_API_URL=http://markov-engine:3001
    depends_on:
      - test-db
      - markov-engine

  test-db:
    image: postgres:14-alpine
    environment:
      - POSTGRES_DB=test_db
      - POSTGRES_PASSWORD=postgres

  markov-engine:
    image: markov-engine:latest
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@markov-db:5432/markov_db
      - GOTONG_ROYONG_WEBHOOK_SECRET=test_secret_32_chars_minimum_here

  markov-db:
    image: postgres:14-alpine
    environment:
      - POSTGRES_DB=markov_db
      - POSTGRES_PASSWORD=postgres
```

**Run E2E Tests**:
```bash
docker-compose -f docker-compose.test.yml up -d
npm run test:e2e
docker-compose -f docker-compose.test.yml down
```

## E2E Tests

### Testing Complete User Flows

**Purpose**: Test complete user journeys

**Example (Playwright)**:

```javascript
// tests/e2e/contribution-flow.spec.js
const { test, expect } = require('@playwright/test');

test.describe('Contribution Flow', () => {
  test('user completes task with evidence', async ({ page }) => {
    // 1. Login
    await page.goto('http://localhost:3000/login');
    await page.fill('input[name="email"]', 'alice@example.com');
    await page.fill('input[name="password"]', 'SecurePassword123!');
    await page.click('button[type="submit"]');

    // 2. Navigate to task
    await page.goto('http://localhost:3000/tasks/task_123');

    // 3. Mark as complete
    await page.click('button:has-text("Mark Complete")');

    // 4. Upload evidence
    const fileInput = await page.locator('input[type="file"]');
    await fileInput.setInputFiles('tests/fixtures/evidence.jpg');

    // 5. Submit
    await page.click('button:has-text("Submit Evidence")');

    // 6. Verify success message
    await expect(page.locator('.success-message')).toContainText('Evidence submitted');

    // 7. Verify contribution appears
    await page.goto('http://localhost:3000/profile');
    await expect(page.locator('.contributions-list')).toContainText('task_123');
  });
});
```

**Run E2E Tests**:
```bash
npx playwright test
```

## Test Data Fixtures

### Creating Test Fixtures

**Purpose**: Reusable test data

**Example**:
```javascript
// tests/fixtures/users.js
module.exports = {
  validUser: {
    username: 'alice',
    email: 'alice@example.com',
    password: 'SecurePassword123!',
  },

  invalidUsers: [
    {
      username: 'a',  // Too short
      email: 'alice@example.com',
      password: 'SecurePassword123!',
    },
    {
      username: 'alice',
      email: 'invalid-email',  // Invalid format
      password: 'SecurePassword123!',
    },
    {
      username: 'alice',
      email: 'alice@example.com',
      password: 'weak',  // Too weak
    },
  ],
};
```

**Load Test Payloads from Markov Fixtures**:
```javascript
// tests/fixtures/gotong-royong-payloads.js
const fs = require('fs');
const path = require('path');

const fixturesPath = path.join(
  __dirname,
  '../../../../tandang/markov-engine/tests/fixtures/gotong_royong_payloads.json'
);

const payloads = JSON.parse(fs.readFileSync(fixturesPath, 'utf-8'));

module.exports = payloads;
```

**Usage**:
```javascript
const { valid_contribution } = require('../fixtures/gotong-royong-payloads');

it('processes valid contribution', async () => {
  const response = await request(app)
    .post('/api/webhook')
    .send(valid_contribution)
    .expect(200);

  expect(response.body.processed).toBe(1);
});
```

## Continuous Integration

### GitHub Actions

**.github/workflows/test.yml**:
```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: test_db
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
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
        run: npm ci

      - name: Run linter
        run: npm run lint

      - name: Run tests
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/test_db
          REDIS_URL: redis://localhost:6379
        run: npm test -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v2
        with:
          files: ./coverage/lcov.info
```

## Testing Best Practices

### DO

- ✅ Write tests before fixing bugs (TDD)
- ✅ Test edge cases and error conditions
- ✅ Use descriptive test names
- ✅ Keep tests independent (no shared state)
- ✅ Use fixtures for test data
- ✅ Mock external services (Markov, S3)
- ✅ Run tests in CI/CD pipeline
- ✅ Maintain 80%+ code coverage

### DON'T

- ❌ Test implementation details
- ❌ Write flaky tests (timing-dependent)
- ❌ Skip tests in CI
- ❌ Test third-party libraries
- ❌ Use production database for tests
- ❌ Commit test secrets
- ❌ Write tests that depend on each other

## Performance Testing

### Load Testing (k6)

**Purpose**: Test API under load

**Setup**:
```bash
brew install k6
```

**Test Script** (k6-load-test.js):
```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '30s', target: 20 },   // Ramp-up to 20 users
    { duration: '1m', target: 20 },    // Stay at 20 users
    { duration: '30s', target: 0 },    // Ramp-down to 0 users
  ],
};

export default function () {
  let response = http.get('http://localhost:3000/api/tasks');

  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 200ms': (r) => r.timings.duration < 200,
  });

  sleep(1);
}
```

**Run Load Test**:
```bash
k6 run k6-load-test.js
```

## Security Testing

### Dependency Scanning

```bash
# Node.js
npm audit
npm audit fix

# Python
pip-audit
safety check

# Rust
cargo audit
```

### Static Analysis

```bash
# Node.js
npm run lint

# Python
bandit -r .
pylint .

# Rust
cargo clippy
```

## Test Documentation

### Writing Test Documentation

**Test README** (tests/README.md):
```markdown
# Testing Guide

## Running Tests

Unit tests: `npm test`
Integration tests: `npm run test:integration`
E2E tests: `npm run test:e2e`
All tests: `npm run test:all`

## Test Coverage

View coverage: `npm run test:coverage`
Target: 80% coverage

## CI/CD

Tests run automatically on:
- Every push to main
- Every pull request
- Nightly builds
```

## References

- [Setup Guide](setup-guide.md) - Local development setup
- [Local Development](local-development.md) - Development workflow
- [Markov Integration Guide](../../../tandang/markov-engine/docs/GOTONG-ROYONG-INTEGRATION-GUIDE.md) - Integration details
- [API Specifications](../api/webhook-spec.md) - API endpoints
