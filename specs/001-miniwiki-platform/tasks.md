---

description: "Task list for miniWiki Knowledge Management Platform implementation"
---

# Tasks: miniWiki Knowledge Management Platform

**Feature Branch**: `001-miniwiki-platform`  
**Input**: Design documents from `/specs/001-miniwiki-platform/`  
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Data Model**: [data-model.md](data-model.md) | **Contracts**: [contracts/](contracts/) | **Quickstart**: [quickstart.md](quickstart.md)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on `plan.md` structure:
- **Backend**: `backend/src/`, `backend/services/`, `backend/migrations/`
- **Frontend**: `flutter_app/lib/`, `flutter_app/test/`
- **Shared**: `backend/shared/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Initialize Rust workspace in backend/ with Cargo.toml and members
- [x] T002 Create backend/services/ structure: auth_service/, document_service/, sync_service/, file_service/, websocket_service/
- [x] T003 Create flutter_app/ structure per plan.md (main.dart, core/, domain/, data/, presentation/, services/)
- [x] T004 Configure backend/Cargo.toml with actix-web, sqlx, tokio, serde dependencies
- [x] T005 [P] Configure rustfmt and clippy for backend code quality
- [x] T006 [P] Configure flutter_app/pubspec.yaml with riverpod, dio, isar, y_crdt, flutter_quill dependencies
- [x] T007 [P] Configure flutter_lints and analysis_options.yaml for Dart code quality
- [x] T008 Create docker-compose.yml with postgres:14, redis:6, minio:latest services
- [x] T009 Create backend/migrations/001_initial_schema.sql with all tables from data-model.md
- [x] T010 Create .env.example with all required environment variables per quickstart.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Shared Infrastructure

- [x] T011 Create backend/shared/errors/error_types.rs with custom error types
- [x] T012 Create backend/shared/errors/mod.rs with error handling macros
- [x] T013 [P] Create backend/shared/database/db.rs with sqlx connection pool
- [x] T014 [P] Create backend/shared/database/migrations.rs with migration runner
- [x] T015 Create backend/shared/models/mod.rs with shared model types
- [x] T016 [P] Create backend/shared/config/mod.rs with environment configuration loading

### Authentication Framework

- [x] T017 Create backend/services/auth_service/src/lib.rs with service structure
- [x] T018 Create backend/services/auth_service/src/handlers.rs with auth HTTP handlers
- [x] T019 Create backend/services/auth_service/src/jwt.rs with JWT token generation/validation
- [x] T020 Create backend/services/auth_service/src/password.rs with bcrypt password hashing
- [x] T021 Create backend/services/auth_service/src/models.rs with auth-related models
- [x] T022 [P] Create backend/services/auth_service/src/repository.rs with database operations
- [x] T023 Create flutter_app/lib/core/config/ with environment configuration providers
- [x] T024 Create flutter_app/lib/services/auth_service.dart with authentication logic
- [x] T025 [P] Create flutter_app/lib/domain/repositories/auth_repository.dart interface
- [x] T026 Create flutter_app/lib/data/repositories/auth_repository_impl.dart with API implementation

### Database Schema

- [x] T027 Create backend/migrations/001_initial_schema.sql with all tables from data-model.md
- [x] T028 Create backend/services/document_service/migrations/ for document-related tables
- [x] T029 Create flutter_app/lib/data/datasources/isar_datasource.dart for offline storage
- [x] T030 [P] Create flutter_app/lib/data/models/ with Isar entity definitions

**Database Schema Status**: Migrations created - requires running when Docker PostgreSQL is available

### API Foundation

- [x] T031 Create backend/src/main.rs with Actix-web application factory
- [x] T032 Create backend/src/routes/mod.rs with API route structure
- [x] T033 Create backend/src/middleware/auth_middleware.rs with JWT verification
- [x] T034 Create backend/src/middleware/error_handler.rs with consistent error responses
- [x] T035 Create flutter_app/lib/core/network/api_client.dart with Dio configuration
- [x] T036 [P] Create flutter_app/lib/core/network/network_error.dart with error handling

### CRDT Foundation

- [x] T037 Create backend/services/sync_service/src/lib.rs with sync service structure
- [x] T038 Create backend/services/sync_service/src/yjs_handler.rs with Yjs/Dart CRDT document handling
- [x] T039 Create backend/services/sync_service/src/state_vector.rs with state vector operations
- [x] T040 Create flutter_app/lib/services/crdt_service.dart with y_crdt integration
- [x] T041 Create flutter_app/lib/services/sync_service.dart with sync orchestration

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 6 - User Authentication (Priority: P1) 🎯 MVP

**Goal**: Users can register accounts, log in securely, and recover passwords

**Independent Test**: Register a new account, login with valid credentials, request password reset, verify email verification flow

**Constitution**: TDD mandatory - tests must be written and FAIL before implementation

### Tests for User Story 6

- [ ] T042 [P] [US6] Create backend/tests/auth/register_test.rs for registration endpoint
- [ ] T043 [P] [US6] Create backend/tests/auth/login_test.rs for login endpoint
- [ ] T044 [P] [US6] Create backend/tests/auth/password_reset_test.rs for password reset
- [ ] T045 [P] [US6] Create flutter_app/test/auth_service_test.dart for auth service unit tests
- [ ] T046 [P] [US6] Create flutter_app/test/auth_repository_test.dart for repository integration tests

### Backend Implementation for User Story 6

- [ ] T047 [US6] Create users table migration in backend/migrations/002_users.sql
- [ ] T048 [US6] Create backend/services/auth_service/src/register.rs with registration handler
- [ ] T049 [US6] Create backend/services/auth_service/src/login.rs with login handler
- [ ] T050 [US6] Create backend/services/auth_service/src/logout.rs with logout handler
- [ ] T051 [US6] Create backend/services/auth_service/src/password_reset.rs with password reset handlers
- [ ] T052 [US6] Create backend/services/auth_service/src/email_verification.rs with email verification
- [ ] T053 [US6] Add refresh token endpoints in backend/services/auth_service/src/refresh.rs
- [ ] T054 [US6] Add session management endpoints in backend/services/auth_service/src/sessions.rs
- [ ] T055 [US6] Add rate limiting middleware for auth endpoints

### Frontend Implementation for User Story 6

- [ ] T056 [US6] Create flutter_app/lib/presentation/pages/auth/login_page.dart
- [ ] T057 [US6] Create flutter_app/lib/presentation/pages/auth/register_page.dart
- [ ] T058 [US6] Create flutter_app/lib/presentation/pages/auth/password_reset_page.dart
- [ ] T059 [US6] Create flutter_app/lib/presentation/providers/auth_provider.dart with Riverpod state
- [ ] T060 [US6] Create flutter_app/lib/presentation/dialogs/email_verification_dialog.dart
- [ ] T061 [US6] Implement login form validation and submission in login_page.dart
- [ ] T062 [US6] Implement registration form validation and submission in register_page.dart
- [ ] T063 [US6] Connect auth_provider to auth_service for authentication flow

### Integration for User Story 6

- [ ] T064 [US6] Verify auth endpoints work with PostgreSQL via integration test
- [ ] T065 [US6] Verify JWT tokens are correctly generated and validated
- [ ] T066 [US6] Verify refresh token rotation works correctly
- [ ] T067 [US6] Test complete login → document list → logout flow
- [ ] T068 [US6] Verify flutter_app login page integrates with auth_service correctly

**Checkpoint**: User Story 6 complete - authentication system is fully functional

---

## Phase 4: User Story 1 - Document Creation & Editing (Priority: P1) 🎯 MVP

**Goal**: Users can create documents, edit with rich text formatting, and save content

**Independent Test**: Create a new document, add headings, lists, code blocks, images, verify formatting saves correctly

### Tests for User Story 1

- [ ] T069 [P] [US1] Create backend/tests/documents/crud_test.rs for document CRUD operations
- [ ] T070 [P] [US1] Create backend/tests/documents/versions_test.rs for version operations
- [ ] T071 [P] [US1] Create flutter_app/test/document_service_test.dart for service unit tests
- [ ] T072 [P] [US1] Create flutter_app/test/document_repository_test.dart for repository tests

### Backend Implementation for User Story 1

- [ ] T073 [US1] Create documents table migration in backend/migrations/003_documents.sql
- [ ] T074 [US1] Create backend/services/document_service/src/lib.rs with service structure
- [ ] T075 [US1] Create backend/services/document_service/src/handlers.rs with CRUD handlers
- [ ] T076 [US1] Create backend/services/document_service/src/repository.rs with database operations
- [ ] T077 [US1] Create backend/services/document_service/src/validation.rs with document validation
- [ ] T078 [US1] Implement POST /spaces/{spaceId}/documents endpoint
- [ ] T079 [US1] Implement GET /documents/{documentId} endpoint
- [ ] T080 [US1] Implement PATCH /documents/{documentId} endpoint for metadata updates
- [ ] T081 [US1] Implement DELETE /documents/{documentId} endpoint (soft delete)
- [ ] T082 [US1] Implement POST /documents/{documentId}/versions for version creation

### Frontend Implementation for User Story 1

- [ ] T083 [US1] Create flutter_app/lib/domain/entities/document.dart with Document entity
- [ ] T084 [US1] Create flutter_app/lib/domain/entities/space.dart with Space entity
- [ ] T085 [US1] Create flutter_app/lib/domain/repositories/document_repository.dart interface
- [ ] T086 [US1] Create flutter_app/lib/data/repositories/document_repository_impl.dart with API implementation
- [ ] T087 [US1] Create flutter_app/lib/services/document_service.dart with CRUD operations
- [ ] T088 [US1] Create flutter_app/lib/presentation/providers/document_provider.dart with Riverpod state
- [ ] T089 [US1] Create flutter_app/lib/presentation/pages/documents/document_list_page.dart
- [ ] T090 [US1] Create flutter_app/lib/presentation/pages/documents/document_editor_page.dart
- [ ] T091 [US1] Create flutter_app/lib/presentation/widgets/rich_text_editor.dart with Flutter Quill integration
- [ ] T092 [US1] Implement document creation flow in document_list_page.dart
- [ ] T093 [US1] Implement rich text editing with Flutter Quill in document_editor_page.dart
- [ ] T094 [US1] Implement auto-save with sync_service integration

### Integration for User Story 1

- [ ] T095 [US1] Verify document CRUD endpoints work with PostgreSQL
- [ ] T096 [US1] Verify Yjs state (via y_crdt) is correctly stored and retrieved
- [ ] T097 [US1] Test document creation → editing → save → retrieve flow
- [ ] T098 [US1] Verify flutter_app editor integrates with document_service correctly

**Checkpoint**: User Story 1 complete - document creation and editing is fully functional

---

## Phase 5: User Story 2 - Document Organization (Priority: P1) 🎯 MVP

**Goal**: Users can organize documents into hierarchical spaces and folders

**Independent Test**: Create spaces, nest documents under parent documents, verify navigation structure

### Tests for User Story 2

- [ ] T099 [P] [US2] Create backend/tests/spaces/spaces_test.rs for space CRUD operations
- [ ] T100 [P] [US2] Create backend/tests/spaces/memberships_test.rs for membership operations
- [ ] T101 [P] [US2] Create flutter_app/test/space_service_test.dart for service unit tests

### Backend Implementation for User Story 2

- [ ] T102 [US2] Create spaces table migration in backend/migrations/004_spaces.sql
- [ ] T103 [US2] Create space_memberships table migration in backend/migrations/005_space_memberships.sql
- [ ] T104 [US2] Create backend/services/document_service/src/hierarchy.rs for document hierarchy
- [ ] T105 [US2] Implement GET /spaces endpoint for listing user's spaces
- [ ] T106 [US2] Implement POST /spaces endpoint for creating new spaces
- [ ] T107 [US2] Implement GET /spaces/{spaceId} endpoint for space details
- [ ] T108 [US2] Implement PATCH /spaces/{spaceId} endpoint for updating space
- [ ] T109 [US2] Implement DELETE /spaces/{spaceId} endpoint (soft delete)
- [ ] T110 [US2] Implement GET /spaces/{spaceId}/members endpoint for listing members
- [ ] T111 [US2] Implement POST /spaces/{spaceId}/members endpoint for adding members
- [ ] T112 [US2] Implement PATCH /spaces/{spaceId}/members/{userId} endpoint for updating roles
- [ ] T113 [US2] Implement DELETE /spaces/{spaceId}/members/{userId} endpoint for removing members

### Frontend Implementation for User Story 2

- [ ] T114 [US2] Create flutter_app/lib/domain/entities/space_membership.dart with membership entity
- [ ] T115 [US2] Create flutter_app/lib/domain/repositories/space_repository.dart interface
- [ ] T116 [US2] Create flutter_app/lib/data/repositories/space_repository_impl.dart with API implementation
- [ ] T117 [US2] Create flutter_app/lib/services/space_service.dart with space operations
- [ ] T118 [US2] Create flutter_app/lib/presentation/providers/space_provider.dart with Riverpod state
- [ ] T119 [US2] Create flutter_app/lib/presentation/pages/spaces/space_list_page.dart
- [ ] T120 [US2] Create flutter_app/lib/presentation/pages/spaces/space_detail_page.dart
- [ ] T121 [US2] Create flutter_app/lib/presentation/pages/spaces/space_settings_page.dart
- [ ] T122 [US2] Create flutter_app/lib/presentation/pages/spaces/member_management_page.dart
- [ ] T123 [US2] Create flutter_app/lib/presentation/widgets/sidebar_navigation.dart with hierarchical view
- [ ] T124 [US2] Implement space creation flow in space_list_page.dart
- [ ] T125 [US2] Implement member invitation flow in member_management_page.dart

### Integration for User Story 2

- [ ] T126 [US2] Verify space CRUD endpoints work with PostgreSQL
- [ ] T127 [US2] Verify hierarchical document queries work correctly
- [ ] T128 [US2] Test space creation → member invitation → document organization flow
- [ ] T129 [US2] Verify flutter_app sidebar navigation displays hierarchy correctly

**Checkpoint**: User Story 2 complete - document organization and space management is fully functional

---

## Phase 6: User Story 3 - Offline-First Access (Priority: P1) 🎯 MVP

**Goal**: Users can access and edit documents without internet connection, with automatic sync when online

**Independent Test**: Go offline, create/edit documents, go online, verify all changes sync correctly with no data loss

### Tests for User Story 3

- [ ] T130 [P] [US3] Create backend/tests/sync/sync_test.rs for sync endpoints
- [ ] T131 [P] [US3] Create backend/tests/sync/conflict_resolution_test.rs for conflict handling
- [ ] T132 [P] [US3] Create flutter_app/test/offline_service_test.dart for offline functionality
- [ ] T133 [P] [US3] Create flutter_app/test/sync_service_test.dart for sync service tests

### Backend Implementation for User Story 3

- [ ] T134 [US3] Create backend/services/sync_service/src/sync_handler.rs with sync endpoints
- [ ] T135 [US3] Create backend/services/sync_service/src/conflict_resolver.rs with CRDT conflict resolution
- [ ] T136 [US3] Implement GET /sync/documents/{documentId} endpoint for sync state
- [ ] T137 [US3] Implement POST /sync/documents/{documentId} endpoint for updates
- [ ] T138 [US3] Implement GET /sync/offline/status endpoint for sync status
- [ ] T139 [US3] Implement POST /sync/offline/sync endpoint for full sync trigger

### Frontend Implementation for User Story 3

- [ ] T140 [US3] Create flutter_app/lib/services/offline_service.dart with offline queue management
- [ ] T141 [US3] Create flutter_app/lib/services/sync_service.dart with sync orchestration
- [ ] T142 [US3] Create flutter_app/lib/data/datasources/pending_sync_datasource.dart for sync queue
- [ ] T143 [US3] Create flutter_app/lib/presentation/providers/sync_provider.dart with sync state
- [ ] T144 [US3] Implement offline document caching with Isar in offline_service.dart
- [ ] T145 [US3] Implement sync queue with retry logic in pending_sync_datasource.dart
- [ ] T146 [US3] Implement background sync when connectivity changes
- [ ] T147 [US3] Implement sync status indicator in UI

### Integration for User Story 3

- [ ] T148 [US3] Verify CRDT sync works end-to-end
- [ ] T149 [US3] Verify offline changes are queued and synced correctly
- [ ] T150 [US3] Test conflict resolution with concurrent edits
- [ ] T151 [US3] Verify flutter_app works offline and syncs correctly

**Checkpoint**: User Story 3 complete - offline-first access with sync is fully functional

---

## Phase 7: User Story 4 - Real-Time Collaboration (Priority: P2)

**Goal**: Users can see other users' cursors and edits in real-time while collaborating

**Independent Test**: Open same document in two windows, verify edits appear in <2 seconds, verify cursor visibility

### Tests for User Story 4

- [ ] T152 [P] [US4] Create backend/tests/websocket/presence_test.rs for WebSocket presence
- [ ] T153 [P] [US4] Create flutter_app/test/websocket_service_test.dart for real-time service tests

### Backend Implementation for User Story 4

- [ ] T154 [US4] Create backend/services/websocket_service/src/lib.rs with WebSocket service
- [ ] T155 [US4] Create backend/services/websocket_service/src/handlers.rs with WebSocket handlers
- [ ] T156 [US4] Create backend/services/websocket_service/src/presence.rs with presence tracking
- [ ] T157 [US4] Implement WebSocket endpoint at /ws/documents/{documentId}
- [ ] T158 [US4] Implement Yjs sync protocol over WebSocket (using y_crdt)
- [ ] T159 [US4] Implement cursor position broadcasting
- [ ] T160 [US4] Implement Redis pub/sub for multi-instance presence

### Frontend Implementation for User Story 4

- [ ] T161 [US4] Create flutter_app/lib/services/websocket_service.dart with WebSocket connection
- [ ] T162 [US4] Create flutter_app/lib/services/presence_service.dart with presence tracking
- [ ] T163 [US4] Create flutter_app/lib/presentation/providers/presence_provider.dart with presence state
- [ ] T164 [US4] Create flutter_app/lib/presentation/widgets/cursor_overlay.dart for cursor visualization
- [ ] TUS165 [US4] Implement WebSocket connection in document_editor_page.dart
- [ ] T166 [US4] Implement real-time update application from WebSocket
- [ ] T167 [US4] Implement cursor position reporting

### Integration for User Story 4

- [ ] T168 [US4] Verify WebSocket connection works end-to-end
- [ ] T169 [US4] Verify real-time updates propagate within 2 seconds
- [ ] T170 [US4] Test cursor visibility across multiple clients
- [ ] T171 [US4] Verify flutter_app displays other users' edits in real-time

**Checkpoint**: User Story 4 complete - real-time collaboration is fully functional

---

## Phase 8: User Story 5 - Version History & Restore (Priority: P2)

**Goal**: Users can view document version history and restore previous versions

**Independent Test**: Make multiple edits, view version list, compare versions, restore to previous version

### Tests for User Story 5

- [ ] T172 [P] [US5] Create backend/tests/documents/version_restore_test.rs for version restore
- [ ] T173 [P] [US5] Create flutter_app/test/version_service_test.dart for version service tests

### Backend Implementation for User Story 5

- [ ] T174 [US5] Create document_versions table migration in backend/migrations/006_document_versions.sql
- [ ] T175 [US5] Create backend/services/document_service/src/versions.rs with version handlers
- [ ] T176 [US5] Implement GET /documents/{documentId}/versions endpoint
- [ ] T177 [US5] Implement GET /documents/{documentId}/versions/{versionId} endpoint
- [ ] T178 [US5] Implement POST /documents/{documentId}/versions/{versionId}/restore endpoint
- [ ] T179 [US5] Implement automatic version creation on significant changes

### Frontend Implementation for User Story 5

- [ ] T180 [US5] Create flutter_app/lib/domain/entities/document_version.dart with version entity
- [ ] T181 [US5] Create flutter_app/lib/domain/repositories/version_repository.dart interface
- [ ] T182 [US5] Create flutter_app/lib/data/repositories/version_repository_impl.dart
- [ ] T183 [US5] Create flutter_app/lib/services/version_service.dart with version operations
- [ ] T184 [US5] Create flutter_app/lib/presentation/providers/version_provider.dart with version state
- [ ] T185 [US5] Create flutter_app/lib/presentation/pages/documents/version_history_page.dart
- [ ] T186 [US5] Create flutter_app/lib/presentation/widgets/version_comparison_widget.dart
- [ ] T187 [US5] Implement version list display in version_history_page.dart
- [ ] T188 [US5] Implement version comparison view
- [ ] T189 [US5] Implement version restore flow

### Integration for User Story 5

- [ ] T190 [US5] Verify version creation works correctly
- [ ] T191 [US5] Verify version restore creates new version (doesn't overwrite)
- [ ] T192 [US5] Test version list loading performance
- [ ] T193 [US5] Verify flutter_app version history integrates correctly

**Checkpoint**: User Story 5 complete - version history and restore is fully functional

---

## Phase 9: User Story 7 - Role-Based Access Control (Priority: P2)

**Goal**: Space owners can assign roles (Editor, Commenter, Viewer) to members with appropriate permissions

**Independent Test**: Create space, invite users with different roles, verify each role can only perform allowed actions

### Tests for User Story 7

- [ ] T194 [P] [US7] Create backend/tests/rbac/permissions_test.rs for RBAC operations
- [ ] T195 [P] [US7] Create flutter_app/test/rbac_service_test.dart for RBAC service tests

### Backend Implementation for User Story 7

- [ ] T196 [US7] Create backend/services/auth_service/src/permissions.rs with permission definitions
- [ ] T197 [US7] Create backend/services/auth_service/src/rbac.rs with RBAC middleware
- [ ] T198 [US7] Implement permission checking in document handlers
- [ ] T199 [US7] Implement permission checking in space handlers
- [ ] T200 [US7] Add role validation to all endpoints

### Frontend Implementation for User Story 7

- [ ] T201 [US7] Create flutter_app/lib/domain/value_objects/role.dart with role types
- [ ] T202 [US7] Create flutter_app/lib/services/rbac_service.dart with permission checks
- [ ] T203 [US7] Create flutter_app/lib/presentation/providers/permission_provider.dart with permission state
- [ ] T204 [US7] Create flutter_app/lib/presentation/widgets/permission_aware_widget.dart for UI filtering
- [ ] T205 [US7] Implement role-based UI in document_editor_page.dart
- [ ] T206 [US7] Implement role-based UI in member_management_page.dart

### Integration for User Story 7

- [ ] T207 [US7] Verify permission checking works correctly for all roles
- [ ] T208 [US7] Test unauthorized access is properly blocked
- [ ] T209 [US7] Verify flutter_app UI updates based on user role
- [ ] T210 [US7] Test role escalation scenarios

**Checkpoint**: User Story 7 complete - RBAC is fully functional

---

## Phase 10: User Story 8 - Document Export (Priority: P2)

**Goal**: Users can export documents to Markdown, HTML, and PDF formats

**Independent Test**: Export document to each format, verify formatting is preserved, verify file is downloadable

### Tests for User Story 8

- [ ] T211 [P] [US8] Create backend/tests/documents/export_test.rs for export operations
- [ ] T212 [P] [US8] Create flutter_app/test/export_service_test.dart for export service tests

### Backend Implementation for User Story 8

- [ ] T213 [US8] Create backend/services/document_service/src/export.rs with export handlers
- [ ] T214 [US8] Implement Markdown export with frontmatter
- [ ] T215 [US8] Implement HTML export with embedded styles
- [ ] T216 [US8] Implement PDF export using weasyprint or similar
- [ ] T217 [US8] Implement GET /documents/{documentId}/export endpoint

### Frontend Implementation for User Story 8

- [ ] T218 [US8] Create flutter_app/lib/services/export_service.dart with export operations
- [ ] T219 [US8] Create flutter_app/lib/presentation/providers/export_provider.dart with export state
- [ ] T220 [US8] Create flutter_app/lib/presentation/widgets/export_dialog.dart with format selection
- [ ] T221 [US8] Implement export format selection in document_editor_page.dart
- [ ] T222 [US8] Implement file download handling

### Integration for User Story 8

- [ ] T223 [US8] Verify all export formats work correctly
- [ ] T224 [US8] Test formatting preservation in exports
- [ ] T225 [US8] Verify flutter_app export dialog works correctly
- [ ] T226 [US8] Test large document export performance

**Checkpoint**: User Story 8 complete - document export is fully functional

---

## Phase 11: User Story 9 - Full-Text Search (Priority: P2)

**Goal**: Users can search across all documents with results in <500ms

**Independent Test**: Create documents with specific content, search for unique terms, verify results are relevant and fast

### Tests for User Story 9

- [ ] T227 [P] [US9] Create backend/tests/search/search_test.rs for search operations
- [ ] T228 [P] [US9] Create flutter_app/test/search_service_test.dart for search service tests

### Backend Implementation for User Story 9

- [ ] T229 [US9] Create comments table migration in backend/migrations/007_comments.sql
- [ ] T230 [US9] Create backend/services/search_service/src/lib.rs with search service
- [ ] T231 [US9] Create backend/services/search_service/src/handlers.rs with search handlers
- [ ] T232 [US9] Create backend/services/search_service/src/indexer.rs with document indexing
- [ ] T233 [US9] Implement PostgreSQL full-text search index
- [ ] T234 [US9] Implement GET /search endpoint with query parameters
- [ ] T235 [US9] Implement search result ranking

### Frontend Implementation for User Story 9

- [ ] T236 [US9] Create flutter_app/lib/domain/entities/search_result.dart with result entity
- [ ] T237 [US9] Create flutter_app/lib/domain/repositories/search_repository.dart interface
- [ ] T238 [US9] Create flutter_app/lib/data/repositories/search_repository_impl.dart
- [ ] T239 [US9] Create flutter_app/lib/services/search_service.dart with search operations
- [ ] T240 [US9] Create flutter_app/lib/presentation/providers/search_provider.dart with search state
- [ ] T241 [US9] Create flutter_app/lib/presentation/pages/search/search_page.dart
- [ ] T242 [US9] Create flutter_app/lib/presentation/widgets/search_bar.dart
- [ ] T243 [US9] Implement search input and results display
- [ ] T244 [US9] Implement search result highlighting

### Integration for User Story 9

- [ ] T245 [US9] Verify search results return within 500ms
- [ ] T246 [US9] Test search relevance ranking
- [ ] T247 [US9] Verify flutter_app search integrates correctly
- [ ] T248 [US9] Test search with special characters and multiple terms

**Checkpoint**: User Story 9 complete - full-text search is fully functional

---

## Phase 12: Comments Feature (Required for RBAC)

**Goal**: Users can add and resolve comments on documents

**Independent Test**: Add comment to document, verify it displays, resolve comment, verify resolution status

### Backend Implementation for Comments

- [ ] T249 Create backend/services/document_service/src/comments.rs with comment handlers
- [ ] T250 Implement GET /documents/{documentId}/comments endpoint
- [ ] T251 Implement POST /documents/{documentId}/comments endpoint
- [ ] T252 Implement PATCH /comments/{commentId} endpoint
- [ ] T253 Implement POST /comments/{commentId}/resolve endpoint
- [ ] T254 Implement POST /comments/{commentId}/unresolve endpoint
- [ ] T255 Implement DELETE /comments/{commentId} endpoint

### Frontend Implementation for Comments

- [ ] T256 Create flutter_app/lib/domain/entities/comment.dart with comment entity
- [ ] T257 Create flutter_app/lib/domain/repositories/comment_repository.dart interface
- [ ] T258 Create flutter_app/lib/data/repositories/comment_repository_impl.dart
- [ ] T259 Create flutter_app/lib/services/comment_service.dart with comment operations
- [ ] T260 Create flutter_app/lib/presentation/providers/comment_provider.dart with comment state
- [ ] T261 Create flutter_app/lib/presentation/widgets/comment_list.dart
- [ ] T262 Create flutter_app/lib/presentation/widgets/comment_input.dart
- [ ] T263 Implement comment display in document_editor_page.dart

---

## Phase 13: File Attachments

**Goal**: Users can upload and manage file attachments in documents

**Independent Test**: Upload file to document, verify file displays, delete file, verify deletion

### Backend Implementation for Files

- [ ] T264 Create backend/services/file_service/src/lib.rs with file service
- [ ] T265 Create backend/services/file_service/src/handlers.rs with file handlers
- [ ] T266 Create backend/services/file_service/src/storage.rs with MinIO integration
- [ ] T267 Create files table migration in backend/migrations/008_files.sql
- [ ] T268 Implement POST /files/upload endpoint
- [ ] T269 Implement GET /files/{fileId}/download endpoint
- [ ] T270 Implement GET /files/{fileId} endpoint for metadata
- [ ] T271 Implement DELETE /files/{fileId} endpoint
- [ ] T272 Implement chunked upload for large files

### Frontend Implementation for Files

- [ ] T273 Create flutter_app/lib/domain/entities/file.dart with file entity
- [ ] T274 Create flutter_app/lib/domain/repositories/file_repository.dart interface
- [ ] T275 Create flutter_app/lib/data/repositories/file_repository_impl.dart
- [ ] T276 Create flutter_app/lib/services/file_service.dart with file operations
- [ ] T277 Create flutter_app/lib/presentation/providers/file_provider.dart with file state
- [ ] T278 Create flutter_app/lib/presentation/widgets/file_upload_widget.dart
- [ ] T279 Create flutter_app/lib/presentation/widgets/file_list.dart
- [ ] T280 Implement file upload in document_editor_page.dart

---

## Phase 14: Share Links (Required for External Access)

**Goal**: Users can create share links for external document access

**Independent Test**: Create share link, access document via link (unauthenticated), verify access controls

### Backend Implementation for Share Links

- [ ] T281 Create share_links table migration in backend/migrations/009_share_links.sql
- [ ] T282 Create backend/services/document_service/src/sharing.rs with share link handlers
- [ ] T283 Implement POST /documents/{documentId}/share endpoint
- [ ] T284 Implement GET /documents/{documentId}/share endpoint
- [ ] T285 Implement GET /share/{token} endpoint for external access
- [ ] T286 Implement DELETE /documents/{documentId}/share/{token} endpoint

### Frontend Implementation for Share Links

- [ ] T287 Create flutter_app/lib/domain/entities/share_link.dart with share link entity
- [ ] T288 Create flutter_app/lib/domain/repositories/share_repository.dart interface
- [ ] T289 Create flutter_app/lib/data/repositories/share_repository_impl.dart
- [ ] T290 Create flutter_app/lib/services/share_service.dart with share operations
- [ ] T291 Create flutter_app/lib/presentation/providers/share_provider.dart with share state
- [ ] T292 Create flutter_app/lib/presentation/dialogs/share_link_dialog.dart
- [ ] T293 Implement share link creation and display

---

## Phase 15: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

### Performance & Optimization

- [ ] T294 [P] Add database indexes for frequently queried fields
- [ ] T295 [P] Implement connection pooling optimization
- [ ] T296 [P] Add caching layer with Redis for frequently accessed data
- [ ] T297 Optimize Flutter widget rebuild performance

### Security Hardening

- [ ] T298 [P] Implement CSRF protection
- [ ] T299 [P] Add request validation middleware
- [ ] T300 [P] Implement security headers (HSTS, X-Frame-Options, etc.)
- [ ] T301 Conduct security audit of all endpoints

### Observability

- [ ] T302 [P] Implement structured logging
- [ ] T303 [P] Add metrics collection (request latency, error rates)
- [ ] T304 [P] Implement distributed tracing for sync operations
- [ ] T305 Create health check endpoint at /health

### Documentation

- [ ] T306 [P] Update README.md with setup instructions
- [ ] T307 [P] Add API documentation (OpenAPI spec)
- [ ] T308 [P] Add user documentation for features

### Testing

- [ ] T309 [P] Run all unit tests and verify passing
- [ ] T310 [P] Run all integration tests and verify passing
- [ ] T311 [P] Run end-to-end tests and verify passing
- [ ] T312 [P] Verify test coverage >80%

### Quickstart Validation

- [ ] T313 [P] Verify all quickstart.md steps work correctly
- [ ] T314 [P] Test development environment setup from scratch
- [ ] T315 [P] Verify docker-compose setup works reliably

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-14)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Phase 15)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 6 (Auth)**: Can start after Foundational - No dependencies on other stories
- **User Story 1 (Document)**: Can start after Foundational - May integrate with US6
- **User Story 2 (Organization)**: Can start after Foundational - May integrate with US1, US6
- **User Story 3 (Offline)**: Can start after Foundational - Depends on US1
- **User Story 4 (Real-time)**: Can start after Foundational - Depends on US1, US3
- **User Story 5 (Versions)**: Can start after Foundational - Depends on US1
- **User Story 7 (RBAC)**: Can start after Foundational - Depends on US2, US6
- **User Story 8 (Export)**: Can start after Foundational - Depends on US1
- **User Story 9 (Search)**: Can start after Foundational - Depends on US1, Comments

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task T069: "Contract test for document CRUD operations in backend/tests/documents/crud_test.rs"
Task T070: "Contract test for version operations in backend/tests/documents/versions_test.rs"
Task T071: "Unit test for document service in flutter_app/test/document_service_test.dart"
Task T072: "Repository test for document operations in flutter_app/test/document_repository_test.dart"

# Launch all models for User Story 1 together:
Task T083: "Create Document entity in flutter_app/lib/domain/entities/document.dart"
Task T084: "Create Space entity in flutter_app/lib/domain/entities/space.dart"

# Launch backend/frontend in parallel after foundation:
Developer A: User Story 6 (Authentication)
Developer B: User Story 1 (Document Creation)
Developer C: User Story 2 (Organization)
```

