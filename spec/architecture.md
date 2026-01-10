# Technical Architecture Specification

## 1. Executive Summary

### 1.1 System Overview
miniWiki is a self-hosted, Notion-like knowledge management platform built with Flutter for cross-platform support (Web, Desktop, Mobile) and Rust backend services. The system follows an offline-first architecture with CRDT-based synchronization, enabling real-time collaboration with automatic conflict resolution.

### 1.2 Architecture Philosophy
- **Cross-Platform First**: Flutter provides unified UI/UX across all platforms (Web, Desktop, Mobile)
- **Offline-First**: Full functionality without internet, seamless sync when online
- **CRDT-Based Sync**: Yjs CRDT for automatic conflict resolution
- **Self-Hosted**: Complete deployment control via Docker/Kubernetes
- **Microservices**: Modular backend services for scalability and maintainability

### 1.3 High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Client Layer                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │  Web     │  │ Desktop  │  │  Mobile  │              │
│  │ (Flutter) │  │(Flutter) │  │(Flutter) │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘
                               │
                               │ HTTP/WebSocket
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        API Gateway / Nginx                              │
│                         (Load Balancing, SSL)                              │
└─────────────────────────────────────────────────────────────────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        ▼                      ▼                      ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Auth Service │    │ Document     │    │  Sync        │
│  (Rust)      │    │ Service      │    │  Service     │
│              │    │  (Rust)      │    │  (Yjs CRDT) │
└──────────────┘    └──────────────┘    └──────────────┘
        │                      │                      │
        └──────────────────────┼──────────────────────┘
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Data Layer                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ PostgreSQL   │  │    Redis     │  │    MinIO     │   │
│  │ (Documents,  │  │    (Cache,   │  │    (Files,    │   │
│  │  Users,      │  │     Session)  │  │     Objects)  │   │
│  │  Spaces)     │  │              │  │              │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.4 Technology Stack Summary

| Layer | Technology | Rationale |
|-------|-------------|------------|
| **Frontend** | Flutter (3.22+) | Cross-platform (Web, Desktop, Mobile), hot reload, rich widget ecosystem |
| **Backend** | Rust + Actix-web | Performance, memory safety, async runtime, zero-cost abstractions |
| **Database** | PostgreSQL 14+ | ACID compliance, JSONB for document storage, mature ecosystem |
| **Cache** | Redis 6+ | In-memory caching, pub/sub for WebSocket, session management |
| **Storage** | MinIO | S3-compatible, self-hosted, high performance, Kubernetes-native |
| **Sync** | Yjs CRDT | Automatic conflict resolution, real-time collaboration, offline-first |
| **Offline DB** | Isar (Flutter) | High-performance, Flutter-native, supports CRDT metadata |
| **Editor** | Flutter Quill | WYSIWYG editing, Markdown support, cross-platform |
| **HTTP** | Dio (Flutter) | HTTP client, interceptors, cancellation tokens |
| **WebSocket** | Flutter WebSocket SDK | Real-time sync, presence, collaboration |
| **Deployment** | Docker + Kubernetes | Containerization, orchestration, scaling, self-hosted |

---

## 2. System Architecture

### 2.1 Client Architecture (Flutter)

#### 2.1.1 Project Structure
```
flutter_app/
├── lib/
│   ├── main.dart                      # App entry point
│   ├── core/
│   │   ├── constants/               # App-wide constants
│   │   ├── errors/                   # Error handling
│   │   ├── theme/                    # App theming (light/dark)
│   │   └── utils/                    # Utility functions
│   ├── domain/
│   │   ├── entities/                # Business entities (User, Document, Space)
│   │   ├── repositories/            # Repository interfaces
│   │   └── value_objects/            # Value objects (Email, Password)
│   ├── data/
│   │   ├── datasources/              # Data sources (API, Local)
│   │   ├── models/                   # Data models (DTOs)
│   │   └── repositories/            # Repository implementations
│   ├── presentation/
│   │   ├── providers/                # Riverpod providers
│   │   ├── pages/                   # Pages (Home, Editor, Settings)
│   │   ├── widgets/                 # Reusable widgets
│   │   └── dialogs/                 # Dialogs and modals
│   └── services/
│       ├── auth_service.dart           # Authentication logic
│       ├── document_service.dart       # Document operations
│       ├── sync_service.dart           # Sync orchestration
│       ├── offline_service.dart         # Offline management
│       └── crdt_service.dart          # CRDT operations
├── test/
│   ├── unit/                         # Unit tests
│   ├── widget/                       # Widget tests
│   └── integration/                  # Integration tests
├── pubspec.yaml                      # Dependencies
└── assets/
    ├── images/                        # Images
    ├── fonts/                         # Fonts (Inter, JetBrains Mono)
    └── translations/                  # i18n JSON files
```

#### 2.1.2 State Management (Riverpod)
- **Provider Pattern**: Riverpod for state management
- **State Providers**: Separate providers for Auth, Documents, Spaces, Sync
- **Immutable State**: All state is immutable, updated via `copyWith`
- **Async State**: `AsyncNotifier` for async operations
- **Computed State**: Derived state via `select`

