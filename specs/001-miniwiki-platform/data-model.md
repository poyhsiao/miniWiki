# Data Model: miniWiki Platform

**Feature**: miniWiki Knowledge Management Platform  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)  
**Date**: 2026-01-11

## Overview

This document defines the complete data model for the miniWiki platform, including entity relationships, field definitions, validation rules, and indexes. The model supports offline-first architecture with CRDT synchronization.

## Entity Relationship Diagram

```
┌─────────────┐       ┌─────────────────────┐       ┌─────────────┐
│    User     │───────│  SpaceMembership    │───────│    Space    │
└─────────────┘       └─────────────────────┘       └─────────────┘
      │                        │                           │
      │                        │                           │
      │                        │                    ┌──────┴──────┐
      │                        │                    │             │
      │              ┌─────────┴─────────┐   ┌─────┴─────┐ ┌─────┴─────┐
      │              │                   │   │ Document  │ │   File    │
      │              │   SpaceDocument   │   └─────┬─────┘ └───────────┘
      │              │                   │         │
      └──────────────│                   │         │
                     └───────────────────┘         │
                              │                   │
                              │          ┌────────┴────────┐
                              │          │                 │
                              │    ┌─────┴─────┐     ┌─────┴─────┐
                              │    │Document   │     │  Comment  │
                              │    │Version    │     └───────────┘
                              │    └───────────┘
                              │          │
                              │          │ ┌───────────────┐
                              └──────────┼─│  AuditLog     │
                                         │ └───────────────┘
                                         │
                                  ┌──────┴──────┐
                                  │ SyncSession │
                                  └─────────────┘
```

## Core Entities

### User

Represents a registered user of the platform.

**Table**: `users`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `email` | VARCHAR(255) | UNIQUE, NOT NULL, INDEX | User email (lowercase) |
| `password_hash` | VARCHAR(255) | NOT NULL | bcrypt hash |
| `display_name` | VARCHAR(100) | NOT NULL | Display name |
| `avatar_url` | VARCHAR(512) | NULL | Avatar image URL |
| `timezone` | VARCHAR(50) | NOT NULL DEFAULT 'UTC' | User timezone |
| `language` | VARCHAR(10) | NOT NULL DEFAULT 'en' | Language preference |
| `is_active` | BOOLEAN | NOT NULL DEFAULT true | Account active status |
| `is_email_verified` | BOOLEAN | NOT NULL DEFAULT false | Email verified |
| `email_verified_at` | TIMESTAMP | NULL | When email was verified |
| `last_login_at` | TIMESTAMP | NULL | Last login timestamp |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Creation timestamp |
| `updated_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Last update timestamp |

**Indexes**:
- `idx_users_email` ON (`email`)
- `idx_users_active` ON (`is_active`)

**Validation Rules**:
- `email`: RFC 5322 compliant, max 255 chars, lowercase stored
- `password_hash`: bcrypt cost 12, never store plain text
- `display_name`: 1-100 chars, unicode supported

---

### Space

A collection of documents (analogous to Notion pages or Confluence spaces).

**Table**: `spaces`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `owner_id` | UUID | NOT NULL, REFERENCES users(id) | Space owner |
| `name` | VARCHAR(200) | NOT NULL, INDEX | Space name |
| `icon` | VARCHAR(50) | NULL | Space icon (emoji or icon name) |
| `description` | TEXT | NULL | Space description |
| `is_public` | BOOLEAN | NOT NULL DEFAULT false | Public visibility |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Creation timestamp |
| `updated_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Last update timestamp |

**Indexes**:
- `idx_spaces_owner` ON (`owner_id`)
- `idx_spaces_public` ON (`is_public`)

**Validation Rules**:
- `name`: 1-200 chars, required
- `is_public`: When true, space is visible to anyone with link

---

### SpaceMembership

Junction table linking users to spaces with roles.

