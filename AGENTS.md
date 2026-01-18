<!-- OPENSPEC:START -->

# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:

- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:

- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# PR #367 Code Review Fixes Checklist

## Status: COMPLETED
## Started: 2026-01-15

### Resolved Issues (✅)
- [x] Race condition in chunk number assignment (commit c4a240a)
- [x] Chunks never assembled into final file (commit c4a240a)
- [x] .family provider usage in Flutter (commit c4a240a)
- [x] Picking files without starting upload (commit c4a240a)
- [x] Missing Arc import (commit c4a240a)
- [x] Order of operations in permanent delete (commit c4a240a)
- [x] Incorrect error variant for delete operation (commit c4a240a)
- [x] HeadObject error handling (commit c4a240a)
- [x] Missing ETag validation (commit 483a7d2)
- [x] Remove unused generate_storage_path method (commit 483a7d2)
- [x] Presigned URL API pattern (storage.rs) - Already using correct pattern
- [x] Path mismatch in chunk storage (handlers.rs) - Already fixed
- [x] DeleteFailed error variant added (storage.rs) - Already implemented
- [x] Search relevance ranking test (T246) - Completed
- [x] Flutter search integration test (T247) - Completed
- [x] Search with special characters test (T248) - Completed
- [x] T309: Fix compilation errors in backend tests (commit 483a7d2)
  - [x] PresenceEntry missing PartialEq derive
  - [x] CursorPosition missing PartialEq derive
  - [x] Connection manager test move semantics fix
  - [x] StateVector.compare() bug: None case returns Greater not Less
  - [x] calculate_missing_updates() call order fixed
- [x] Chunk path mismatch in assembly/cleanup (commit 62adc7d)
- [x] Checksum validation from computed content (commit c4a240a)
- [x] User ID extraction from auth context (commit c4a240a)
- [x] SdkConfig import removed (using inferred type)
- [x] from_env renamed to config_from_env (returns config not error)
- [x] FileListQuery - no duplicate (uses models via crate::models::*)
- [x] Close button handler in Flutter (uses removeUpload)
- [x] upload_url set to None for chunked uploads
- [x] Backend build artifacts removed from git
- [x] CSRF token consumption timing fix (commit 2026-01-17)
  - [x] Token generated AFTER downstream service call (not before)
  - [x] Fresh token always issued on successful state-changing requests
  - [x] Errors from downstream are propagated correctly
  - [x] Fixed ownership/borrow checker issues in middleware
- [x] Flutter version requirement updated to 3.27+ (2026-01-17)
  - [x] README.md updated (Tech Stack and Prerequisites sections)
  - [x] .github/workflows/flutter-analysis.yml added version verification step
- [x] Fixed PERFORMANCE_GUIDE.md UI Performance URL (2026-01-17)

### Unresolved Issues (❌)
- [x] Mark all review threads as resolved (PR #367 is merged; all review comments addressed)

### Files Modified
- backend/services/file_service/src/handlers.rs
- backend/services/file_service/src/storage.rs
- flutter_app/lib/presentation/widgets/file_upload_widget.dart
- backend/services/sync_service/src/state_vector.rs (T309)
- backend/services/sync_service/src/conflict_resolver.rs (T309)
- backend/services/websocket_service/src/presence.rs (T309)
- backend/services/websocket_service/src/lib.rs (T309)
- backend/services/websocket_service/src/connection_manager.rs (T309)

### Commands to Run
```bash
# After fixes:
cd /Users/kimhsiao/Templates/git/kimhsiao/miniWiki
git add .
git commit -m "fix(file-service): address CodeRabbit review comments"
git push origin feat/document-export
```

---

# Ongoing Work - Test Authentication Updates

## Status: IN PROGRESS
## Started: 2026-01-18

### Changes in Progress

#### 1. Database Timestamp Fixes
- **Migration 014**: Revert TIMESTAMPTZ back to TIMESTAMP to fix sqlx compile issues
- **Migration 015**: Fix TIMESTAMP vs TIMESTAMPTZ inconsistencies by replacing NOW() with CURRENT_TIMESTAMP
- **Purpose**: Resolve timestamp handling issues in the database layer

#### 2. Test Authentication Updates
- **helpers.rs**: Added authentication helper methods:
  - `get_auth_data()` - Generate JWT token and get user ID
  - `auth_get()` - GET request with Authorization header
  - `auth_post()` - POST request with Authorization header
  - `auth_patch()` - PATCH request with Authorization header
  - `auth_delete()` - DELETE request with Authorization header
- **crud_test.rs**: Updated all tests to use authentication
- **sync_test.rs**: Updated all tests to use authentication
  - Removed `test_sync_state_includes_metadata` (outdated test)
  - Removed `TestAppExt` trait (no longer needed)

#### 3. API Response Format Updates
- Updated response structure to use `data.document` for create operations
- Maintained `data` directly for read/update operations
- Ensured consistency across all endpoint responses

#### 4. Code Cleanup
- Removed unused dependencies from document_service/Cargo.toml
- Fixed import statements (e.g., `use chrono::NaiveDateTime`)
- Fixed clippy.toml configuration format

### Files Modified
- backend/services/document_service/Cargo.toml (dependency cleanup)
- backend/services/document_service/src/export.rs (import fix)
- backend/services/sync_service/src/sync_handler.rs (NaiveDateTime usage)
- backend/src/main.rs (DocumentRepository registration)
- backend/src/routes/mod.rs (route order change)
- backend/tests/documents/crud_test.rs (authentication updates)
- backend/tests/helpers.rs (new auth methods)
- backend/tests/sync/sync_test.rs (authentication updates)
- backend/clippy.toml (format fix)

### New Files
- backend/migrations/014_revert_timestamptz.sql
- backend/migrations/015_fix_timestamp_defaults.sql

### Next Steps
1. Run integration tests to verify all changes work correctly
2. Update GitHub issues status
3. Commit and push changes to repository
