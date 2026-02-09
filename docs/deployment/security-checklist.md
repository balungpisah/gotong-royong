# Security Checklist

## Overview

This checklist covers security hardening requirements for the Gotong Royong platform. All items must be implemented before production deployment.

## Pre-Deployment Security Checklist

### Authentication & Authorization

- [ ] **JWT Implementation**
  - [ ] Use strong secret key (min 32 characters)
  - [ ] Set short token expiration (15 minutes for access, 7 days for refresh)
  - [ ] Implement token rotation
  - [ ] Store refresh tokens securely (hashed in database)
  - [ ] Invalidate tokens on logout

- [ ] **Password Security**
  - [ ] Enforce minimum password length (12 characters)
  - [ ] Require password complexity (uppercase, lowercase, number, symbol)
  - [ ] Hash passwords with bcrypt (cost factor: 12) or Argon2id
  - [ ] Implement rate limiting on login attempts (5 attempts per 15 minutes)
  - [ ] Implement account lockout after failed attempts
  - [ ] Support password reset with secure tokens (time-limited, one-time use)

- [ ] **RBAC Implementation**
  - [ ] Define user roles (contributor, verifier, admin)
  - [ ] Implement permission checks on all endpoints
  - [ ] Use role-based access control middleware
  - [ ] Audit role assignments

**Example (Node.js)**:
```javascript
const bcrypt = require('bcrypt');
const SALT_ROUNDS = 12;

async function hashPassword(password) {
  return bcrypt.hash(password, SALT_ROUNDS);
}

async function verifyPassword(password, hash) {
  return bcrypt.compare(password, hash);
}
```

### API Security