---

## Implementation Strategy

### MVP First (User Story 6 + 1 + 2 + 3)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 6 (Authentication) - Foundation for all other stories
4. Complete Phase 4: User Story 1 (Document Creation) - Core feature
5. Complete Phase 5: User Story 2 (Organization) - Enables hierarchy
6. Complete Phase 6: User Story 3 (Offline) - Key differentiator
7. **STOP and VALIDATE**: Test MVP (US6 + US1 + US2 + US3) independently
8. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 6 → Test independently → Deploy/Demo
3. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
4. Add User Story 2 → Test independently → Deploy/Demo
5. Add User Story 3 → Test independently → Deploy/Demo
6. Continue with P2 stories (US4, US5, US7, US8, US9)
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 6 (Auth)
   - Developer B: User Story 1 (Document)
   - Developer C: User Story 2 (Organization)
   - Developer D: User Story 3 (Offline)
3. Stories complete and integrate independently

---

## Task Summary

| Phase | Description | Tasks |
|-------|-------------|-------|
| Phase 1 | Setup | 10 tasks |
| Phase 2 | Foundational | 25 tasks |
| Phase 3 | US6: Authentication | 23 tasks |
| Phase 4 | US1: Document Creation | 30 tasks |
| Phase 5 | US2: Organization | 30 tasks |
| Phase 6 | US3: Offline-First | 22 tasks |
| Phase 7 | US4: Real-Time | 20 tasks |
| Phase 8 | US5: Version History | 22 tasks |
| Phase 9 | US7: RBAC | 17 tasks |
| Phase 10 | US8: Export | 16 tasks |
| Phase 11 | US9: Search | 22 tasks |
| Phase 12 | Comments | 8 tasks |
| Phase 13 | Files | 17 tasks |
| Phase 14 | Share Links | 13 tasks |
| Phase 15 | Polish | 13 tasks |

**Total Tasks**: 308

---

## Notes

- **[P]** tasks = different files, no dependencies
- **[Story]** label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
