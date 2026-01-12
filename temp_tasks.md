# Tasks for miniWiki Knowledge Management Platform implementation

## Phase 1: Setup (Shared Infrastructure)

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
- [x] T008 [P] Create docker-compose.yml with postgres:14, redis:6, minio:latest services
- [x] T009 [P] Create backend/migrations/001_initial_schema.sql with all tables from data-model.md

**Database Schema Status**: Migrations created - requires running when Docker PostgreSQL is available

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
- [x] T018 [P] Create backend/services/auth_service/src/handlers.rs with auth HTTP handlers
- [x] T019 [P] Create backend/services/auth_service/src/jwt.rs with JWT token generation/validation
- [x] T020 [P] Create backend/services/auth_service/src/password.rs with bcrypt password hashing
- [x] T021 [P] Create backend/services/auth_service/src/models.rs with auth-related models
- [x] T022 [P] Create backend/services/auth_service/src/repository.rs with database operations

### API Foundation

- [x] T023 [P] Create backend/src/main.rs with Actix-web application factory
- [x] T024 [P] Create backend/src/routes/mod.rs with API route structure
- [x] T025 [P] Create backend/src/middleware/auth_middleware.rs with JWT verification

### CRDT Foundation

- [x] T026 [P] Create backend/services/sync_service/src/lib.rs with sync service structure
- [x] T027 [P] Create backend/services/sync_service/src/yjs_handler.rs with Yjs/Dart CRDT document handling
- [x] T028 [P] Create backend/services/sync_service/src/state_vector.rs with state vector operations

### Frontend Foundation

- [x] T029 [P] Create flutter_app/lib/core/config/ with environment configuration providers
- [x] T030 [P] Create flutter_app/lib/core/network/api_client.dart with Dio configuration
- [x] T031 [P] Create flutter_app/lib/core/network/network_error.dart with error handling
- [x] T032 [P] Create flutter_app/lib/services/auth_service.dart with authentication logic
- [x] T033 [P] Create flutter_app/lib/domain/repositories/auth_repository.dart interface
- [x] T034 [P] Create flutter_app/lib/data/repositories/auth_repository_impl.dart with API implementation

### Database Schema

- [x] T037 [P] Create backend/migrations/001_initial_schema.sql with all tables from data-model.md

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

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
- [x] T048 [US6] Create backend/services/auth_service/src/register.rs with registration handler
- [x] T049 [P] [US6] Create backend/services/auth_service/src/login.rs with login handler
- [x] T050 [US6] Create backend/services/auth_service/src/logout.rs with logout handler
- [x] T051 [US6] [US6] Create backend/services/auth_service/src/password_reset.rs with password reset handlers
- [x] T052 [US6] Create backend/services/auth_service/src/email_verification.rs with email verification
- [x] T053 [US6] Add refresh token endpoints in backend/services/auth_service/src/refresh.rs
- [x] T054 [US6] Add session management endpoints in backend/services/auth_service/src/sessions.rs
- [x] T055 [US6] Add rate limiting middleware for auth endpoints

### Frontend Implementation for User Story 6

- [x] T056 [US6] Create flutter_app/lib/presentation/pages/auth/login_page.dart
- [ ] T057 [US6] Create flutter_app/lib/presentation/pages/auth/register_page.dart
- [ ] T058 [US6] Create flutter_app/lib/presentation/pages/auth/password_reset_page.dart
- [ ] T059 [US6] Create flutter_app/lib/presentation/providers/auth_provider.dart with Riverpod state
- [ ] T060 [US6] Create flutter_app/lib/presentation/dialogs/email_verification_dialog.dart
- [x] T061 [US6] Implement login form validation and submission in login_page.dart
- [ ] T062 [US6] Implement registration form validation and submission in register_page.dart

### Integration for User Story 6

- [ ] T064 [US6] Verify auth endpoints work with PostgreSQL via integration test
- [ ] T065 [US6] Verify JWT tokens are correctly generated and validated
- [ ] T066 [US6] Verify refresh token rotation works correctly
- [ ] T067 [US6] Test complete login → document list → logout flow
- [ ] T068 [US6] Verify flutter_app login page integrates with auth_service correctly

**Checkpoint**: User Story 6 complete - authentication system is fully functional
