# Infrastructure

## Overview

This document outlines deployment architecture options for the Gotong Royong platform, from local development to production-scale deployments.

## Deployment Architectures

### Development Environment

**Stack**: Docker Compose

**Services**:
- API Server (Node.js/Python/Rust)
- PostgreSQL 14
- Redis 7
- MinIO (S3-compatible)

**docker-compose.yml**:
```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:postgres@db:5432/gotong_royong_dev
      - REDIS_URL=redis://redis:6379
      - S3_ENDPOINT=http://minio:9000
      - S3_ACCESS_KEY=minioadmin
      - S3_SECRET_KEY=minioadmin
      - GOTONG_ROYONG_WEBHOOK_SECRET=${WEBHOOK_SECRET}
    depends_on:
      - db
      - redis
      - minio

  db:
    image: postgres:14-alpine
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=gotong_royong_dev
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio_data:/data

volumes:
  postgres_data:
  redis_data:
  minio_data:
```

**Start**:
```bash
docker-compose up -d
```

**Access**:
- API: http://localhost:3000
- PostgreSQL: localhost:5432
- Redis: localhost:6379
- MinIO Console: http://localhost:9001

---

### Staging Environment

**Stack**: Docker Swarm or Kubernetes (single node)

**Architecture**:
```
┌─────────────────────────────────────────────────────┐
│ Load Balancer (Traefik/nginx)                      │
└────────────────┬────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
┌───────▼───────┐ ┌───────▼───────┐
│ API Server 1  │ │ API Server 2  │
└───────┬───────┘ └───────┬───────┘
        │                 │
        └────────┬────────┘
                 │
        ┌────────┴────────┐
        │                 │
┌───────▼───────┐ ┌───────▼───────┐
│ PostgreSQL    │ │ Redis         │
└───────────────┘ └───────────────┘
        │
┌───────▼───────┐
│ MinIO/S3      │
└───────────────┘
```

**Kubernetes Manifests**:

**deployment.yaml**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gotong-royong-api
  namespace: staging
spec:
  replicas: 2
  selector:
    matchLabels:
      app: gotong-royong-api
  template:
    metadata:
      labels:
        app: gotong-royong-api
    spec:
      containers:
      - name: api
        image: gotong-royong/api:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secrets
              key: url
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: S3_ENDPOINT
          value: "https://s3.staging.gotong-royong.app"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
```

**service.yaml**:
```yaml
apiVersion: v1
kind: Service
metadata:
  name: gotong-royong-api
  namespace: staging
spec:
  selector:
    app: gotong-royong-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

---

### Production Environment

**Stack**: Kubernetes (multi-node cluster)

**Architecture**:
```
                 ┌─────────────────┐
                 │   CloudFlare    │
                 │   (DDoS/CDN)    │
                 └────────┬────────┘
                          │
                 ┌────────▼────────┐
                 │ Load Balancer   │
                 │ (AWS ALB/nginx) │
                 └────────┬────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
┌───────▼───────┐ ┌───────▼───────┐ ┌───────▼───────┐
│ API Pod 1     │ │ API Pod 2     │ │ API Pod 3     │
│ (3 replicas)  │ │ (3 replicas)  │ │ (3 replicas)  │
└───────┬───────┘ └───────┬───────┘ └───────┬───────┘
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
┌───────▼───────┐ ┌───────▼───────┐ ┌───────▼───────┐
│ Worker Pod 1  │ │ Worker Pod 2  │ │ Redis Cluster │
│ (2 replicas)  │ │ (2 replicas)  │ │ (3 nodes)     │
└───────────────┘ └───────────────┘ └───────┬───────┘
                                            │
                          ┌─────────────────┼─────────────────┐
                          │                 │                 │
                  ┌───────▼───────┐ ┌───────▼───────┐ ┌───────▼───────┐
                  │ PostgreSQL    │ │ S3 Storage    │ │ Monitoring    │
                  │ (RDS/managed) │ │ (AWS S3)      │ │ (Prometheus)  │
                  └───────────────┘ └───────────────┘ └───────────────┘
```

**Components**:

1. **API Pods**: Stateless API servers (3 replicas)
2. **Worker Pods**: Background job processors (2 replicas)
3. **PostgreSQL**: RDS or managed PostgreSQL
4. **Redis**: ElastiCache or Redis cluster
5. **S3**: AWS S3 or Cloudflare R2
6. **Load Balancer**: AWS ALB or nginx Ingress Controller
7. **Monitoring**: Prometheus + Grafana

