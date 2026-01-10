# miniWiki Development Quickstart

**Feature**: miniWiki Knowledge Management Platform  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Data Model**: [data-model.md](data-model.md)  
**Date**: 2026-01-11

## Prerequisites

### System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| OS | macOS 14+, Ubuntu 22.04+, Windows 11 | macOS 15+ |
| RAM | 8 GB | 16 GB |
| Storage | 10 GB free | 50 GB free |
| CPU | 4 cores | 8 cores |

### Required Tools

Install these tools before proceeding:

```bash
# macOS with Homebrew
brew install \
  flutter \
  rustup-init \
  docker \
  docker-compose \
  git \
  cmake \
  pkg-config \
  openssl

# Ubuntu/Debian
sudo apt-get install \
  flutter \
  rustc cargo \
  docker.io docker-compose \
  git \
  cmake \
  pkg-config \
  libssl-dev

# Verify installations
flutter --version        # Should be 3.22+
rustc --version          # Should be 1.75+
docker --version         # Should be 24+
docker compose version   # Should be 2.20+
```

## Quick Start (5 minutes)

### 1. Clone Repository

```bash
git clone https://github.com/kimhsiao/miniWiki.git
cd miniWiki
git checkout 001-miniwiki-platform
```

### 2. Start Infrastructure

```bash
# Start all services (PostgreSQL, Redis, MinIO)
docker compose up -d

# Verify services are running
docker compose ps

# Expected output:
# NAME                 STATUS
# miniwiki-postgres     Up (healthy)
# miniwiki-redis        Up (healthy)
# miniwiki-minio        Up (healthy)
```

### 3. Configure Environment

```bash
# Copy environment template
cp .env.example .env

# Edit with your settings (defaults work for local dev)
nano .env
```

**`.env.example`**:
```env
# Application
APP_ENV=development
APP_HOST=0.0.0.0
APP_PORT=8080

# Database
DATABASE_URL=postgresql://miniwiki:miniwiki@localhost:5432/miniwiki
DB_POOL_SIZE=10

# Redis
REDIS_URL=redis://localhost:6379

# MinIO
MINIO_ENDPOINT=localhost:9000
MINIO_ACCESS_KEY=miniwiki
MINIO_SECRET_KEY=miniwiki123
MINIO_BUCKET=miniwiki-files

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_ACCESS_TOKEN_EXPIRY=900
JWT_REFRESH_TOKEN_EXPIRY=604800

# Email (for dev, use Mailhog)
SMTP_HOST=localhost
SMTP_PORT=1025
SMTP_USER=
SMTP_PASS=
EMAIL_FROM=noreply@miniwiki.local

# Features
ENABLE_OFFLINE_SYNC=true
ENABLE_REAL_TIME_COLLAB=true
```

### 4. Run Database Migrations

```bash
# Apply migrations
cd backend
cargo run --bin migrate

# Or with Docker
docker exec -it miniwiki-backend-1 cargo run --bin migrate
```

### 5. Start Backend

```bash
# Terminal 1: Start Rust backend
cd backend
cargo run --bin api

# Backend will start on http://localhost:8080
# Health check: http://localhost:8080/health
```

### 6. Start Frontend

```bash
# Terminal 2: Start Flutter app
cd flutter_app

# Install dependencies
flutter pub get

# Run on web (default)
flutter run -d chrome

# Or run on desktop
flutter run -d macos
flutter run -d windows
flutter run -d linux

# Or run on mobile
flutter run -d ios
flutter run -d android
```

### 7. Verify Installation

Open browser and navigate to:
- **Web App**: http://localhost:5173 (or the port shown by Flutter)
- **API Health**: http://localhost:8080/health
- **API Docs**: http://localhost:8080/docs (if swagger enabled)

## Development Workflow

### Running Tests

```bash
# Backend tests
cd backend
cargo test                    # Run all tests
cargo test --lib             # Library tests only
cargo test --test integration # Integration tests

# Frontend tests
cd flutter_app
flutter test                  # Unit tests
flutter test --integration   # Integration tests
flutter drive --target=test_driver/app.dart # E2E tests
```

### Code Formatting

```bash
# Rust
cd backend
cargo fmt                    # Format code
cargo fmt -- --check         # Check formatting
cargo clippy                 # Lint with clippy

# Flutter
cd flutter_app
flutter format lib/          # Format Dart code
flutter analyze              # Static analysis
dart fix --apply             # Apply fixes
```

### Database Management

