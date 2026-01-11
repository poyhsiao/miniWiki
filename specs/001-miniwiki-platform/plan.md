# Implementation Plan: miniWiki Platform

**Branch**: `001-miniwiki-platform` | **Date**: 2026-01-11 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-miniwiki-platform/spec.md`

## Summary

miniWiki is a self-hosted, Notion-like knowledge management platform built with Flutter for cross-platform support (Web, Desktop, Mobile) and Rust backend services. The system follows an offline-first architecture with CRDT-based synchronization using y_crdt (Yjs Dart port), enabling real-time collaboration with automatic conflict resolution. The platform supports document creation, organization, offline access, real-time collaboration, version history, authentication, RBAC, document export, and full-text search.

**Technical Approach**: Flutter frontend with Isar for offline storage and y_crdt (Yjs Dart port) for CRDT sync. Rust backend with Actix-web for API services. PostgreSQL for persistent storage, Redis for caching and sessions, MinIO for file storage. All services containerized with Docker Compose (MVP) and Kubernetes (production).

## Technical Context

**Language/Version**: Dart 3.x (Flutter 3.22+), Rust 1.75+  
**Primary Dependencies**: Riverpod (state management), Isar (offline DB), Dio (HTTP), Actix-web (backend), sqlx (Rust DB), y_crdt (Dart CRDT sync), Flutter Quill (editor)  
**Storage**: PostgreSQL 14+ (primary DB), Isar (Flutter offline), Redis 6+ (cache/sessions), MinIO (file storage)  
**Testing**: flutter_test, cargo test, Playwright (E2E)  
**Target Platform**: Web, Desktop (Windows/macOS/Linux), Mobile (iOS/Android)  
**Project Type**: Cross-platform mobile + web + desktop with separate backend services  
**Performance Goals**: <100ms UI response, <200ms API p95 latency, 95% search <500ms, 1,000 concurrent users  
**Constraints**: Offline-first mandatory, WCAG 2.1 AA accessibility, <10MB document limit, 50MB file attachment limit, TLS 1.3 required, AES-256 encryption at rest  
**Scale/Scope**: MVP 1,000-3,000 users, long-term 100,000-500,000 users

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| Offline-First Design | ✅ PASS | CRDT with y_crdt (Yjs), Isar offline storage |
| TDD Mandatory | ✅ PASS | Tests required for all features (>80% coverage) |
| Clean Code (<50 line functions) | ✅ PASS | Architecture enforces modular design |
| Security-First (JWT, RBAC, encryption) | ✅ PASS | JWT auth, RBAC, TLS 1.3, AES-256 |
| KISS (no over-design) | ✅ PASS | MVP scope defined, incremental features |
| External Config (.env/YAML) | ✅ PASS | Configuration externalized |
| Database Versioning | ✅ PASS | Migration files required |
| Context7 Dependency Verification | ✅ PASS | All dependencies verified |

**Gates Passed**: Ready for Phase 0 research

## Project Structure

### Documentation (this feature)

```
specs/001-miniwiki-platform/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (this file)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── auth.yaml
│   ├── documents.yaml
│   ├── sync.yaml
│   └── files.yaml
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
# Cross-platform Flutter app
flutter_app/
├── lib/
│   ├── main.dart                      # App entry point
│   ├── core/
│   │   ├── constants/                # App-wide constants
│   │   ├── errors/                  # Error handling
│   │   ├── theme/                   # App theming (light/dark)
│   │   └── utils/                   # Utility functions
│   ├── domain/
│   │   ├── entities/                # Business entities (User, Document, Space)
│   │   ├── repositories/            # Repository interfaces
│   │   └── value_objects/           # Value objects (Email, Password)
│   ├── data/
│   │   ├── datasources/             # Data sources (API, Local Isar)
│   │   ├── models/                  # Data models (DTOs)
│   │   └── repositories/            # Repository implementations
│   ├── presentation/
│   │   ├── providers/               # Riverpod providers
│   │   ├── pages/                  # Pages (Home, Editor, Settings)
│   │   ├── widgets/                # Reusable widgets
│   │   └── dialogs/                # Dialogs and modals
│   └── services/
│       ├── auth_service.dart        # Authentication logic
│       ├── document_service.dart    # Document operations
│       ├── sync_service.dart        # Sync orchestration
│       ├── offline_service.dart     # Offline management
│       └── crdt_service.dart        # CRDT operations
├── test/
│   ├── unit/                       # Unit tests
│   ├── widget/                     # Widget tests
│   └── integration/                # Integration tests
├── pubspec.yaml
└── assets/