Example:
```dart
// Provider
final documentProvider = StateNotifierProvider<DocumentState>((ref) {
  return DocumentNotifier(ref.read(apiServiceProvider));
});

// State
class DocumentState {
  final List<Document> documents;
  final Document? currentDocument;
  final bool isLoading;
  final String? error;

  DocumentState copyWith({
    List<Document>? documents,
    Document? currentDocument,
    bool? isLoading,
    String? error,
  }) {
    return DocumentState(
      documents: documents ?? this.documents,
      currentDocument: currentDocument ?? this.currentDocument,
      isLoading: isLoading ?? this.isLoading,
      error: error ?? this.error,
    );
  }
}
```

#### 2.1.3 Offline Architecture (Isar + Yjs)

**Isar Schema**:
```dart
@collection
class Document {
  Id id = Isar.autoIncrement;

  @Index()
  late String documentId;  // Remote document ID

  @Index()
  late String spaceId;

  late String title;
  late String content;  // Yjs CRDT state
  late DateTime createdAt;
  late DateTime updatedAt;
  late DateTime? deletedAt;

  // Sync metadata
  late int version;
  late bool isDirty;  // Has unsynced changes
  late DateTime? lastSyncedAt;

  // CRDT metadata
  late String yjsState;  // Serialized Yjs document state
}
```

**Yjs Integration**:
- **Yjs Document**: Each document is a Yjs CRDT document
- **Local Edits**: Applied to local Yjs document
- **Sync Algorithm**:
  1. Serialize local Yjs state (`yjsState`)
  2. Send to server with version number
  3. Server merges with remote Yjs state
  4. Receive merged state and apply locally
  5. Update Isar with new state and increment version

**Sync Flow**:
```
User Edit → Local Yjs Document → Isar (isDirty: true)
                                            │
                                            ▼
                                    Sync Service (background)
                                            │
            ┌───────────────────────────┼───────────────────────────┐
            ▼                           ▼                           ▼
      Fetch Remote             Merge Yjs States         Push to Server
         │                            │                           │
         ▼                            ▼                           ▼
    Remote Yjs           Merged Yjs State         Server Merge
         │                            │                           │
         └────────────────────────────┴───────────────────────────┘
                                    │
                                    ▼
                          Update Local Isar (isDirty: false, version++)
                                    │
                                    ▼
                            Notify UI (Riverpod)
```

### 2.2 Backend Architecture (Rust)

#### 2.2.1 Project Structure
```
backend/
├── Cargo.toml                        # Workspace configuration
├── services/
│   ├── auth_service/                # Authentication service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── handlers/
│   │       ├── models/
│   │       ├── repositories/
│   │       └── middleware/
│   ├── document_service/             # Document service
│   ├── sync_service/                 # Sync service (Yjs)
│   ├── file_service/                 # File storage service
│   └── websocket_service/             # WebSocket service (presence)
├── shared/
│   ├── database/                     # Database utilities
│   ├── models/                        # Shared models
│   └── errors/                        # Error types
└── migrations/                       # SQL migrations
```

#### 2.2.2 Actix-Web Architecture
- **Actix-Web**: Async web framework
- **HTTP/1.1 & HTTP/2**: Supported
- **Middleware**: Authentication, logging, CORS, rate limiting
- **Extractors**: JWT validation, user context
- **Error Handling**: Custom error types with proper HTTP status codes

Example:
```rust
// Main.rs
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Database pool
    let pool = database::create_pool().await?;

    // HTTP Server
    HttpServer::new(move || {
        let app = App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Cors::permissive())
            .wrap(middleware::Condition::new()
                .branch(cfg::get_config().auth_enabled, auth_middleware);

        // Routes
        app.service(web::scope("/api/auth")
                .route("/register", web::post().to(auth::register))
                .route("/login", web::post().to(auth::login))
            .service(web::scope("/api/documents")
                .route("", web::get().to(document::list))
                .route("", web::post().to(document::create))
                .route("/{id}", web::get().to(document::get))
                .route("/{id}", web::put().to(document::update))
                .route("/{id}", web::delete().to(document::delete))
            .route("/ws", web::get().to(websocket::index));

        app
    })
    .bind(("127.0.0.1", 8080))?
    .run()
}
```

#### 2.2.3 Yjs Sync Service
- **Yjs Server**: Node.js server for CRDT operations
- **Rust Wrapper**: Rust service wraps Yjs server via HTTP
- **Database**: PostgreSQL for persistent storage
- **WebSocket**: Real-time updates to connected clients

