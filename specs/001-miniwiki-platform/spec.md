# Feature Specification: miniWiki Knowledge Management Platform

**Feature Branch**: `001-miniwiki-platform`  
**Created**: 2026-01-11  
**Status**: Draft  
**Input**: Product Requirements Document from `spec/prd.md`

## Clarifications

### Session 2026-01-11

- Q: Document lifecycle states → A: Basic active/deleted states only (documents have active and soft-deleted states, no draft/published/archived states for MVP)
- Q: External user access → A: Share links with optional access codes (external users access via unique links, optionally password-protected)
- Q: Comment resolution permissions → A: Document Editor+ and comment author can resolve comments (balances practicality with accountability)
- Q: Account recovery methods → A: Email-based password reset only (standard flow with 1-hour expiry link)
- Q: Document content size limits → A: 10MB per document maximum (practical limit for CRDT performance)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Document Creation & Editing (Priority: P1)

As a knowledge worker, I want to create and edit documents with rich text formatting so that I can produce professional documentation without learning complex markup languages.

**Why this priority**: Document creation is the core value proposition. Without this, the platform has no purpose. All other features depend on having documents to organize, share, and collaborate on.

**Independent Test**: Can be tested by creating a new document, adding various content types (headings, lists, images, code blocks), and verifying all content renders correctly. Delivers basic productivity value.

**Acceptance Scenarios**:

1. **Given** a user is on the document list page, **When** they click "New Document", **Then** a new document is created and opened in the editor with a default title.
2. **Given** a user is in the editor, **When** they select text and apply bold formatting, **Then** the selected text becomes bold and persists on save.
3. **Given** a user is writing a document, **When** they type Markdown syntax (e.g., `**text**` for bold), **Then** the markdown renders as formatted text in real-time.
4. **Given** a user wants to add code, **When** they insert a code block and select a language, **Then** syntax highlighting is applied to the code.
5. **Given** a user is adding images, **When** they upload or embed an image, **Then** the image displays in the document at the specified location.

---

### User Story 2 - Document Organization (Priority: P1)

As a knowledge manager, I want to organize documents into folders and subfolders so that I can find content easily and maintain a logical hierarchy.

**Why this priority**: Without organization, users cannot manage growing content libraries. Hierarchical structure is fundamental to knowledge management and directly impacts user productivity.

**Independent Test**: Can be tested by creating folders, nesting folders, moving documents between folders, and verifying the folder structure is reflected in navigation. Delivers findability value.

**Acceptance Scenarios**:

1. **Given** a user is in the sidebar, **When** they click "New Folder", **Then** a new folder is created at the current level.
2. **Given** a folder exists, **When** a user drags a document into the folder, **Then** the document is moved and appears under that folder.
3. **Given** deeply nested folders, **When** a user expands parent folders, **Then** all nested levels are visible and accessible.
4. **Given** a folder with content, **When** a user renames or deletes the folder, **Then** the action is confirmed and all contained documents are updated accordingly.

---

### User Story 3 - Offline-First Access (Priority: P1)

As a traveler, I want to access and edit all my documents without internet connection so that I can work during flights, in remote locations, and in areas with poor connectivity.

**Why this priority**: This is a key differentiator from competitors like Notion. Users explicitly need offline access for their workflows, and lacking this feature would be a deal-breaker.

**Independent Test**: Can be tested by going offline (disabling network), creating and editing documents, then going back online and verifying all changes sync correctly. Delivers productivity value in offline scenarios.

**Acceptance Scenarios**:

1. **Given** a user has opened documents previously, **When** they lose internet connection, **Then** all previously accessed documents remain readable and editable.
2. **Given** a user is editing offline, **When** they make changes to a document, **Then** changes are saved locally and marked as pending sync.
3. **Given** offline edits exist, **When** internet connection is restored, **Then** changes automatically sync to the server without user intervention.
4. **Given** two devices editing the same document offline, **When** both come online, **Then** the system resolves conflicts automatically using CRDT.

---

### User Story 4 - Real-Time Collaboration (Priority: P2)

As a team member, I want to see other team members' cursors and edits in real-time so that we can collaborate on documents without stepping on each other's toes.

**Why this priority**: Collaboration is a key value proposition, but users can collaborate via comments and version history even without real-time sync. This enhances but is not strictly required for MVP.

**Independent Test**: Can be tested by opening the same document in two browser windows, making edits in one window, and verifying changes appear in the other window within seconds. Delivers collaboration value.

**Acceptance Scenarios**:

1. **Given** two users are editing the same document, **When** one user types, **Then** the other user sees the text appear within 2 seconds.
2. **Given** multiple users are viewing a document, **When** users move their cursors, **Then** each user's cursor position and selection is visible to others.
3. **Given** users are collaborating, **When** a user saves or the document auto-saves, **Then** all connected users see the saved state.

---