**Table**: `space_memberships`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `space_id` | UUID | NOT NULL, REFERENCES spaces(id), INDEX | Related space |
| `user_id` | UUID | NOT NULL, REFERENCES users(id), INDEX | Member user |
| `role` | VARCHAR(20) | NOT NULL, CHECK IN ('owner', 'editor', 'commenter', 'viewer') | Member role |
| `joined_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | When joined |
| `invited_by` | UUID | NOT NULL, REFERENCES users(id) | Who invited |

**Constraints**:
- UNIQUE (`space_id`, `user_id`) - One role per user per space
- `owner` role cannot be removed or changed

**Role Permissions Matrix**:

| Permission | Owner | Editor | Commenter | Viewer |
|------------|-------|--------|-----------|--------|
| View documents | ✅ | ✅ | ✅ | ✅ |
| Create documents | ✅ | ✅ | ❌ | ❌ |
| Edit documents | ✅ | ✅ | ✅ (own) | ❌ |
| Delete documents | ✅ | ✅ | ❌ | ❌ |
| Manage members | ✅ | ❌ | ❌ | ❌ |
| Space settings | ✅ | ❌ | ❌ | ❌ |
| Add comments | ✅ | ✅ | ✅ | ❌ |
| Resolve comments | ✅ | ✅ | ✅ (own) | ❌ |

---

### Document

A single document/page within a space.

**Table**: `documents`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `space_id` | UUID | NOT NULL, REFERENCES spaces(id), INDEX | Parent space |
| `parent_id` | UUID | NULL, REFERENCES documents(id) | Parent document (hierarchy) |
| `title` | VARCHAR(200) | NOT NULL | Document title |
| `icon` | VARCHAR(50) | NULL | Document icon |
| `content` | JSONB | NOT NULL DEFAULT '{}' | Yjs CRDT document state |
| `content_size` | INTEGER | NOT NULL DEFAULT 0 | Size in bytes (max 10MB) |
| `is_archived` | BOOLEAN | NOT NULL DEFAULT false | Soft deleted |
| `archived_at` | TIMESTAMP | NULL | When archived |
| `created_by` | UUID | NOT NULL, REFERENCES users(id) | Original author |
| `last_edited_by` | UUID | NOT NULL, REFERENCES users(id) | Last editor |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Creation timestamp |
| `updated_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Last update timestamp |

**Indexes**:
- `idx_documents_space` ON (`space_id`)
- `idx_documents_parent` ON (`parent_id`)
- `idx_documents_archived` ON (`is_archived`)
- `idx_documents_updated` ON (`updated_at` DESC)

**Constraints**:
- CHECK (`content_size` <= 10485760) -- 10MB max
- CHECK (`title` <> '') -- Non-empty title

**Validation Rules**:
- `title`: 1-200 chars, required
- `content_size`: Calculated from JSONB, enforced at 10MB
- `is_archived`: Soft delete for recovery

**Yjs Document State**:
```json
{
  "type": "Y.Doc",
  "update": "<base64-encoded Uint8Array>",
  "vector_clock": {
    "client_id": "uuid",
    "clock": 123
  }
}
```

---

### DocumentVersion

Historical snapshots of document content for version history.

**Table**: `document_versions`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `document_id` | UUID | NOT NULL, REFERENCES documents(id), INDEX | Related document |
| `version_number` | INTEGER | NOT NULL | Sequential version |
| `content` | JSONB | NOT NULL | Yjs state at this version |
| `title` | VARCHAR(200) | NOT NULL | Title at this version |
| `created_by` | UUID | NOT NULL, REFERENCES users(id) | Who created this version |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | When version was created |
| `change_summary` | VARCHAR(500) | NULL | Optional description |

**Indexes**:
- `idx_versions_document` ON (`document_id`, `version_number` DESC)

**Constraints**:
- UNIQUE (`document_id`, `version_number`)
- Automatic version number increment

**Version Retention**:
- Keep all versions for 30 days
- After 30 days, keep only major versions (every 10th)
- Versions > 1 year: keep only monthly snapshots

---

### File

File attachments uploaded to documents.

**Table**: `files`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `space_id` | UUID | NOT NULL, REFERENCES spaces(id), INDEX | Parent space |
| `document_id` | UUID | NULL, REFERENCES documents(id), INDEX | Attached document |
| `uploaded_by` | UUID | NOT NULL, REFERENCES users(id) | Uploader |
| `file_name` | VARCHAR(255) | NOT NULL | Original file name |
| `file_type` | VARCHAR(100) | NOT NULL | MIME type |
| `file_size` | BIGINT | NOT NULL | Size in bytes (max 50MB) |
| `storage_path` | VARCHAR(512) | NOT NULL | MinIO object path |
| `storage_bucket` | VARCHAR(50) | NOT NULL DEFAULT 'files' | MinIO bucket |
| `checksum` | VARCHAR(64) | NOT NULL | SHA-256 hash |
| `is_deleted` | BOOLEAN | NOT NULL DEFAULT false | Soft deleted |
| `deleted_at` | TIMESTAMP | NULL | When deleted |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Upload timestamp |