**Sync Algorithm**:
```rust
// Sync request
#[derive(Serialize, Deserialize)]
pub struct SyncRequest {
    pub document_id: String,
    pub version: i64,
    pub yjs_state: String,  // Serialized Yjs state
}

// Sync response
#[derive(Serialize, Deserialize)]
pub struct SyncResponse {
    pub document_id: String,
    pub version: i64,
    pub yjs_state: String,  // Merged Yjs state
}

// Sync handler
pub async fn sync_document(
    req: Json<SyncRequest>,
    pool: web::Data<DbPool>,
) -> Result<Json<SyncResponse>> {
    let sync_req = req.into_inner();

    // Fetch current document state
    let doc = sqlx::query!(
        &pool,
        "SELECT yjs_state, version FROM documents WHERE id = $1",
        sync_req.document_id
    ).fetch_one().await?;

    // Merge Yjs states
    let merged_state = yjs::merge_states(&doc.yjs_state, &sync_req.yjs_state)?;

    // Update document
    let new_version = doc.version + 1;
    sqlx::query!(
        &pool,
        "UPDATE documents SET yjs_state = $1, version = $2 WHERE id = $3",
        merged_state,
        new_version,
        sync_req.document_id
    ).execute().await?;

    // Broadcast to connected clients via WebSocket
    websocket::broadcast_update(&sync_req.document_id, &merged_state).await;

    Ok(Json(SyncResponse {
        document_id: sync_req.document_id,
        version: new_version,
        yjs_state: merged_state,
    }))
}
```

### 2.3 Database Architecture

#### 2.3.1 PostgreSQL Schema

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    avatar_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Spaces (workspaces) table
CREATE TABLE spaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Space memberships (many-to-many)
CREATE TABLE space_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID REFERENCES spaces(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- 'owner', 'editor', 'commenter', 'viewer'
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(space_id, user_id)
);