### User Story 5 - Version History & Restore (Priority: P2)

As a document owner, I want to view and restore previous document versions so that I can recover from mistakes and track how documents evolved over time.

**Why this priority**: Version control addresses user pain points with inadequate version tracking in existing tools. It protects against data loss and enables accountability.

**Independent Test**: Can be tested by making multiple edits to a document, viewing version history, comparing two versions, and restoring to a previous version. Delivers safety and recovery value.

**Acceptance Scenarios**:

1. **Given** a document has been edited multiple times, **When** a user views version history, **Then** they see a list of all versions with timestamps and authors.
2. **Given** version history exists, **When** a user selects two versions, **Then** they see a visual diff highlighting additions and deletions.
3. **Given** a user wants to restore, **When** they click "Restore" on a previous version, **Then** a new version is created restoring that content (original versions preserved for audit).

---

### User Story 6 - User Authentication (Priority: P1)

As a new user, I want to create an account and log in securely so that my documents are protected and accessible only to authorized users.

**Why this priority**: Authentication is foundational to security and multi-user support. Without it, there's no way to isolate user data or implement access controls.

**Independent Test**: Can be tested by registering a new account, logging in with valid credentials, and verifying access to the user's personal documents. Delivers security value.

**Acceptance Scenarios**:

1. **Given** a new visitor, **When** they register with email and password, **Then** an account is created and they are logged in automatically.
2. **Given** a registered user, **When** they log in with correct credentials, **Then** they are redirected to their document dashboard.
3. **Given** incorrect login attempts, **When** a user provides wrong password multiple times, **Then** the account is temporarily locked to prevent brute force attacks.
4. **Given** logged-in users, **When** their session expires, **Then** they are prompted to re-authenticate.
5. **Given** a user who forgot their password, **When** they request a password reset, **Then** they receive an email with a secure reset link that expires after 1 hour.

---

### User Story 7 - Role-Based Access Control (Priority: P2)

As a space owner, I want to control who can view, comment, or edit documents so that I can share content appropriately with team members and external collaborators.

**Why this priority**: RBAC enables team collaboration with appropriate permissions. Without it, sharing is all-or-nothing, limiting practical use cases.

**Independent Test**: Can be tested by creating a space, inviting users with different roles, and verifying each role can only perform allowed actions. Delivers collaboration control value.

**Acceptance Scenarios**:

1. **Given** a space owner, **When** they invite a user as Editor, **Then** the user can create, edit, and delete documents in that space.
2. **Given** a space owner, **When** they invite a user as Commenter, **Then** the user can view documents and add comments but cannot edit content.
3. **Given** a space owner, **When** they invite a user as Viewer, **Then** the user can only view documents in read-only mode.
4. **Given** permissions have changed, **When** users attempt actions beyond their role, **Then** the system denies access with an appropriate message.

---

### User Story 8 - Document Export (Priority: P2)

As a publisher, I want to export documents to standard formats so that I can share content outside the platform or use it in other tools.

**Why this priority**: Export capability addresses vendor lock-in concerns and enables workflows that require documents outside the platform.

**Independent Test**: Can be tested by selecting a document, choosing an export format (Markdown, HTML, PDF), and verifying the exported file contains the document content correctly formatted. Delivers portability value.

**Acceptance Scenarios**:

1. **Given** a document with rich content, **When** a user exports to Markdown, **Then** the exported file preserves formatting and can be opened in other Markdown editors.
2. **Given** a document with images, **When** a user exports to HTML, **Then** images are embedded or referenced correctly in the HTML file.
3. **Given** a document needs to be shared formally, **When** a user exports to PDF, **Then** the PDF maintains document layout and is print-ready.

---

### User Story 9 - Full-Text Search (Priority: P2)

As a user with many documents, I want to search across all content so that I can quickly find specific information without manual browsing.

**Why this priority**: Search is essential for productivity as document libraries grow. Without it, users cannot efficiently retrieve information.

**Independent Test**: Can be tested by creating documents with specific content, searching for unique terms, and verifying search returns relevant results with correct ranking. Delivers findability value at scale.

**Acceptance Scenarios**:

1. **Given** documents exist in the library, **When** a user types a search query, **Then** results appear within 500ms showing matching documents.
2. **Given** search results, **When** a user clicks a result, **Then** they are navigated to the specific document at the matching location.
3. **Given** no matching documents, **When** a user searches, **Then** they see a "No results found" message with suggestions.

---

### Edge Cases