**Indexes**:
- `idx_files_space` ON (`space_id`)
- `idx_files_document` ON (`document_id`)
- `idx_files_uploader` ON (`uploaded_by`)
- `idx_files_checksum` ON (`checksum`)

**Constraints**:
- CHECK (`file_size` <= 52428800) -- 50MB max
- CHECK (`file_type` NOT IN ('application/x-msdownload', 'application/x-executable'))

**Storage Path Format**:
```
{space_id}/{file_id}/{YYYY-MM-DD}/{file_name}
```

---

### Comment

Comments on documents.

**Table**: `comments`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `document_id` | UUID | NOT NULL, REFERENCES documents(id), INDEX | Related document |
| `parent_id` | UUID | NULL, REFERENCES comments(id) | Parent comment (thread) |
| `author_id` | UUID | NOT NULL, REFERENCES users(id) | Comment author |
| `content` | TEXT | NOT NULL | Comment text (max 5000 chars) |
| `is_resolved` | BOOLEAN | NOT NULL DEFAULT false | Resolved status |
| `resolved_by` | UUID | NULL, REFERENCES users(id) | Who resolved |
| `resolved_at` | TIMESTAMP | NULL | When resolved |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Creation timestamp |
| `updated_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Last update timestamp |

**Indexes**:
- `idx_comments_document` ON (`document_id`)
- `idx_comments_parent` ON (`parent_id`)
- `idx_comments_author` ON (`author_id`)
- `idx_comments_resolved` ON (`is_resolved`)

**Constraints**:
- CHECK (`content` <> '') -- Non-empty
- CHECK (`length(content)` <= 5000)

**Resolution Rules**:
- Editor+ or comment author can resolve
- Resolution persists author and timestamp

---

### AuditLog

Immutable audit trail for security and compliance.

**Table**: `audit_logs`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `user_id` | UUID | NULL, REFERENCES users(id) | Acting user (nullable for system) |
| `action` | VARCHAR(50) | NOT NULL, INDEX | Action type |
| `resource_type` | VARCHAR(50) | NOT NULL | Resource type |
| `resource_id` | UUID | NOT NULL | Resource identifier |
| `details` | JSONB | NULL | Additional details |
| `ip_address` | INET | NULL | Client IP |
| `user_agent` | VARCHAR(500) | NULL | Client user agent |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | When action occurred |

**Indexes**:
- `idx_audit_user` ON (`user_id`)
- `idx_audit_action` ON (`action`)
- `idx_audit_resource` ON (`resource_type`, `resource_id`)
- `idx_audit_created` ON (`created_at` DESC)

**Actions Logged**:
- `user.login`, `user.logout`, `user.register`
- `document.create`, `document.update`, `document.delete`, `document.archive`
- `document.share`, `document.unshare`
- `comment.create`, `comment.update`, `comment.delete`, `comment.resolve`
- `file.upload`, `file.delete`
- `space.create`, `space.update`, `space.delete`
- `membership.add`, `membership.update`, `membership.remove`
- `password.reset_request`, `password.reset_complete`

**Retention**: 2 years, then archive to cold storage

---

### SyncSession

Tracks active sync sessions for presence and CRDT updates.

**Table**: `sync_sessions`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `document_id` | UUID | NOT NULL, REFERENCES documents(id), INDEX | Synced document |
| `user_id` | UUID | NOT NULL, REFERENCES users(id) | Syncing user |
| `client_id` | VARCHAR(36) | NOT NULL | Yjs client ID |
| `status` | VARCHAR(20) | NOT NULL, CHECK IN ('active', 'idle', 'disconnected') | Session status |
| `last_sync_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Last sync timestamp |
| `expires_at` | TIMESTAMP | NOT NULL | Session expiry (5min TTL) |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Session creation |

**Indexes**:
- `idx_sync_document` ON (`document_id`)
- `idx_sync_user` ON (`user_id`)
- `idx_sync_expires` ON (`expires_at`)

**Constraints**:
- TTL index on `expires_at` for automatic cleanup
- Automatic cleanup of disconnected sessions