-- Documents table
CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID REFERENCES spaces(id) ON DELETE CASCADE,
    owner_id UUID REFERENCES users(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    yjs_state TEXT NOT NULL, -- Serialized Yjs CRDT state
    version INTEGER NOT NULL DEFAULT 0,
    parent_id UUID REFERENCES documents(id) ON DELETE CASCADE, -- For folders
    is_folder BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Document versions (for version history)
CREATE TABLE document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    yjs_state TEXT NOT NULL,
    version INTEGER NOT NULL,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Comments table
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID REFERENCES documents(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE, -- For threads
    content TEXT NOT NULL,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Files table
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL,
    storage_key TEXT NOT NULL, -- MinIO key
    is_compressed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit log table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL, -- 'create', 'update', 'delete', 'share'
    entity_type VARCHAR(100) NOT NULL, -- 'document', 'space', 'user'
    entity_id UUID NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_documents_space_id ON documents(space_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_documents_parent_id ON documents(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_comments_document_id ON comments(document_id);
CREATE INDEX idx_files_document_id ON files(document_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
```

#### 2.3.2 Redis Data Structures
```
# Session storage (access tokens)
session:{token} → { user_id: uuid, expires_at: timestamp } (TTL: 15min)

# Refresh tokens
refresh_token:{token} → { user_id: uuid, expires_at: timestamp } (TTL: 7 days)

# Rate limiting
rate_limit:{user_id}:{endpoint} → { count: int, window_start: timestamp } (TTL: 1min)

# Document locks (for editing)
document_lock:{document_id} → { user_id: uuid, locked_at: timestamp } (TTL: 30min)

# WebSocket presence (who is viewing/editing)
presence:{document_id} → { user_id: { cursor_position: int, is_editing: bool } } (TTL: 5min)

# Cache (frequently accessed data)
cache:document:{document_id} → { document_json } (TTL: 5min)
cache:space:{space_id} → { space_json } (TTL: 10min)
cache:user:{user_id} → { user_json } (TTL: 1hour)
```

---

## 3. API Architecture

### 3.1 REST API Design

#### 3.1.1 API Versioning
- **URL Versioning**: `/api/v1/` prefix
- **Backward Compatibility**: Maintain older versions for at least 6 months
- **Deprecation Policy**: 3-month deprecation notice

#### 3.1.2 Authentication Endpoints

```
POST /api/v1/auth/register
Request:
{
  "email": "user@example.com",
  "password": "securepassword123",
  "full_name": "John Doe"
}
Response:
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": { ... }
}

POST /api/v1/auth/login
Request:
{
  "email": "user@example.com",
  "password": "securepassword123"
}
Response:
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": { ... }
}

POST /api/v1/auth/logout
Headers: Authorization: Bearer {access_token}
Response: 204 No Content

POST /api/v1/auth/refresh
Request:
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
Response:
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### 3.1.3 Document Endpoints

```
GET /api/v1/documents
Query Params: space_id, parent_id, search, limit, offset
Headers: Authorization: Bearer {access_token}
Response:
{
  "documents": [
    {
      "id": "uuid",
      "space_id": "uuid",
      "title": "Document Title",
      "version": 5,
      "is_folder": false,
      "created_at": "2026-01-11T00:00:00Z",
      "updated_at": "2026-01-11T12:00:00Z"
    }
  ],
  "total": 100,
  "limit": 20,
  "offset": 0
}

GET /api/v1/documents/{id}
Headers: Authorization: Bearer {access_token}
Response:
{
  "id": "uuid",
  "space_id": "uuid",
  "title": "Document Title",
  "yjs_state": "serialized Yjs state",
  "version": 5,
  "created_at": "2026-01-11T00:00:00Z",
  "updated_at": "2026-01-11T12:00:00Z"
}

POST /api/v1/documents
Headers: Authorization: Bearer {access_token}
Request:
{
  "space_id": "uuid",
  "title": "New Document",
  "yjs_state": "serialized Yjs state",
  "parent_id": "uuid" // Optional
}
Response: 201 Created
{
  "id": "uuid",
  ... // Same as GET response
}

PUT /api/v1/documents/{id}
Headers: Authorization: Bearer {access_token}
Request:
{
  "title": "Updated Title",
  "yjs_state": "updated Yjs state"
}
Response:
{
  "id": "uuid",
  ... // Same as GET response
}

DELETE /api/v1/documents/{id}
Headers: Authorization: Bearer {access_token}
Response: 204 No Content

GET /api/v1/documents/{id}/versions
Headers: Authorization: Bearer {access_token}
Response:
{
  "versions": [
    {
      "id": "uuid",
      "version": 4,
      "yjs_state": "serialized Yjs state",
      "created_by": { ... },
      "created_at": "2026-01-11T11:00:00Z"
    }
  ]
}

POST /api/v1/documents/{id}/restore
Headers: Authorization: Bearer {access_token}
Request:
{
  "version": 4
}
Response:
{
  "id": "uuid",
  ... // Same as GET response
}
```

#### 3.1.4 Sync Endpoint

```
POST /api/v1/sync
Headers: Authorization: Bearer {access_token}
Request:
{
  "document_id": "uuid",
  "version": 5,
  "yjs_state": "serialized Yjs state"
}
Response:
{
  "document_id": "uuid",
  "version": 6,
  "yjs_state": "merged Yjs state"
}
```

#### 3.1.5 File Upload Endpoint

```
POST /api/v1/files/upload
Headers: Authorization: Bearer {access_token}
Content-Type: multipart/form-data
Request:
{
  "file": <binary data>,
  "document_id": "uuid"
}
Response:
{
  "id": "uuid",
  "document_id": "uuid",
  "filename": "example.png",
  "content_type": "image/png",
  "size_bytes": 12345,
  "storage_key": "files/uuid/example.png"
}
```

### 3.2 WebSocket Architecture

#### 3.2.1 Connection Flow
1. **Client Connects**: `ws://server/ws?token={access_token}`
2. **Authentication**: JWT token validated
3. **Session Created**: WebSocket session associated with user
4. **Subscriptions**: Client subscribes to document updates
5. **Presence**: Client broadcasts cursor position, receives others' cursors

#### 3.2.2 Message Format

```json
// Client → Server (Subscribe to document)
{
  "type": "subscribe",
  "document_id": "uuid"
}

// Client → Server (Cursor update)
{
  "type": "cursor_update",
  "document_id": "uuid",
  "position": { "line": 10, "column": 15 }
}

// Server → Client (Document update)
{
  "type": "document_update",
  "document_id": "uuid",
  "version": 6,
  "yjs_state": "merged Yjs state"
}

// Server → Client (Presence update)
{
  "type": "presence_update",
  "document_id": "uuid",
  "users": [
    {
      "user_id": "uuid",
      "full_name": "John Doe",
      "cursor_position": { "line": 10, "column": 15 }
    }
  ]
}

// Server → Client (Error)
{
  "type": "error",
  "message": "Unauthorized"
}
```

#### 3.2.3 Connection Management
- **Heartbeat**: Ping/Pong every 30 seconds
- **Reconnection**: Exponential backoff (1s, 2s, 4s, 8s, 16s)
- **Session Cleanup**: Remove from presence after 5 minutes of inactivity

---

## 4. Authentication & Authorization

### 4.1 Authentication Flow

```
┌──────────┐
│  Client  │
└────┬─────┘
     │
     │ 1. Register/Login
     ▼
┌──────────────┐
│ Auth Service │
└──────┬───────┘
       │
       │ 2. Validate credentials
       ▼
┌──────────────┐
│ PostgreSQL  │
└──────┬───────┘
       │
       │ 3. User exists, hash matches
       ▼
┌──────────────┐
│ Auth Service │
└──────┬───────┘
       │
       │ 4. Generate JWT tokens
       ▼
┌──────────────┐
│   Redis     │  (Store session)
└──────┬───────┘
       │
       │ 5. Return tokens
       ▼
┌──────────┐
│  Client  │  (Store tokens securely)
└──────────┘
```

### 4.2 JWT Tokens

**Access Token**:
- **Algorithm**: RS256 (asymmetric)
- **Expiration**: 15 minutes
- **Claims**:
  ```json
  {
    "sub": "user_id",
    "email": "user@example.com",
    "role": "editor",
    "space_ids": ["uuid1", "uuid2"],
    "exp": 1736618400,
    "iat": 1736618400
  }
  ```

**Refresh Token**:
- **Algorithm**: HS256 (symmetric)
- **Expiration**: 7 days
- **Storage**: httpOnly cookie (web), secure storage (Flutter)

### 4.3 Role-Based Access Control (RBAC)

**Roles**:
- **Owner**: Full control over space and all documents
- **Editor**: Create, edit, delete documents
- **Commenter**: View documents, add comments
- **Viewer**: Read-only access

**Permission Matrix**:

| Action | Owner | Editor | Commenter | Viewer |
|--------|--------|--------|-----------|--------|
| Create Document | ✅ | ✅ | ❌ | ❌ |
| Edit Document | ✅ | ✅ | ❌ | ❌ |
| Delete Document | ✅ | ✅ | ❌ | ❌ |
| Add Comment | ✅ | ✅ | ✅ | ❌ |
| View Document | ✅ | ✅ | ✅ | ✅ |
| Manage Members | ✅ | ❌ | ❌ | ❌ |
| Change Space Settings | ✅ | ❌ | ❌ | ❌ |

**Authorization Middleware**:
```rust
pub async fn authorize(
    req: ServiceRequest,
    pool: web::Data<DbPool>,
) -> Result<ServiceRequest, Error> {
    // Extract JWT from Authorization header
    let token = req
        .headers()
        .get("authorization")
        .and_then(|h| h.strip_prefix("Bearer "));

    // Validate token
    let claims = jwt::validate(token)?;

    // Check user has required permission
    let document_id = req.match_info().get("id").unwrap();
    let permission = check_permission(&pool, &claims.sub, document_id, "edit").await?;

    if !permission {
        return Err(Error::Forbidden);
    }

    // Add user context to request
    req.extensions_mut().insert(claims);
    Ok(req)
}
```

---

## 5. File Storage Architecture

### 5.1 MinIO Configuration

**Buckets**:
- `documents`: Document attachments (images, videos, PDFs)
- `avatars`: User profile images
- `exports`: Exported documents (PDF, HTML, Markdown)
- `backups`: Database backups

**Storage Policy**:
- **Lifecycle Rules**: Transition to Glacier after 90 days (optional)
- **Versioning**: Enabled for backup/restore
- **Encryption**: Server-side encryption (SSE-S3)
- **Access Control**: Presigned URLs for direct upload/download

### 5.2 File Upload Flow

```
┌──────────┐
│  Client  │
└────┬─────┘
     │
     │ 1. Initiate upload
     ▼
┌──────────────┐
│ File Service │
└──────┬───────┘
       │
       │ 2. Generate presigned URL
       ▼
┌──────────────┐
│   MinIO     │
└──────┬───────┘
       │
       │ 3. Return presigned URL
       ▼
┌──────────┐
│  Client  │
└────┬─────┘
     │
     │ 4. Upload directly to MinIO
     ▼
┌──────────────┐
│   MinIO     │
└──────┬───────┘
       │
       │ 5. Notify backend (file uploaded)
       ▼
┌──────────────┐
│ File Service │
└──────┬───────┘
       │
       │ 6. Create file record in PostgreSQL
       ▼
┌──────────────┐
│ PostgreSQL  │
└──────────────┘
```

### 5.3 File Compression

**Configuration**:
```yaml
compression:
  enabled: true
  tiers:
    - name: "high"
      max_size_mb: 5
      quality: 70
    - name: "medium"
      max_size_mb: 20
      quality: 80
    - name: "low"
      max_size_mb: 50
      quality: 90
```

**Supported Formats**:
- **Images**: JPEG, PNG (converted to WebP for compression)
- **Videos**: MP4 (transcoded to H.264)
- **Documents**: PDF (no compression, size limit enforced)

---

## 6. Deployment Architecture

### 6.1 Docker Compose (MVP)

```yaml
version: '3.8'

services:
  # Frontend (Flutter Web)
  frontend:
    build:
      context: ./flutter_app
      dockerfile: Dockerfile.web
    ports:
      - "3000:80"
    environment:
      - API_BASE_URL=http://api-gateway:8080
    depends_on:
      - api-gateway

  # API Gateway
  api-gateway:
    image: nginx:1.25
    ports:
      - "8080:80"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - auth-service
      - document-service
      - sync-service
      - websocket-service

  # Auth Service
  auth-service:
    build:
      context: ./backend/services/auth_service
      dockerfile: Dockerfile
    environment:
      - DATABASE_URL=postgres://user:password@postgres:5432/miniwiki
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis

  # Document Service
  document-service:
    build:
      context: ./backend/services/document_service
      dockerfile: Dockerfile
    environment:
      - DATABASE_URL=postgres://user:password@postgres:5432/miniwiki
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis

  # Sync Service (Yjs)
  sync-service:
    build:
      context: ./backend/services/sync_service
      dockerfile: Dockerfile
    environment:
      - DATABASE_URL=postgres://user:password@postgres:5432/miniwiki
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis

  # WebSocket Service
  websocket-service:
    build:
      context: ./backend/services/websocket_service
      dockerfile: Dockerfile
    environment:
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis

  # PostgreSQL
  postgres:
    image: postgres:14
    environment:
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=miniwiki
    volumes:
      - postgres-data:/var/lib/postgresql/data

  # Redis
  redis:
    image: redis:7
    volumes:
      - redis-data:/data

  # MinIO
  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      - MINIO_ROOT_USER=admin
      - MINIO_ROOT_PASSWORD=password
    volumes:
      - minio-data:/data

volumes:
  postgres-data:
  redis-data:
  minio-data:
```

### 6.2 Kubernetes (Production)

**Deployment Strategy**:
- **Namespace**: `miniwiki`
- **Ingress**: Nginx Ingress Controller
- **Service Mesh**: Istio (optional for production)
- **Horizontal Pod Autoscaler**: HPA for services
- **Cluster Autoscaler**: Cluster Autoscaler for nodes

**Helm Chart Structure**:
```
helm/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── ingress.yaml
│   ├── configmap.yaml
│   └── secret.yaml
└── charts/
    ├── frontend/
    ├── auth-service/
    ├── document-service/
    ├── sync-service/
    └── websocket-service/
```

**values.yaml**:
```yaml
replicaCount: 3

image:
  repository: miniwiki/auth-service
  tag: "1.0.0"
  pullPolicy: Always

service:
  type: ClusterIP
  port: 8080

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 250m
    memory: 256Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

database:
  host: postgres-service
  port: 5432
  name: miniwiki
  username: user
  passwordSecretRef: db-password

redis:
  host: redis-service
  port: 6379
```

---

## 7. Monitoring & Observability

### 7.1 Logging Strategy

**Structured Logging (JSON)**:
```rust
use serde::Serialize;
use tracing::{info, error, instrument};

#[derive(Debug, Serialize)]
pub struct LogContext {
    pub user_id: Option<String>,
    pub request_id: String,
    pub document_id: Option<String>,
}

pub async fn get_document(
    req: HttpRequest,
    path: Path<(String,)>,
) -> Result<HttpResponse> {
    let document_id = path.into_inner();
    let context = LogContext {
        user_id: Some(req.user_id.clone()),
        request_id: uuid::Uuid::new_v4().to_string(),
        document_id: Some(document_id.clone()),
    };

    info!(
        user_id = context.user_id,
        request_id = %context.request_id,
        document_id = %context.document_id,
        "Fetching document"
    );

    match fetch_document(&document_id).await {
        Ok(doc) => {
            info!(
                user_id = context.user_id,
                request_id = %context.request_id,
                "Document fetched successfully"
            );
            Ok(HttpResponse::Ok().json(doc))
        }
        Err(e) => {
            error!(
                    user_id = context.user_id,
                    request_id = %context.request_id,
                    error = %e,
                    "Failed to fetch document"
                );
            Err(Error::NotFound)
        }
    }
}
```

**Log Aggregation**:
- **ELK Stack**: Elasticsearch + Logstash + Kibana (production)
- **Loki**: Lightweight log aggregation (MVP)
- **Grafana**: Visualization

### 7.2 Metrics Strategy

**Prometheus Metrics**:
```rust
use prometheus::{Encoder, TextEncoder};
use prometheus::{Counter, Histogram, Gauge};

lazy_static! {
    static ref REQUEST_COUNT: Counter = register(
        Counter::new("http_requests_total", "Total number of HTTP requests")
            .expect("metrics can be registered")
    ).unwrap();

    static ref REQUEST_DURATION: Histogram = register(
        Histogram::new("http_request_duration_seconds", "HTTP request latencies in seconds")
            .expect("metrics can be registered")
    ).unwrap();
}

pub async fn metrics_handler() -> Result<HttpResponse> {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&mut buffer).unwrap();

    Ok(HttpResponse::Ok()
        .content_type(ContentType(mime::APPLICATION_JSON))
        .body(String::from_utf8(buffer).unwrap()))
}

pub async fn with_metrics<F, R>(
    handler: F,
) -> impl Fn<ServiceRequest, Result<R>> for F
where
    F: Fn(ServiceRequest) -> Result<R>,
{
    move |req: ServiceRequest| -> Pin<Box<dyn Future<Output = Result<R>> + Send>> {
        Box::pin(async move {
            let start = Instant::now();
            REQUEST_COUNT.inc();

            match handler(req).await {
                Ok(response) => {
                    let duration = start.elapsed().as_secs_f64();
                    REQUEST_DURATION.observe(duration);
                    Ok(response)
                }
                Err(e) => Err(e),
            }
        })
    }
}
```

**Key Metrics**:
- **Request Count**: `http_requests_total{endpoint, method, status}`
- **Request Duration**: `http_request_duration_seconds{endpoint, method}`
- **Database Query Duration**: `db_query_duration_seconds{query_type}`
- **CRDT Sync Duration**: `crdt_sync_duration_seconds{document_id}`
- **WebSocket Connections**: `websocket_connections_total`
- **Active Users**: `active_users{space_id}`

### 7.3 Distributed Tracing

**Jaeger Tracing**:
```rust
use opentelemetry::trace::Tracer;
use opentelemetry::trace::Span;

pub async fn get_document(
    tracer: &Tracer,
    document_id: String,
) -> Result<Document> {
    let mut span = tracer.start("get_document");
    span.set_attribute("document.id", &document_id);

    match db_fetch_document(&document_id).await {
        Ok(doc) => {
            span.set_status(StatusCode::Ok, "document fetched");
            span.end();
            Ok(doc)
        }
        Err(e) => {
            span.record_error(&e);
            span.set_status(StatusCode::Error, "database error");
            span.end();
            Err(e)
        }
    }
}
```

**Trace Spans**:
- `http_request`: Entire request lifecycle
- `db_query`: Database queries
- `crdt_merge`: CRDT merge operations
- `file_upload`: File upload to MinIO

---

## 8. Security Architecture

### 8.1 Security Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                    Network Layer                        │
│               - TLS 1.3 (All traffic)                 │
│               - Firewall Rules                           │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                    API Gateway                       │
│               - Rate Limiting                           │
│               - CORS Policy                            │
│               - Request Validation                       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                Authentication Layer                 │
│               - JWT Validation                          │
│               - Session Management                     │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                Authorization Layer                 │
│               - RBAC Enforcement                       │
│               - Permission Checks                      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Data Layer                          │
│               - Encryption at Rest (AES-256)               │
│               - Parameterized Queries                      │
│               - Audit Logging                          │
└─────────────────────────────────────────────────────────────────┘
```

### 8.2 Rate Limiting

**Token Bucket Algorithm**:
```rust
use redis::AsyncCommands;

pub async fn rate_limit(
    redis: &mut redis::Client,
    user_id: &str,
    endpoint: &str,
    limit: usize,
    window: Duration,
) -> Result<bool> {
    let key = format!("rate_limit:{}:{}", user_id, endpoint);

    // Get current count
    let count: usize = redis.get(&key).await.unwrap_or(0);

    if count >= limit {
        return Ok(false); // Rate limited
    }

    // Increment count
    redis.incr(&key).await?;

    // Set expiration
    redis.expire(&key, window.as_secs() as usize).await?;

    Ok(true)
}
```

**Rate Limits**:
- **Anonymous**: 100 requests/hour
- **Authenticated**: 1000 requests/hour
- **Document Updates**: 60 updates/minute per document
- **File Uploads**: 10 uploads/hour

### 8.3 Input Validation

**Validator Middleware**:
```rust
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct CreateDocumentRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    #[validate(custom = "validate_yjs_state")]
    pub yjs_state: String,

    #[validate(custom = "validate_uuid")]
    pub space_id: String,
}

pub async fn create_document(
    req: Json<CreateDocumentRequest>,
) -> Result<HttpResponse> {
    let doc_req = req.into_inner();

    // Validate automatically by serde validator
    if let Err(e) = doc_req.validate() {
        return Err(Error::ValidationError(e.to_string()));
    }

    // Create document...
}
```

---

## 9. Performance Optimization

### 9.1 Caching Strategy

**Multi-Level Caching**:
```
┌─────────────────────────────────────────────────────────┐
│                  Client Cache                        │
│         (In-memory, 5min TTL)                     │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                 Redis Cache                         │
│         (Distributed, 10min TTL)                    │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              PostgreSQL Database                       │
│         (Persistent, single source of truth)           │
└─────────────────────────────────────────────────────────┘
```

**Cache Keys**:
- `document:{id}`: Document metadata
- `document_content:{id}`: Document Yjs state
- `user_permissions:{user_id}:{document_id}`: User permissions
- `space_members:{space_id}`: Space member list

### 9.2 Database Optimization

**Indexes**:
```sql
-- Composite index for document listing
CREATE INDEX idx_documents_space_parent
ON documents(space_id, parent_id, updated_at DESC)
WHERE deleted_at IS NULL;

-- Index for full-text search
CREATE INDEX idx_documents_title_trgm
ON documents USING gin (title gin_trgm_ops);

-- Index for audit log queries
CREATE INDEX idx_audit_logs_user_action
ON audit_logs(user_id, action, created_at DESC);
```

**Query Optimization**:
- **Pagination**: Use `LIMIT` + `OFFSET` for large result sets
- **Cursor Pagination**: Use cursor-based pagination for infinite scroll
- **Connection Pooling**: Max 20 connections per service
- **Prepared Statements**: Always use parameterized queries

### 9.3 CDN Strategy

**Static Assets**:
- **Frontend Assets**: Hosted on CDN (Cloudflare, AWS CloudFront)
- **Uploaded Files**: Served directly from MinIO (CDN integration optional)
- **Versioning**: Cache busting via URL versioning

---

## 10. Scalability Strategy

### 10.1 Horizontal Scaling

**Stateless Services**:
- All backend services are stateless
- Session state stored in Redis
- No sticky sessions required
- Services can be scaled horizontally

**Auto Scaling**:
```yaml
# Kubernetes HPA
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: auth-service-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: auth-service
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

### 10.2 Database Scaling

**Read Replicas**:
```yaml
postgresql:
  primary:
    replicas: 1
    storage: 1TB SSD
  readReplicas:
    replicas: 3
    storage: 1TB SSD
  loadBalancing: pgpool-II
```

**Partitioning Strategy**:
- **Shard by Space ID**: Documents partitioned by space
- **Hot Spaces**: Frequently accessed spaces on separate shards
- **Cold Spaces**: Less active spaces on shared shards

### 10.3 File Storage Scaling

**MinIO Distributed Mode**:
```yaml
minio:
  mode: distributed
  servers:
    - address: minio-1:9000
    - address: minio-2:9000
    - address: minio-3:9000
    - address: minio-4:9000
  erasureCoding:
    k: 4
    m: 2
```

---

## 11. Disaster Recovery

### 11.1 Backup Strategy

**Automated Backups**:
- **Database**: Daily full backup, hourly incremental backups
- **Redis**: RDB snapshots every 6 hours
- **MinIO**: Versioning enabled, weekly snapshots
- **Retention**: 30 days (configurable)

**Backup Script**:
```bash
#!/bin/bash

# Database backup
pg_dump -h postgres -U user miniwiki | gzip > /backups/postgres-$(date +%Y%m%d).sql.gz

# Redis backup
redis-cli --rdb /backups/redis-$(date +%Y%m%d).rdb

# Upload to AWS S3 (if configured)
aws s3 cp /backups/postgres-$(date +%Y%m%d).sql.gz s3://miniwiki-backups/
aws s3 cp /backups/redis-$(date +%Y%m%d).rdb s3://miniwiki-backups/
```

### 11.2 Disaster Recovery Plan

**RTO (Recovery Time Objective)**: 4 hours
**RPO (Recovery Point Objective)**: 1 hour

**Recovery Steps**:
1. **Assess Impact**: Determine affected services and data
2. **Restore from Backup**: Restore most recent backup
3. **Verify Data**: Validate data integrity
4. **Test Services**: Run smoke tests
5. **Switch Traffic**: Update DNS/load balancer
6. **Monitor**: Monitor for issues post-recovery

---

## 12. Technology Rationale

### 12.1 Why Flutter?

**Decision**: Use Flutter for all platforms (Web, Desktop, Mobile)

**Rationale**:
- **Single Codebase**: 95% code shared across all platforms
- **Native Performance**: 60-120 FPS on mobile, smooth animations
- **Hot Reload**: Development speed, instant feedback
- **Rich Ecosystem**: Packages for all requirements (Isar, Dio, Riverpod)
- **Future-Proof**: Google's continued investment, growing community

**Alternatives Considered**:
- **React Native**: Web bundle overhead, inconsistent performance
- **Native Apps**: Separate codebases, 3x development cost
- **Electron**: High memory usage, poor mobile support

### 12.2 Why Rust?

**Decision**: Use Rust for backend services

**Rationale**:
- **Performance**: Zero-cost abstractions, memory safety, no garbage collector
- **Concurrency**: Async/await, safe multi-threading
- **Type Safety**: Prevent entire classes of bugs at compile time
- **Ecosystem**: Actix-web (high-performance web framework), sqlx (type-safe SQL)

**Alternatives Considered**:
- **Go**: Excellent, but Rust's type safety preferred
- **Node.js**: Good for sync, but performance concerns for API
- **Python**: Too slow for high-throughput APIs

### 12.3 Why Yjs CRDT?

**Decision**: Use Yjs for real-time sync and collaboration

**Rationale**:
- **Automatic Conflict Resolution**: No manual merge needed
- **Offline-First**: Works seamlessly with offline editing
- **Real-Time**: Sub-millisecond sync latency
- **Proven**: Used by Notion, BlockSuite, and other production apps

**Alternatives Considered**:
- **Operational Transformation**: Complex, error-prone
- **Last-Write-Wins**: Data loss in concurrent edits
- **Manual Merge**: Poor user experience, blocking

### 12.4 Why PostgreSQL?

**Decision**: Use PostgreSQL as primary database

**Rationale**:
- **ACID Compliance**: Strong consistency for documents
- **JSONB**: Efficient storage for Yjs CRDT states
- **Full-Text Search**: Built-in text search with extensions
- **Mature**: Battle-tested, excellent tooling, strong community

**Alternatives Considered**:
- **MongoDB**: Flexible but no ACID, weaker consistency
- **MySQL**: Good, but PostgreSQL's JSONB superior for document storage

### 12.5 Why MinIO?

**Decision**: Use MinIO for object storage

**Rationale**:
- **S3-Compatible**: Uses standard S3 SDK, easy migration
- **Self-Hosted**: Complete control, no cloud dependency
- **High Performance**: Faster than AWS S3 for same hardware
- **Cost**: No storage egress fees

**Alternatives Considered**:
- **AWS S3**: Excellent but not self-hosted, vendor lock-in
- **Local Filesystem**: No versioning, poor scalability

---

**Document Version**: 1.0
**Last Updated**: January 11, 2026
**Maintainer**: Engineering Team