---

## Infrastructure as Code

### Terraform (AWS)

**main.tf**:
```hcl
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# VPC
resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"

  tags = {
    Name = "gotong-royong-vpc"
  }
}

# EKS Cluster
resource "aws_eks_cluster" "main" {
  name     = "gotong-royong-cluster"
  role_arn = aws_iam_role.eks_cluster.arn

  vpc_config {
    subnet_ids = aws_subnet.private[*].id
  }

  depends_on = [
    aws_iam_role_policy_attachment.eks_cluster_policy,
  ]
}

# RDS PostgreSQL
resource "aws_db_instance" "postgres" {
  identifier           = "gotong-royong-db"
  engine               = "postgres"
  engine_version       = "14.7"
  instance_class       = "db.t3.medium"
  allocated_storage    = 100
  storage_encrypted    = true
  db_name              = "gotong_royong"
  username             = var.db_username
  password             = var.db_password
  skip_final_snapshot  = false
  backup_retention_period = 7

  vpc_security_group_ids = [aws_security_group.db.id]
  db_subnet_group_name   = aws_db_subnet_group.main.name

  tags = {
    Name = "gotong-royong-db"
  }
}

# ElastiCache Redis
resource "aws_elasticache_cluster" "redis" {
  cluster_id           = "gotong-royong-redis"
  engine               = "redis"
  node_type            = "cache.t3.micro"
  num_cache_nodes      = 1
  parameter_group_name = "default.redis7"
  port                 = 6379
  subnet_group_name    = aws_elasticache_subnet_group.main.name
  security_group_ids   = [aws_security_group.redis.id]

  tags = {
    Name = "gotong-royong-redis"
  }
}

# S3 Bucket for Evidence
resource "aws_s3_bucket" "evidence" {
  bucket = "gotong-royong-evidence"

  tags = {
    Name = "gotong-royong-evidence"
  }
}

resource "aws_s3_bucket_versioning" "evidence" {
  bucket = aws_s3_bucket.evidence.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "evidence" {
  bucket = aws_s3_bucket.evidence.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}
```

**variables.tf**:
```hcl
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "db_username" {
  description = "Database username"
  type        = string
  sensitive   = true
}

variable "db_password" {
  description = "Database password"
  type        = string
  sensitive   = true
}
```

**Deploy**:
```bash
terraform init
terraform plan
terraform apply
```

---

## Kubernetes Configuration

### Production Deployment

**namespace.yaml**:
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: gotong-royong
```

**configmap.yaml**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
  namespace: gotong-royong
data:
  REDIS_URL: "redis://redis-service:6379"
  S3_ENDPOINT: "https://s3.amazonaws.com"
  S3_BUCKET: "gotong-royong-evidence"
  MARKOV_API_URL: "https://api.markov.local"
```

**secrets.yaml**:
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
  namespace: gotong-royong
type: Opaque
data:
  DATABASE_URL: <base64-encoded>
  GOTONG_ROYONG_WEBHOOK_SECRET: <base64-encoded>
  AWS_ACCESS_KEY_ID: <base64-encoded>
  AWS_SECRET_ACCESS_KEY: <base64-encoded>
```

**deployment.yaml**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gotong-royong-api
  namespace: gotong-royong
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: gotong-royong-api
  template:
    metadata:
      labels:
        app: gotong-royong-api
        version: v1.0.0
    spec:
      containers:
      - name: api
        image: gotong-royong/api:1.0.0
        ports:
        - containerPort: 3000
        envFrom:
        - configMapRef:
            name: app-config
        - secretRef:
            name: app-secrets
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
```

**hpa.yaml** (Horizontal Pod Autoscaler):
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: gotong-royong-api-hpa
  namespace: gotong-royong
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: gotong-royong-api
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

**ingress.yaml**:
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: gotong-royong-ingress
  namespace: gotong-royong
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - api.gotong-royong.app
    secretName: gotong-royong-tls
  rules:
  - host: api.gotong-royong.app
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: gotong-royong-api
            port:
              number: 80