**Presence Information**:
```json
{
  "user": {
    "id": "uuid",
    "display_name": "John Doe",
    "avatar_url": "https://..."
  },
  "cursor": {
    "position": 123,
    "selection": {"anchor": 100, "head": 150}
  },
  "status": "active"
}
```

---

### ShareLink

Public share links for external document access.

**Table**: `share_links`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `document_id` | UUID | NOT NULL, REFERENCES documents(id), INDEX | Shared document |
| `created_by` | UUID | NOT NULL, REFERENCES users(id) | Link creator |
| `token` | VARCHAR(64) | NOT NULL, UNIQUE, INDEX | Share token |
| `access_code` | VARCHAR(10) | NULL | Optional password |
| `expires_at` | TIMESTAMP | NULL | Optional expiry |
| `permission` | VARCHAR(20) | NOT NULL DEFAULT 'view', CHECK IN ('view', 'comment') | Access level |
| `access_count` | INTEGER | NOT NULL DEFAULT 0 | Access counter |
| `max_access` | INTEGER | NULL | Max access limit |
| `is_active` | BOOLEAN | NOT NULL DEFAULT true | Link active status |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Link creation |

**Indexes**:
- `idx_share_token` ON (`token`)
- `idx_share_document` ON (`document_id`)

**Constraints**:
- CHECK (`access_code` IS NULL OR `length(access_code)` >= 4)
- One token per link (URL-safe base64)

**URL Format**:
```
https://miniwiki.example.com/share/{token}
```

---

### PasswordReset

Password reset tokens for account recovery.

**Table**: `password_resets`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `user_id` | UUID | NOT NULL, REFERENCES users(id), INDEX | User requesting reset |
| `token` | VARCHAR(64) | NOT NULL, UNIQUE, INDEX | Reset token |
| `expires_at` | TIMESTAMP | NOT NULL | Token expiry (1 hour) |
| `used_at` | TIMESTAMP | NULL | When token was used |
| `ip_address` | INET | NULL | Request IP |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Request timestamp |

**Indexes**:
- `idx_reset_token` ON (`token`)
- `idx_reset_user` ON (`user_id`)
- `idx_reset_expires` ON (`expires_at`)

**Constraints**:
- Automatic cleanup after 1 hour
- Token used once only

---

### RefreshToken

JWT refresh tokens for session management.

**Table**: `refresh_tokens`

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY | Unique identifier |
| `user_id` | UUID | NOT NULL, REFERENCES users(id), INDEX | Associated user |
| `token` | VARCHAR(64) | NOT NULL, UNIQUE, INDEX | Refresh token |
| `expires_at` | TIMESTAMP | NOT NULL | Token expiry (7 days) |
| `ip_address` | INET | NULL | Created from IP |
| `user_agent` | VARCHAR(500) | NULL | Created from UA |
| `is_revoked` | BOOLEAN | NOT NULL DEFAULT false | Revoked status |
| `revoked_at` | TIMESTAMP | NULL | When revoked |
| `created_at` | TIMESTAMP | NOT NULL DEFAULT NOW() | Token creation |

**Indexes**:
- `idx_refresh_token` ON (`token`)
- `idx_refresh_user` ON (`user_id`)
- `idx_refresh_expires` ON (`expires_at`)

**Constraints**:
- One token per device/session
- Automatic cleanup on revocation or expiry

---

## Offline Storage Schema (Isar)

Flutter client stores local copy for offline access.

### Isar Collections