- **Concurrent offline edits conflict**: When two devices edit the same document offline and sync simultaneously, CRDT must automatically merge changes without data loss.
- **Large document with many versions**: Version history must load efficiently even for documents with thousands of edits.
- **Image upload during offline**: Images queued for upload during offline must upload correctly when back online.
- **Session expiration during editing**: Active document editing must not be lost when session expires.
- **External user access**: External collaborators without miniWiki accounts can access shared documents via share links. Links may be protected with optional access codes for sensitive content. External users have read-only access unless explicitly granted editor/commenter roles.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow users to create, edit, and delete documents with rich text formatting including bold, italic, underline, headings (H1-H6), ordered/unordered lists, code blocks, blockquotes, tables, and horizontal rules.
- **FR-001b**: System MUST enforce a maximum document size of 10MB to ensure CRDT performance and prevent abuse. Users MUST be notified when approaching this limit.
- **FR-002**: System MUST support Markdown input with live preview, allowing users to type Markdown syntax and see rendered output in real-time.
- **FR-003**: System MUST allow users to create, rename, move, and delete folders to organize documents hierarchically with unlimited nesting depth.
- **FR-004**: System MUST provide offline access to all previously opened documents, allowing full read and write capabilities without internet connection.
- **FR-005**: System MUST automatically sync offline changes when internet connection is restored, with no user intervention required.
- **FR-006**: System MUST use CRDT (Conflict-free Replicated Data Type) technology to automatically resolve conflicts from concurrent offline edits.
- **FR-007**: System MUST allow users to register accounts with email and password, with password hashed using bcrypt (cost factor 12) before storage.
- **FR-008**: System MUST implement JWT-based stateless authentication with access tokens expiring in 15 minutes and refresh tokens expiring in 7 days.
- **FR-008b**: System MUST support share links for external document access. Share links allow unauthenticated users to view documents. Share links MAY be protected with optional access codes. Share link URLs MUST be unique and difficult to guess (UUID-based).
- **FR-009**: System MUST support four role levels: Owner (full control), Editor (create/edit/delete documents), Commenter (view and comment), and Viewer (read-only).
- **FR-010**: System MUST track document version history, displaying timestamps, authors, and enabling visual diff comparison between versions.
- **FR-011**: System MUST allow users to restore previous versions, creating a new version rather than overwriting history.
- **FR-012**: System MUST support exporting documents to Markdown, HTML, and PDF formats with formatting preserved.
- **FR-013**: System MUST provide full-text search across all documents, returning results within 500ms with relevance ranking.
- **FR-014**: System MUST support file attachments up to 50MB per file, with compression configurable by administrators.
- **FR-015**: System MUST implement rate limiting: 100 requests/hour for anonymous users, 1000 requests/hour for authenticated users.
- **FR-016**: System MUST log all security events (login, logout, permission changes) in an immutable audit trail retained for 90 days.
- **FR-017**: System MUST encrypt all data at rest using AES-256 and all network traffic using TLS 1.3.

### Key Entities

- **User**: Represents a registered user with authentication credentials, profile information, and role assignments. Attributes: id, email, password_hash, full_name, avatar_url, created_at, updated_at.
- **Space**: Represents a workspace that contains documents and user memberships. Attributes: id, name, description, owner_id, created_at, updated_at.
- **Document**: Represents a content document with CRDT state for sync. Documents have a simple lifecycle: active (normal use) and deleted (soft-deleted via `deleted_at` timestamp). Attributes: id, space_id, owner_id, title, yjs_state, version, parent_id (for folder hierarchy), is_folder, created_at, updated_at, deleted_at.
- **DocumentVersion**: Represents a historical snapshot of document content. Attributes: id, document_id, yjs_state, version, created_by, created_at.
- **SpaceMembership**: Represents user association with a space and their role. Attributes: id, space_id, user_id, role, created_at.
- **Comment**: Represents a threaded comment on a document. Comments can be resolved by users with Editor role or higher, or by the original comment author. Resolved comments are visually distinguished but remain visible for audit. Attributes: id, document_id, user_id, parent_id (for reply threads), content, resolved, created_at, updated_at.
- **File**: Represents an uploaded file attachment. Attributes: id, document_id, uploaded_by, filename, content_type, size_bytes, storage_key, is_compressed, created_at.
- **AuditLog**: Represents a security event for compliance. Attributes: id, user_id, action, entity_type, entity_id, metadata, created_at.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create and save a document in under 30 seconds from initial click to saved state.
- **SC-002**: Users can find any previously created document through organization or search within 15 seconds.
- **SC-003**: Offline document editing and sync works seamlessly - users report zero data loss after working offline and syncing.
- **SC-004**: 95% of search queries return results within 500ms.
- **SC-005**: Real-time collaboration shows other users' edits within 2 seconds of occurrence.
- **SC-006**: Version history loads within 2 seconds and diff comparison is readable without refresh.
- **SC-007**: 99.9% of authentication attempts succeed on first try with valid credentials.
- **SC-008**: All exported documents maintain formatting fidelity - recipients cannot distinguish exported from platform-rendered content.
- **SC-009**: System supports 1,000 concurrent users with average API response time under 200ms.
- **SC-010**: New users can complete onboarding and create their first document within 5 minutes of registration.