```

**Deploy**:
```bash
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f secrets.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f hpa.yaml
kubectl apply -f ingress.yaml
```

---

## Scaling Strategy

### Horizontal Scaling

**Triggers**:
- CPU usage > 70%
- Memory usage > 80%
- Request queue depth > 100

**Auto-scaling Configuration**:
```yaml
# Kubernetes HPA (see above)
minReplicas: 3
maxReplicas: 10
targetCPUUtilizationPercentage: 70
```

### Vertical Scaling

**Database**:
- Start: db.t3.medium (2 vCPU, 4 GB RAM)
- Scale up: db.t3.large (2 vCPU, 8 GB RAM)
- Scale up: db.m5.xlarge (4 vCPU, 16 GB RAM)

**Redis**:
- Start: cache.t3.micro (2 vCPU, 0.5 GB RAM)
- Scale up: cache.t3.small (2 vCPU, 1.37 GB RAM)
- Scale up: cache.t3.medium (2 vCPU, 3.09 GB RAM)

### Database Read Replicas

For read-heavy workloads:

```hcl
resource "aws_db_instance" "postgres_replica" {
  replicate_source_db = aws_db_instance.postgres.id
  instance_class      = "db.t3.medium"
  publicly_accessible = false

  tags = {
    Name = "gotong-royong-db-replica"
  }
}
```

---

## High Availability

### Multi-AZ Deployment

**Database**: Enable Multi-AZ for automatic failover

```hcl
resource "aws_db_instance" "postgres" {
  # ...
  multi_az = true
}
```

**Redis**: Use Redis Cluster mode

```hcl
resource "aws_elasticache_replication_group" "redis" {
  replication_group_id       = "gotong-royong-redis-cluster"
  replication_group_description = "Redis cluster for Gotong Royong"
  engine                     = "redis"
  node_type                  = "cache.t3.medium"
  num_cache_clusters         = 3
  automatic_failover_enabled = true
}
```

### Health Checks

**API Health Endpoint**:
```javascript
app.get('/health', async (req, res) => {
  const checks = {
    database: await checkDatabase(),
    redis: await checkRedis(),
    s3: await checkS3(),
  };

  const healthy = Object.values(checks).every(c => c === true);

  res.status(healthy ? 200 : 503).json({
    status: healthy ? 'healthy' : 'unhealthy',
    checks,
  });
});

async function checkDatabase() {
  try {
    await db.query('SELECT 1');
    return true;
  } catch (error) {
    return false;
  }
}
```

---

## Deployment Pipeline

### CI/CD with GitHub Actions

**.github/workflows/deploy.yml**:
```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run tests
        run: npm test

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build Docker image
        run: docker build -t gotong-royong/api:${{ github.sha }} .
      - name: Push to registry
        run: docker push gotong-royong/api:${{ github.sha }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Deploy to Kubernetes
        run: |
          kubectl set image deployment/gotong-royong-api \
            api=gotong-royong/api:${{ github.sha }} \
            -n gotong-royong
          kubectl rollout status deployment/gotong-royong-api -n gotong-royong
```

### Deployment Checklist

- [ ] Run database migrations
- [ ] Update environment variables
- [ ] Build and push Docker image
- [ ] Deploy to staging
- [ ] Run smoke tests
- [ ] Deploy to production (canary)
- [ ] Monitor error rates
- [ ] Complete rollout or rollback

---

## Cost Estimation

### Small Scale (10,000 users)

**AWS Costs**:
- EKS Cluster: $73/month
- EC2 Instances (3×t3.medium): $94/month
- RDS PostgreSQL (db.t3.medium): $61/month
- ElastiCache (cache.t3.micro): $12/month
- S3 Storage (500 GB): $12/month
- ALB: $23/month

**Total: ~$275/month**

### Medium Scale (100,000 users)

**AWS Costs**:
- EKS Cluster: $73/month
- EC2 Instances (6×t3.large): $377/month
- RDS PostgreSQL (db.m5.xlarge): $278/month
- ElastiCache (cache.t3.medium): $50/month
- S3 Storage (5 TB): $115/month
- ALB + CloudFront: $150/month

**Total: ~$1,043/month**

---

## References

- [Security Checklist](security-checklist.md) - Security hardening
- [Monitoring](monitoring.md) - Observability setup
- [Database Schema](../database/schema-requirements.md) - Database design
- [Storage Requirements](../por-evidence/storage-requirements.md) - S3 configuration