- [ ] **HTTPS/TLS**
  - [ ] Enforce HTTPS in production (no HTTP)
  - [ ] Use TLS 1.3 (disable TLS 1.0, 1.1)
  - [ ] Configure strong cipher suites
  - [ ] Use valid SSL certificates (Let's Encrypt)
  - [ ] Implement HSTS headers (`Strict-Transport-Security: max-age=31536000`)

- [ ] **Rate Limiting**
  - [ ] Global rate limit: 100 req/min per IP
  - [ ] Login endpoint: 5 req/15min per IP
  - [ ] Webhook endpoint: 1000 req/min
  - [ ] Evidence upload: 10 uploads/hour per user
  - [ ] Return `429 Too Many Requests` with `Retry-After` header

**Example (Express)**:
```javascript
const rateLimit = require('express-rate-limit');

const loginLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 5,
  message: 'Too many login attempts, please try again later',
});

app.post('/api/auth/login', loginLimiter, loginHandler);
```

- [ ] **Input Validation**
  - [ ] Validate all user input (use schema validation: Joi, Zod, yup)
  - [ ] Sanitize HTML input (prevent XSS)
  - [ ] Validate file uploads (type, size, content)
  - [ ] Use parameterized queries (prevent SQL injection)
  - [ ] Validate webhook signatures

- [ ] **CORS Configuration**
  - [ ] Whitelist allowed origins (no `*` in production)
  - [ ] Configure allowed methods (GET, POST, PUT, DELETE)
  - [ ] Configure allowed headers
  - [ ] Set `Access-Control-Allow-Credentials: true` only if needed

**Example**:
```javascript
const cors = require('cors');

app.use(cors({
  origin: ['https://app.gotong-royong.app'],
  methods: ['GET', 'POST', 'PUT', 'DELETE'],
  allowedHeaders: ['Content-Type', 'Authorization'],
  credentials: true,
}));
```

- [ ] **Security Headers**
  - [ ] `Content-Security-Policy`: Restrict resource loading
  - [ ] `X-Content-Type-Options: nosniff`
  - [ ] `X-Frame-Options: DENY`
  - [ ] `X-XSS-Protection: 1; mode=block`
  - [ ] `Referrer-Policy: strict-origin-when-cross-origin`

**Example (Helmet.js)**:
```javascript
const helmet = require('helmet');

app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
    },
  },
}));
```

### Database Security

- [ ] **Access Control**
  - [ ] Use separate database user for application (not superuser)
  - [ ] Grant minimum required privileges (SELECT, INSERT, UPDATE on specific tables)
  - [ ] Disable remote root access
  - [ ] Use connection pooling with max connections limit

- [ ] **Encryption**
  - [ ] Enable encryption at rest (AWS RDS, Azure SQL)
  - [ ] Use TLS for database connections
  - [ ] Encrypt sensitive columns (PII data)
  - [ ] Never store passwords in plain text

- [ ] **SQL Injection Prevention**
  - [ ] Use parameterized queries (prepared statements)
  - [ ] Never concatenate user input into SQL
  - [ ] Validate input types
  - [ ] Use ORM with proper escaping

**Example (PostgreSQL with parameterized queries)**:
```javascript
// ✅ GOOD: Parameterized query
const result = await db.query(
  'SELECT * FROM users WHERE email = $1',
  [email]
);

// ❌ BAD: String concatenation (SQL injection risk)
const result = await db.query(
  `SELECT * FROM users WHERE email = '${email}'`
);
```

- [ ] **Backup Security**
  - [ ] Encrypt backups
  - [ ] Store backups in separate account/region
  - [ ] Test restore procedures regularly
  - [ ] Implement backup retention policy

### Webhook Security

- [ ] **Signature Verification**
  - [ ] Verify HMAC-SHA256 signatures on all webhooks
  - [ ] Use constant-time comparison (prevent timing attacks)
  - [ ] Reject webhooks with invalid signatures
  - [ ] Log signature verification failures

- [ ] **Secret Management**
  - [ ] Generate strong webhook secrets (32+ characters)
  - [ ] Store secrets in environment variables or secrets manager
  - [ ] Rotate secrets every 90 days
  - [ ] Support dual secrets during rotation

- [ ] **Replay Attack Prevention**
  - [ ] Validate event timestamps (reject if >5 minutes old)
  - [ ] Use event IDs for idempotency
  - [ ] Store processed event IDs to detect duplicates

### File Upload Security

- [ ] **Validation**
  - [ ] Validate file type (check magic bytes, not just extension)
  - [ ] Enforce file size limits (max 10MB for photos)
  - [ ] Scan for malware (ClamAV)
  - [ ] Validate image dimensions
  - [ ] Strip EXIF metadata (except required fields)

**Example (file type validation)**:
```javascript
const fileType = require('file-type');

async function validateFileType(buffer) {
  const type = await fileType.fromBuffer(buffer);

  const allowedTypes = ['image/jpeg', 'image/png', 'image/heic'];
  if (!type || !allowedTypes.includes(type.mime)) {
    throw new Error('Invalid file type');
  }

  return type;
}
```

- [ ] **Storage Security**
  - [ ] Store files in private S3 bucket
  - [ ] Use presigned URLs for uploads (time-limited)
  - [ ] Use presigned URLs for downloads (time-limited)
  - [ ] Enable S3 versioning
  - [ ] Enable S3 access logging

- [ ] **Access Control**
  - [ ] Verify user owns the contribution before allowing upload
  - [ ] Verify user has permission before allowing download
  - [ ] Log all file access for audit

### Secrets Management

- [ ] **Environment Variables**
  - [ ] Never commit secrets to version control
  - [ ] Use `.env.local` for development (in `.gitignore`)
  - [ ] Use secrets manager in production (AWS Secrets Manager, Vault)
  - [ ] Rotate secrets regularly
  - [ ] Use different secrets for each environment

- [ ] **Secrets Checklist**
  - [ ] Database connection string
  - [ ] Redis URL
  - [ ] JWT secret
  - [ ] Webhook secret
  - [ ] AWS access keys
  - [ ] Encryption keys
  - [ ] Third-party API keys

**Example (.gitignore)**:
```
.env
.env.local
.env.production
secrets/
```

### Logging & Monitoring

- [ ] **Security Logging**
  - [ ] Log all authentication attempts (success and failure)
  - [ ] Log authorization failures
  - [ ] Log webhook signature failures
  - [ ] Log suspicious activity (repeated failed logins, etc.)
  - [ ] Never log passwords, tokens, or secrets

**Example (structured logging)**:
```javascript
const winston = require('winston');

const logger = winston.createLogger({
  format: winston.format.json(),
  transports: [
    new winston.transports.File({ filename: 'security.log' }),
  ],
});

// Log failed login
logger.warn('Login failed', {
  event: 'auth.login.failed',
  ip: req.ip,
  email: email,
  reason: 'invalid_password',
  timestamp: new Date().toISOString(),
});
```

- [ ] **Monitoring & Alerting**
  - [ ] Monitor failed login attempts (alert if >10/min)
  - [ ] Monitor webhook signature failures (alert if >10/min)
  - [ ] Monitor database connection failures
  - [ ] Monitor disk space usage
  - [ ] Monitor API error rates

### Data Privacy (GDPR Compliance)

- [ ] **User Data Protection**
  - [ ] Implement user data export (GDPR Article 20)
  - [ ] Implement user data deletion (GDPR Article 17)
  - [ ] Obtain consent for data processing
  - [ ] Provide privacy policy
  - [ ] Implement data minimization (only collect necessary data)

- [ ] **PII Handling**
  - [ ] Identify PII fields (email, name, location, phone)
  - [ ] Encrypt PII in database
  - [ ] Anonymize PII in logs
  - [ ] Implement data retention policies
  - [ ] Provide data breach notification plan

### Infrastructure Security

- [ ] **Network Security**
  - [ ] Use VPC with private subnets for databases
  - [ ] Configure security groups (whitelist IPs)
  - [ ] Disable unused ports
  - [ ] Use bastion host for SSH access
  - [ ] Enable VPC flow logs

- [ ] **Container Security**
  - [ ] Use official base images
  - [ ] Scan images for vulnerabilities (Trivy, Snyk)
  - [ ] Run containers as non-root user
  - [ ] Use read-only root filesystem
  - [ ] Limit container resources (CPU, memory)

**Example (Dockerfile)**:
```dockerfile
FROM node:18-alpine

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001

# Set working directory
WORKDIR /app

# Copy files
COPY --chown=nodejs:nodejs . .

# Install dependencies
RUN npm ci --only=production

# Switch to non-root user
USER nodejs

# Run application
CMD ["node", "server.js"]
```

- [ ] **Kubernetes Security**
  - [ ] Use network policies to restrict pod communication
  - [ ] Use pod security policies
  - [ ] Enable RBAC
  - [ ] Scan Kubernetes configs (kube-bench)
  - [ ] Use secrets for sensitive data (not ConfigMaps)

### Dependency Security

- [ ] **Dependency Management**
  - [ ] Use lock files (package-lock.json, Cargo.lock)
  - [ ] Run `npm audit` or `cargo audit` regularly
  - [ ] Update dependencies monthly
  - [ ] Use automated dependency updates (Dependabot, Renovate)
  - [ ] Review security advisories

**Example**:
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

- [ ] **Supply Chain Security**
  - [ ] Verify package integrity (checksums)
  - [ ] Use private npm registry for internal packages
  - [ ] Pin dependency versions
  - [ ] Review dependency licenses

### Incident Response

- [ ] **Incident Response Plan**
  - [ ] Document incident response procedures
  - [ ] Assign incident response team
  - [ ] Define severity levels
  - [ ] Establish communication channels
  - [ ] Conduct incident response drills

- [ ] **Breach Notification**
  - [ ] Document data breach notification procedures
  - [ ] Identify regulatory requirements (GDPR: 72 hours)
  - [ ] Prepare notification templates
  - [ ] Maintain contact list (users, regulators)

### Security Testing

- [ ] **Penetration Testing**
  - [ ] Conduct annual penetration tests
  - [ ] Test authentication/authorization
  - [ ] Test API endpoints
  - [ ] Test file upload functionality
  - [ ] Test webhook security

- [ ] **Vulnerability Scanning**
  - [ ] Scan web application (OWASP ZAP, Burp Suite)
  - [ ] Scan dependencies (npm audit, Snyk)
  - [ ] Scan Docker images (Trivy, Clair)
  - [ ] Scan infrastructure (AWS Inspector, Nessus)

- [ ] **Code Review**
  - [ ] Review all code changes before merge
  - [ ] Use static analysis tools (ESLint, Clippy)
  - [ ] Check for hardcoded secrets (git-secrets, TruffleHog)
  - [ ] Review security-critical code by security team

## Production Deployment Checklist

### Pre-Deployment

- [ ] All security checklist items completed
- [ ] Security scan passed
- [ ] Penetration test completed
- [ ] Code review completed
- [ ] Secrets rotated
- [ ] Backups configured
- [ ] Monitoring configured
- [ ] Incident response plan documented

### Post-Deployment

- [ ] Verify HTTPS works
- [ ] Verify authentication works
- [ ] Verify webhook signatures work
- [ ] Verify file uploads work
- [ ] Verify monitoring/alerting works
- [ ] Review security logs
- [ ] Document deployment

## Security Monitoring

### Metrics to Track

- Failed login attempts per minute
- Webhook signature failures per minute
- API error rate (4xx, 5xx)
- Database connection failures
- File upload failures
- Suspicious IP addresses

### Alerts

**Critical (PagerDuty)**:
- Failed logins > 100/min (brute force attack)
- Webhook signature failures > 50/min (potential attack)
- Database unavailable
- S3 bucket made public

**Warning (Slack)**:
- Failed logins > 20/min
- Webhook signature failures > 10/min
- API error rate > 5%
- Disk usage > 80%

## Compliance

### Standards

- [ ] **OWASP Top 10**: Address all vulnerabilities
- [ ] **PCI DSS**: If handling payment data
- [ ] **GDPR**: If serving EU users
- [ ] **SOC 2**: For enterprise customers
- [ ] **ISO 27001**: Information security management

### Audits

- [ ] Conduct annual security audits
- [ ] Review security logs monthly
- [ ] Update security policies annually
- [ ] Train team on security best practices

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [CIS Benchmarks](https://www.cisecurity.org/cis-benchmarks/)
- [Infrastructure](infrastructure.md) - Deployment architecture
- [Monitoring](monitoring.md) - Observability setup