```dart
// User collection
@Collection()
class UserEntity {
  Id? id;
  String get uuid => id!.toString();
  String email = '';
  String displayName = '';
  String? avatarUrl;
  String timezone = 'UTC';
  String language = 'en';
}

// Space collection
@Collection()
class SpaceEntity {
  Id? id;
  String get uuid => id!.toString();
  String name = '';
  String? icon;
  String? description;
  bool isPublic = false;
  String ownerId = '';
  DateTime? createdAt;
}

// Document collection
@Collection()
class DocumentEntity {
  Id? id;
  String get uuid => id!.toString();
  String spaceId = '';
  String? parentId;
  String title = '';
  String? icon;
  Map<String, dynamic> content = {};
  int contentSize = 0;
  bool isArchived = false;
  String createdBy = '';
  String lastEditedBy = '';
  DateTime? createdAt;
  DateTime? updatedAt;
  bool isSynced = true;
  bool isDirty = false;
  DateTime? lastSyncedAt;
}

// Comment collection
@Collection()
class CommentEntity {
  Id? id;
  String get uuid => id!.toString();
  String documentId = '';
  String? parentId;
  String authorId = '';
  String authorName = '';
  String? authorAvatar;
  String content = '';
  bool isResolved = false;
  DateTime? createdAt;
  DateTime? updatedAt;
  bool isSynced = true;
}

// File collection
@Collection()
class FileEntity {
  Id? id;
  String get uuid => id!.toString();
  String spaceId = '';
  String? documentId;
  String fileName = '';
  String fileType = '';
  int fileSize = 0;
  String storagePath = '';
  String checksum = '';
  bool isSynced = true;
  DateTime? createdAt;
}

// Sync queue for pending operations
@Collection()
class SyncQueueItem {
  Id? id;
  String get uuid => id!.toString();
  String entityType = ''; // 'document', 'comment', 'file'
  String entityId = '';
  String operation = ''; // 'create', 'update', 'delete'
  Map<String, dynamic> data = {};
  int retryCount = 0;
  DateTime? nextRetryAt;
  DateTime createdAt = DateTime.now();
}
```

### Isar Indexes

```dart
// Document indexes
@Name('idx_documents_space')
@Index()
final spaceIdIndex = Index('spaceId');

// Space indexes
@Name('idx_spaces_owner')
@Index()
final ownerIdIndex = Index('ownerId');

// Sync queue indexes
@Name('idx_sync_pending')
@Index()
final pendingSyncIndex = Index('nextRetryAt', type: IndexType.value);
```

---

## Database Migrations

All migrations versioned in `backend/migrations/`.

### Migration Naming

```
{version}_{description}.sql
```

### Example Migrations

**001_initial_schema.sql**:
```sql
-- Create users table
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  display_name VARCHAR(100) NOT NULL,
  avatar_url VARCHAR(512),
  timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
  language VARCHAR(10) NOT NULL DEFAULT 'en',
  is_active BOOLEAN NOT NULL DEFAULT true,
  is_email_verified BOOLEAN NOT NULL DEFAULT false,
  email_verified_at TIMESTAMP,
  last_login_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create spaces table
CREATE TABLE spaces (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_id UUID NOT NULL REFERENCES users(id),
  name VARCHAR(200) NOT NULL,
  icon VARCHAR(50),
  description TEXT,
  is_public BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- ... additional tables
```

### Rollback Support

Each migration includes corresponding rollback:

```sql
-- Down migration
DROP TABLE IF EXISTS users;
```

---

## Validation Rules Summary

| Entity | Field | Rule | Error Code |
|--------|-------|------|------------|
| User | email | RFC 5322, max 255 | `VALIDATION_EMAIL_INVALID` |
| User | password | 8+ chars, 1 uppercase, 1 number | `VALIDATION_PASSWORD_WEAK` |
| User | display_name | 1-100 chars | `VALIDATION_DISPLAY_NAME_INVALID` |
| Space | name | 1-200 chars | `VALIDATION_SPACE_NAME_INVALID` |
| Document | title | 1-200 chars, required | `VALIDATION_DOCUMENT_TITLE_INVALID` |
| Document | content | Max 10MB | `VALIDATION_DOCUMENT_CONTENT_TOO_LARGE` |
| File | file_size | Max 50MB | `VALIDATION_FILE_TOO_LARGE` |
| File | file_type | Whitelist only | `VALIDATION_FILE_TYPE_BLOCKED` |
| Comment | content | 1-5000 chars | `VALIDATION_COMMENT_INVALID` |
| ShareLink | access_code | 4-10 chars if present | `VALIDATION_ACCESS_CODE_INVALID` |

---

## Seed Data

Initial admin user for system setup:

```sql
-- Admin user (password: change-in-production!)
INSERT INTO users (
  email,
  password_hash,
  display_name,
  is_email_verified
) VALUES (
  'admin@miniwiki.local',
  '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4aYJGYxMnC6C5.Oy', -- 'admin123'
  'Admin',
  true
);
```

---

## Related Documents

- [Architecture](architecture.md) - System architecture
- [spec.md](spec.md) - Feature specification
- [quickstart.md](quickstart.md) - Development setup
- [contracts/](contracts/) - API specifications