```bash
# Run migrations
cd backend
cargo run --bin migrate

# Rollback last migration
cargo run --bin migrate -- rollback

# Create new migration
cargo run --bin migrate -- create "migration_name"

# Connect to PostgreSQL
docker exec -it miniwiki-postgres psql -U miniwiki -d miniwiki

# Connect to Redis
docker exec -it miniwiki-redis redis-cli

# Access MinIO Console
# Open http://localhost:9001
# User: miniwiki / miniwiki123
```

### Viewing Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f backend
docker compose logs -f postgres

# With timestamp
docker compose logs -f -t
```

## Project Structure

```
miniWiki/
├── flutter_app/              # Flutter client
│   ├── lib/
│   │   ├── main.dart        # Entry point
│   │   ├── core/           # Constants, errors, theme, utils
│   │   ├── domain/         # Entities, repositories, value objects
│   │   ├── data/           # Data sources, models, repositories
│   │   ├── presentation/   # Providers, pages, widgets, dialogs
│   │   └── services/       # Auth, document, sync, offline, CRDT
│   ├── test/               # Unit, widget, integration tests
│   ├── pubspec.yaml
│   └── assets/
│
├── backend/                 # Rust microservices
│   ├── services/
│   │   ├── auth_service/   # Authentication
│   │   ├── document_service/ # Document CRUD
│   │   ├── sync_service/   # Yjs sync
│   │   ├── file_service/   # File storage
│   │   └── websocket_service/ # WebSocket
│   ├── shared/
│   │   ├── database/       # Database utilities
│   │   ├── models/         # Shared models
│   │   └── errors/         # Error types
│   ├── migrations/         # SQL migrations
│   ├── Cargo.toml
│   └── Dockerfile
│
├── specs/
│   └── 001-miniwiki-platform/
│       ├── spec.md
│       ├── plan.md
│       ├── data-model.md
│       ├── quickstart.md   # This file
│       ├── contracts/
│       │   ├── auth.yaml
│       │   ├── documents.yaml
│       │   ├── sync.yaml
│       │   └── files.yaml
│       └── tasks.md
│
├── docker-compose.yml
├── .env.example
├── nginx/
└── README.md
```

## Useful Commands

### Backend Development

```bash
# Watch mode (auto-reload)
cargo watch -x run

# Run with specific config
cargo run -- --config development.toml

# Check dependencies
cargo check
cargo update

# Build release
cargo build --release

# Benchmark
cargo bench
```

### Frontend Development

```bash
# Hot reload (in running flutter run)
r  # Press 'r' in terminal

# Build for web
flutter build web

# Build for desktop
flutter build macos
flutter build windows
flutter build linux

# Build for mobile
flutter build ios --release
flutter build apk --release

# Analyze for performance
flutter build apk --release --analyze-size
```

### Docker Operations

```bash
# Rebuild services
docker compose build

# Restart services
docker compose restart

# Stop all services
docker compose down

# Stop and remove volumes
docker compose down -v

# View resource usage
docker stats
```

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Port already in use | Change port in `.env` or stop conflicting service |
| Flutter not found | Add Flutter to PATH: `export PATH="$PATH:`pwd`/flutter/bin"` |
| Rust toolchain error | Run `rustup default stable` |
| Docker permission denied | Add user to docker group: `sudo usermod -aG docker $USER` |
| Database connection failed | Verify PostgreSQL is running: `docker compose ps` |
| MinIO access denied | Check credentials in `.env` |
| CORS errors | Ensure `APP_URL` is set correctly in `.env` |

### Getting Help

```bash
# Check logs
docker compose logs backend

# Verify environment
cd backend && cargo run --bin check-env

# Run diagnostics
flutter doctor
rustc --version
docker --version
```

## Next Steps

1. **Read the spec**: Review [spec.md](spec.md) for feature requirements
2. **Understand architecture**: See [architecture.md](architecture.md) for system design
3. **Review data model**: Check [data-model.md](data-model.md) for entity definitions
4. **Check API contracts**: Browse [contracts/](contracts/) for endpoint specifications
5. **Pick a task**: See [tasks.md](tasks.md) for implementation tasks

## Additional Resources

- **Flutter Docs**: https://docs.flutter.dev
- **Riverpod**: https://riverpod.dev
- **Rust Docs**: https://doc.rust-lang.org
- **Actix-web**: https://actix.rs/docs
- **Yjs Docs**: https://docs.yjs.dev
- **PostgreSQL**: https://www.postgresql.org/docs
- **MinIO**: https://docs.min.io