# Rust backend services
backend/
├── Cargo.toml                       # Workspace configuration
├── services/
│   ├── auth_service/               # Authentication service
│   ├── document_service/           # Document CRUD
│   ├── sync_service/               # CRDT sync service (y_crdt)
│   ├── file_service/              # File storage service
│   └── websocket_service/          # WebSocket for presence
├── shared/
│   ├── database/                  # Database utilities
│   ├── models/                    # Shared models
│   └── errors/                     # Error types
└── migrations/                     # SQL migrations

# Docker deployment
docker-compose.yml
Dockerfile.web
Dockerfile.backend
nginx/
```

**Structure Decision**: Flutter app for all client platforms (Web, Desktop, Mobile) with separate Rust microservices backend. Modular architecture allows independent scaling of services.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Yjs CRDT integration | Required for offline-first and real-time collaboration without data loss | Last-write-wins loses data, OT is complex and error-prone |
| Multiple services | Independent scaling and maintainability for auth, documents, sync | Monolith simpler but doesn't scale for real-time collaboration |
| Isar + PostgreSQL | Offline requires local DB, server requires centralized DB | Single DB cannot support true offline-first with sync |

## Phase 0: Research & Decisions

### Technology Decisions

**Flutter for Cross-Platform**:
- Decision: Use Flutter 3.22+ for Web, Desktop, Mobile
- Rationale: Single codebase, 95% code sharing, native performance, hot reload, rich ecosystem (Riverpod, Isar, Dio)
- Alternatives: React Native (web overhead), Native (3x cost), Electron (poor mobile)

**Rust + Actix-web Backend**:
- Decision: Rust 1.75+ with Actix-web
- Rationale: Zero-cost abstractions, memory safety, async runtime, high performance
- Alternatives: Go (less type safety), Node.js (performance concerns), Python (too slow)

**Yjs CRDT for Sync**:
- Decision: y_crdt (Yjs Dart port) for document sync and conflict resolution
- Rationale: Automatic conflict resolution, offline-first, sub-millisecond latency, proven by Notion
- Alternatives: OT (complex), Last-write-wins (data loss), Manual merge (blocking)

**PostgreSQL + Isar**:
- Decision: PostgreSQL 14+ (server), Isar (Flutter offline)
- Rationale: ACID compliance, JSONB for CRDT states, mature ecosystem. Isar is Flutter-native, high performance
- Alternatives: MongoDB (no ACID), MySQL (weaker JSONB), SQLite (less mature Flutter support)

**MinIO for File Storage**:
- Decision: MinIO (S3-compatible, self-hosted)
- Rationale: Complete control, no cloud dependency, high performance, Kubernetes-native
- Alternatives: AWS S3 (vendor lock-in), Local filesystem (no versioning)

### Key Research Findings

1. **CRDT Performance**: Yjs handles documents up to 10MB efficiently. Beyond 10MB, performance degrades. Limit enforced at 10MB per document.

2. **Offline Sync Strategy**: Local Isar stores pending changes (isDirty flag). Background sync service reconciles when online. CRDT automatically merges conflicts.

3. **WebSocket Presence**: Redis pub/sub for presence across multiple backend instances. Heartbeat every 30s, cleanup after 5min inactivity.

4. **Rate Limiting**: Token bucket algorithm in Redis. 100 req/hour anonymous, 1000 req/hour authenticated.

## Phase 1: Design Artifacts

### Data Model

See [data-model.md](data-model.md) for detailed entity definitions.

### API Contracts

See [contracts/](contracts/) directory for OpenAPI specifications:
- [auth.yaml](contracts/auth.yaml) - Authentication endpoints
- [documents.yaml](contracts/documents.yaml) - Document CRUD, versions
- [sync.yaml](contracts/sync.yaml) - CRDT sync endpoints
- [files.yaml](contracts/files.yaml) - File upload/download

### Quick Start Guide

See [quickstart.md](quickstart.md) for development setup instructions.

## Next Steps

Proceed to `/speckit.tasks` to generate implementation tasks based on this plan.
