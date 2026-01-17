# miniWiki Knowledge Management Platform

A self-hosted, Notion-like knowledge management platform built with Flutter for cross-platform support (Web, Desktop, Mobile) and Rust backend services.

## Features

- 📝 **Rich Document Editing** - Create and edit documents with Flutter Quill rich text editor
- 📁 **Document Organization** - Hierarchical spaces and nested documents
- 🔐 **User Authentication** - Secure JWT-based authentication with email verification
- 👥 **Role-Based Access Control** - Owner, Editor, Commenter, and Viewer roles
- 📱 **Offline-First** - Work without internet, automatic sync when online
- 🤝 **Real-Time Collaboration** - See other users' edits in real-time
- 📊 **Version History** - View and restore previous document versions
- 🔍 **Full-Text Search** - Fast search across all documents
- 📤 **Document Export** - Export to Markdown, HTML, and PDF
- 📎 **File Attachments** - Upload and manage file attachments
- 🔗 **Share Links** - Create share links for external document access

## Tech Stack

### Frontend

- **Flutter 3.27+** - Cross-platform framework
- **Riverpod** - State management
- **Dio** - HTTP client
- **Isar** - Offline database
- **y_crdt** - CRDT for sync
- **Flutter Quill** - Rich text editor

### Backend

- **Rust 1.75+** - Programming language
- **Actix-web** - Web framework
- **SQLx** - PostgreSQL database
- **Redis** - Caching and sessions
- **MinIO** - File storage (S3-compatible)

### Infrastructure

- **Docker Compose** - Local development
- **PostgreSQL 14+** - Primary database
- **Redis 6+** - Cache and sessions

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust 1.75+ (for backend development)
- Flutter 3.27+ (for frontend development)

### 1. Clone the Repository

```bash
git clone https://github.com/poyhsiao/miniWiki.git
cd miniWiki
```

### 2. Start Infrastructure

```bash
# Start PostgreSQL, Redis, and MinIO
docker-compose up -d

# Verify services are running
docker-compose ps
```

### 3. Configure Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit .env with your configuration
```

### 4. Run Backend

```bash
cd backend
cargo run
```

The API will be available at `http://localhost:8080`

### 5. Run Frontend

```bash
cd flutter_app
flutter run -d chrome
```

The web app will be available at `http://localhost:3000`

## Project Structure

```
miniWiki/
├── backend/                 # Rust backend services
│   ├── src/                # Actix-web API
│   ├── services/           # Microservices
│   │   ├── auth_service/   # Authentication
│   │   ├── document_service/ # Documents CRUD
│   │   ├── space_service/  # Spaces and organization
│   │   ├── sync_service/   # CRDT sync
│   │   ├── file_service/   # File attachments
│   │   ├── websocket_service/ # Real-time collaboration
│   │   └── search_service/ # Full-text search
│   ├── shared/             # Shared crates
│   └── migrations/         # SQL migrations
│
├── flutter_app/            # Flutter frontend
│   ├── lib/
│   │   ├── core/          # Core functionality
│   │   ├── domain/        # Business entities
│   │   ├── data/          # Data layer
│   │   ├── presentation/  # UI components
│   │   └── services/      # Business logic
│   └── test/              # Tests
│
├── specs/                  # Feature specifications
└── docker-compose.yml      # Local development
```

## API Documentation

API documentation is available at `/api/docs` when running the backend, or see the [OpenAPI specification](specs/001-miniwiki-platform/contracts/).

## Development

### Database Migrations

```bash
cd backend
sqlx migrate run
```

### Running Tests

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd flutter_app
flutter test
```

### Code Quality

```bash
# Backend linting
cd backend
cargo clippy

# Backend formatting
cargo fmt

# Frontend analysis
cd flutter_app
flutter analyze
```

## Deployment

### Production Build

```bash
# Backend
cd backend
cargo build --release

# Frontend
cd flutter_app
flutter build web
```

### Docker Production

```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Contributing

1. Create a feature branch
2. Implement your feature
3. Add tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details
