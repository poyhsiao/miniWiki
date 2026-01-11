# Development Principles

---

**Document Version**: 2.1
**Last Updated**: January 11, 2026
**Maintainer**: Development Team

## 1. Core Philosophy

### 1.1 User-Centric Design
- **Accessibility First**: All features must be accessible to users with disabilities (WCAG 2.1 AA compliant)
- **Performance Matters**: UI interactions must complete within 100ms for optimal perceived performance
- **Offline-First**: Core functionality must work without internet connection
- **Privacy by Design**: User data remains private and under user control

### 1.2 Technical Excellence
- **Code Quality**: Clean, maintainable, well-documented code
- **Security**: Zero-trust architecture, defense-in-depth
- **Scalability**: Design for 100K-500K users from day one
- **Reliability**: 99.9% uptime target for self-hosted deployments

## 2. Technology Constraints

### 2.1 Technology Stack
- **Frontend**: Flutter (Web, Desktop, Mobile)
- **Backend**: Rust (Actix-web)
- **Database**: PostgreSQL (backend), Isar (Flutter offline)
- **Cache**: Redis
- **Storage**: MinIO (self-hosted S3-compatible)
- **Sync**: Yjs CRDT

### 2.2 Framework Decisions
- **Flutter**: Used for all platforms (Web, Desktop, Mobile) - unified UI
- **Rust Backend**: Performance-critical backend services
- **No External Cloud Services**: Self-hosted only (Docker/Kubernetes)
- **CRDT for Sync**: Automatic conflict resolution

## 3. Development Workflow

### 3.1 Version Control
- **Git Flow**: Feature branches → develop → main
- **Conventional Commits**: Standardized commit messages
- **Pull Requests**: Mandatory code review for all changes
- **Protected Branches**: Main and develop require review approval

### 3.2 Code Review Standards
- **Minimum Reviewers**: 2 senior engineers
- **Automated Checks**: CI/CD must pass before review
- **Security Review**: Required for authentication and data changes
- **Performance Review**: Required for database queries and API endpoints

### 3.3 Testing Strategy
- **Unit Tests**: >80% coverage requirement
- **Integration Tests**: All API endpoints tested
- **E2E Tests**: Critical user flows automated (Playwright)
- **Manual Testing**: QA team validates each release
- **Performance Testing**: Load testing before major releases

## 4. Architecture Principles

### 4.1 Modular Design
- **Layered Architecture**: Clear separation of concerns
- **Dependency Injection**: Testable, maintainable code
- **Single Responsibility**: Each component has one job
- **Open/Closed Principle**: Open for extension, closed for modification

### 4.2 Data Management
- **Offline-First**: Local database as source of truth
- **Optimistic UI**: Immediate UI updates, sync in background
- **Conflict Resolution**: CRDTs handle merges automatically
- **Data Consistency**: Strong consistency where possible, eventual where necessary

### 4.3 API Design
- **RESTful Standards**: Consistent API design patterns
- **Versioned APIs**: Backward compatibility required
- **Rate Limiting**: Protect against abuse
- **Request Validation**: Strict input validation on all endpoints

## 5. Security Guidelines

### 5.1 Authentication & Authorization
- **JWT Tokens**: Stateless authentication
- **RBAC**: Role-based access control
- **Session Management**: Secure token handling and refresh
- **Multi-Factor Auth**: Optional MFA for enhanced security

### 5.2 Data Protection
- **Encryption at Rest**: AES-256 for stored data
- **Encryption in Transit**: TLS 1.3 for all network communication
- **Secure Storage**: Sensitive data encrypted in local storage
- **Secure Headers**: CSP, HSTS, X-Frame-Options on all responses

### 5.3 Input Validation
- **Server-Side Validation**: Never trust client input
- **SQL Injection Prevention**: Parameterized queries only
- **XSS Prevention**: Proper output encoding and CSP
- **CSRF Protection**: Token-based CSRF mitigation

## 6. Performance Standards

### 6.1 Response Time Targets
- **API Endpoints**: <200ms p95 latency
- **Page Load**: <2s initial content paint
- **Sync Operations**: <5s for incremental sync
- **Search**: <500ms for search queries

### 6.2 Resource Limits
- **Memory**: Backend services <2GB per instance
- **Database Queries**: <100ms average query time
- **File Uploads**: 50MB default limit (configurable)
- **Concurrent Connections**: 1000+ per server instance

### 6.3 Optimization Strategies
- **Caching**: Aggressive caching for frequently accessed data
- **Database Indexing**: Proper indexes for all query patterns
- **Lazy Loading**: On-demand data loading
- **Code Splitting**: Flutter lazy loading for large apps

## 7. Observability & Monitoring

### 7.1 Logging
- **Structured Logging**: JSON-formatted logs
- **Log Levels**: ERROR, WARN, INFO, DEBUG
- **Centralized Logging**: Aggregated logs from all services
- **Sensitive Data**: Never log passwords, tokens, or personal data

### 7.2 Metrics
- **Application Metrics**: Request rate, error rate, latency
- **System Metrics**: CPU, memory, disk, network
- **Business Metrics**: DAU, MAU, document creation rate
- **Custom Metrics**: Domain-specific metrics as needed

### 7.3 Alerting
- **Critical Alerts**: Immediate notification for service downtime
- **Warning Alerts**: Notification for degraded performance
- **Thresholds**: Data-driven alert thresholds
- **Escalation**: Automated escalation for unresolved alerts

## 8. Deployment Guidelines

### 8.1 Container Strategy
- **Docker Images**: All services containerized
- **Multi-Stage Builds**: Optimized image sizes
- **Base Images**: Minimal, security-scanned base images
- **Image Signing**: Signed images for production

### 8.2 Orchestration
- **Docker Compose**: MVP deployments
- **Kubernetes**: Production deployments
- **Helm Charts**: Standardized deployment manifests
- **Rolling Updates**: Zero-downtime deployments

### 8.3 Backup & Recovery
- **Daily Backups**: Automated database backups
- **Redundancy**: Multi-region backup storage
- **Disaster Recovery**: Tested recovery procedures
- **Retention**: 30-day backup retention

## 9. Team Collaboration

### 9.1 Documentation
- **API Documentation**: OpenAPI/Swagger specifications
- **Architecture Decisions**: ADRs for major decisions
- **Runbooks**: Step-by-step operational procedures
- **Onboarding**: New developer setup guides

### 9.2 Communication
- **Async First**: Written documentation over meetings
- **Pull Request Reviews**: Primary communication channel
- **Standups**: Daily team sync meetings
- **Retrospectives**: Post-sprint improvement discussions

### 9.3 Knowledge Sharing
- **Code Reviews**: Learning opportunity for all
- **Tech Talks**: Regular technical presentations
- **Pair Programming**: For complex features
- **Documentation**: Living documentation updated regularly

## 10. Quality Assurance

### 10.1 Code Quality
- **Linting**: Enforced code style
- **Static Analysis**: Security and quality scans
- **Code Coverage**: 80%+ coverage requirement
- **Complexity Limits**: Cyclomatic complexity <15 per function

### 10.2 Testing Standards
- **Test Pyramid**: More unit tests, fewer integration, even fewer E2E
- **Test Independence**: Tests must not depend on each other
- **Test Speed**: Unit tests <100ms each
- **Flaky Tests**: Must be fixed immediately

### 10.3 Release Criteria
- **All Tests Pass**: No failing tests
- **Performance Benchmarks**: Meet performance targets
- **Security Scan**: No high-severity vulnerabilities
- **Documentation Updated**: All changes documented

## 11. Continuous Improvement

### 11.1 Metrics-Driven Development
- **KPIs**: Key performance indicators tracked
- **Experiments**: A/B testing for UX improvements
- **Data Analysis**: Regular analysis of usage patterns
- **Feedback Loops**: Continuous user feedback collection

### 11.2 Technical Debt
- **Debt Tracking**: Track technical debt backlog
- **Sprints**: Dedicated time for debt reduction
- **Prioritization**: High-impact debt prioritized
- **Documentation**: Debt decisions documented

### 11.3 Innovation
- **R&D Time**: 20% time for exploration
- **Tech Watch**: Stay current with industry trends
- **Prototyping**: Proof-of-concept for new features
- **Evaluation**: Rigorous evaluation before adoption

## 12. Development Best Practices

### 12.1 Configuration Management
- **External Configuration**: All settings MUST be separated in `.env` or YAML-format files
- **No Hard-Coding**: Never hard-code configuration values into program code
- **Environment-Specific**: Separate configuration files for dev, staging, production
- **Secret Management**: Use environment variables for sensitive data (API keys, passwords)
- **Validation**: Validate configuration on application startup
- **Documentation**: Document all configuration options with examples

**Example Structure**:
```
config/
├── .env.example              # Template with all options documented
├── .env.development          # Development environment
├── .env.staging              # Staging environment
├── .env.production            # Production environment (gitignored)
└── application.yaml            # Non-sensitive configuration
```

### 12.2 Design Principles
- **KISS (Keep It Simple, Stupid)**: Never over-design programs or features
- **YAGNI (You Aren't Gonna Need It)**: Avoid implementing features not currently needed
- **DRY (Don't Repeat Yourself)**: Reuse code, avoid duplication
- **SOLID Principles**: Single responsibility, open/closed, Liskov substitution, interface segregation, dependency inversion
- **Minimal Viable Product (MVP)**: Start simple, iterate based on user feedback

### 12.3 Code Documentation
- **Comment All Code**: Every function, class, and complex logic MUST be commented
- **Comment Purpose**: Explain WHY code exists, not WHAT it does (code should be self-explanatory)
- **Inline Comments**: Use inline comments for complex logic, algorithms, or non-obvious decisions
- **Docstrings**: Use docstrings/function documentation for all public APIs
- **Comment Style**: Follow language-specific conventions (e.g., JSDoc for JavaScript, Rustdoc for Rust)
- **Outdated Comments**: Update or remove outdated comments immediately

**Example (Rust)**:
```rust
/// Authenticates a user with email and password.
///
/// # Arguments
/// * `email` - User's email address
/// * `password` - User's password (will be hashed)
///
/// # Returns
/// * `Ok(User)` on successful authentication
/// * `Err(AuthError)` if credentials are invalid
///
/// # Example
/// ```
/// match authenticate("user@example.com", "password").await {
///     Ok(user) => println!("Authenticated: {}", user.id),
///     Err(e) => eprintln!("Authentication failed: {}", e),
/// }
/// ```
pub async fn authenticate(
    email: &str,
    password: &str,
) -> Result<User, AuthError> {
    // Hash password with bcrypt (cost factor 12)
    let password_hash = bcrypt::hash(password, 12)?;

    // Query database for user with matching email
    let user = db::get_user_by_email(email).await?;

    // Compare password hashes
    if !bcrypt::verify(password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(user)
}
```

### 12.4 Database Versioning
- **Migration Files**: All database schema changes MUST be versioned migration files
- **Naming Convention**: `{timestamp}_{description}.sql` (e.g., `20260111000001_create_users_table.sql`)
- **Rollback Support**: Each migration MUST include rollback script
- **Migration Tracking**: Track executed migrations in a dedicated table
- **Zero-Downtime**: Use online migrations (non-breaking changes) when possible
- **Testing**: Test migrations on staging before production

**Migration Structure**:
```sql
-- 20260111000001_create_users_table.sql

-- UP migration
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- DOWN migration
DROP TABLE IF EXISTS users;
```

**Migration Tracking Table**:
```sql
CREATE TABLE schema_migrations (
    id SERIAL PRIMARY KEY,
    version VARCHAR(50) UNIQUE NOT NULL,  -- Migration filename
    executed_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 12.5 Test-Driven Development (TDD)
- **TDD Cycle**: Write failing test → Write minimum code → Refactor → Repeat
- **Test First**: Write tests BEFORE implementing functionality
- **Red-Green-Refactor**: Follow the TDD cycle strictly
- **Unit Tests**: Test individual functions and methods in isolation
- **Integration Tests**: Test interactions between components
- **Test Coverage**: Maintain >80% test coverage
- **Test Quality**: Tests should be readable, maintainable, and meaningful

**TDD Example**:
```rust
// 1. Write failing test
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticate_with_valid_credentials() {
        let user = create_test_user().await;
        let result = authenticate(&user.email, &user.password).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, user.id);
    }

    #[tokio::test]
    async fn test_authenticate_with_invalid_credentials() {
        let result = authenticate("invalid@example.com", "wrongpassword").await;

        assert!(result.is_err());
    }
}

// 2. Write minimum code to pass test
pub async fn authenticate(
    email: &str,
    password: &str,
) -> Result<User, AuthError> {
    let user = db::get_user_by_email(email).await?;
    
    if !bcrypt::verify(password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(user)
}

// 3. Refactor (if needed)
// ... optimize code, extract functions, etc.
```

### 12.6 Dependency Management
- **Version Verification**: Before installing ANY library/package, MUST confirm LATEST/stable version using Context7
- **Context7 Usage**: Check official documentation, GitHub releases, and community feedback
- **Stability First**: Prefer stable releases over beta/alpha versions
- **Minimal Dependencies**: Only add dependencies when absolutely necessary
- **Regular Updates**: Update dependencies regularly for security patches
- **Lock Files**: Commit lock files (package-lock.json, Cargo.lock, pubspec.lock) to ensure reproducibility
- **Vulnerability Scanning**: Run security scans on dependencies regularly

**Dependency Verification Checklist**:
- [ ] Checked Context7 for latest stable version
- [ ] Reviewed official documentation
- [ ] Checked GitHub release notes
- [ ] Verified community feedback/usage
- [ ] Confirmed license compatibility
- [ ] Reviewed security advisories
- [ ] Tested in development environment

### 12.7 Best Practices by Technology

#### 12.7.1 Rust Best Practices
- **Error Handling**: Use `Result<T, E>` instead of `unwrap()` or `expect()`
- **Ownership**: Understand and use ownership, borrowing, and lifetimes correctly
- **Async/Await**: Use async runtime (Tokio) for I/O operations
- **Pattern Matching**: Use exhaustive pattern matching instead of nested if-else
- **Traits**: Define and use traits for shared behavior
- **Clippy**: Run Clippy linter for additional checks
- **Documentation**: Use `///` for public API documentation (Rustdoc)

#### 12.7.2 Flutter/Dart Best Practices
- **Immutable State**: Use `final` for variables that shouldn't change
- **Async/Await**: Use `async`/`await` for I/O operations
- **Widget Composition**: Build complex widgets from smaller, reusable widgets
- **State Management**: Use Riverpod for predictable state management
- **Const Constructors**: Use `const` constructors for compile-time constants
- **Linter**: Run `dart analyze` and `flutter analyze` before commits
- **Effective Dart**: Enable Effective Dart linter rules

#### 12.7.3 SQL Best Practices
- **Parameterized Queries**: ALWAYS use parameterized queries to prevent SQL injection
- **Transactions**: Use transactions for multi-step operations
- **Indexes**: Create indexes for frequently queried columns
- **EXPLAIN**: Use `EXPLAIN ANALYZE` to optimize slow queries
- **Limit Results**: Always use `LIMIT` to prevent fetching too many rows
- **Avoid SELECT * **: Only select columns you need
- **Normalized Design**: Follow database normalization (1NF, 2NF, 3NF)

#### 12.7.4 API Best Practices
- **HTTP Methods**: Use correct HTTP methods (GET for read, POST for create, PUT for update, DELETE for delete)
- **Status Codes**: Return appropriate HTTP status codes (200, 201, 400, 401, 403, 404, 500)
- **Versioning**: Include API version in URL (e.g., `/api/v1/`)
- **Rate Limiting**: Implement rate limiting to prevent abuse
- **Pagination**: Use pagination for large result sets
- **Consistent Response**: Use consistent response format across all endpoints
- **Error Handling**: Return detailed, actionable error messages

### 12.8 Professional Development Methodologies

#### 12.8.1 Clean Code
- **Meaningful Names**: Use descriptive variable, function, and class names
- **Small Functions**: Functions should do one thing and do it well (<50 lines preferred)
- **No Magic Numbers**: Use named constants instead of magic numbers
- **Single Responsibility**: Each function/class should have one reason to change
- **DRY Principle**: Don't repeat yourself, extract common code
- **Comments on WHY**: Comment why code exists, not what it does
- **Code Smells**: Eliminate code smells (long methods, duplicate code, large classes)

#### 12.8.2 SOLID Principles
- **S**ingle Responsibility: Each class should have one responsibility
- **O**pen/Closed: Open for extension, closed for modification
- **L**iskov Substitution: Subtypes should be substitutable for base types
- **I**nterface Segregation: Many specific interfaces over one general interface
- **D**ependency Inversion: Depend on abstractions, not concretions

#### 12.8.3 Refactoring
- **Refactor Mercilessly**: Continuously improve code quality
- **Refactor Tests**: Keep tests passing while refactoring
- **Small Steps**: Refactor in small, verifiable steps
- **Automated Refactoring**: Use IDE refactoring tools when possible
- **Code Review**: Refactorings must go through code review

#### 12.8.4 Code Review Checklist
- [ ] Code follows project style guidelines
- [ ] All code is properly commented
- [ ] No hard-coded configuration values
- [ ] Tests are written (following TDD)
- [ ] Error handling is proper
- [ ] No security vulnerabilities
- [ ] Performance implications considered
- [ ] Documentation updated
- [ ] Dependencies verified (Context7 used)
- [ ] Database migrations versioned

#### 12.8.5 Ultrawork Development Methodology

##### 12.8.5.1 Ultrawork Mode Activation
- **Mandatory Activation**: When starting any development task, users MUST input the "ultrawork" string (or "ulw" shorthand) to activate Ultrawork Mode.
- **Purpose**: Ultrawork Mode ensures maximum precision, exhaustive agent utilization, and comprehensive verification throughout the development lifecycle.
- **Activation Method**: Begin development by typing "ultrawork" or "ulw" in the AI assistant prompt to trigger the full development workflow.

##### 12.8.5.2 Ultrawork Execution Principles
- **Agent Utilization**: Leverage ALL available agents to their fullest potential for each task:
  - **Codebase Exploration**: Use background exploration agents for file patterns, internal implementations, and project structure
  - **Documentation & References**: Use librarian agents via background tasks for API references and external library docs
  - **Planning & Strategy**: ALWAYS spawn a dedicated planning agent for work breakdown (never plan yourself)
  - **High-IQ Reasoning**: Leverage specialized agents for architecture decisions and code review
  - **Frontend/UI Tasks**: Delegate to UI-specialized agents for design and implementation
- **Parallel Execution**: Fire independent agent calls simultaneously via background_task - NEVER wait sequentially
- **Background First**: Use background_task for exploration/research agents (10+ concurrent if needed)

##### 12.8.5.3 Verification Guarantee (Non-Negotiable)
- **Pre-Implementation**: Define success criteria explicitly (Functional, Observable, Pass/Fail)
- **Test Plan**: Create mandatory test plan with test cases, prerequisites, and success criteria
- **Evidence Requirements**: NOTHING is "done" without proof it works:
  - Build command: Exit code 0, no errors
  - Test execution: All tests pass (with output)
  - Manual verification: Demonstrate feature works
  - Regression testing: Existing tests still pass
- **TDD Workflow**: Follow Red-Green-Refactor cycle strictly
- **Verification Anti-Patterns (BLOCKING)**:
  - ❌ "It should work now" - MUST run and verify
  - ❌ "I added the tests" - MUST show test output
  - ❌ "Fixed the bug" - MUST describe test verification
  - ❌ Skipping test execution - Tests exist to be RUN

##### 12.8.5.4 Zero Tolerance Failures
- **NO Scope Reduction**: Never make "demo", "skeleton", or "simplified" versions - deliver FULL implementation
- **NO MockUp Work**: When asked to implement feature X, deliver 100% working port - no mock data
- **NO Partial Completion**: Never stop at 60-80% - finish 100%
- **NO Assumed Shortcuts**: Never skip requirements deemed "optional"
- **NO Premature Stopping**: Never declare done until ALL TODOs completed and verified
- **NO TEST DELETION**: Never delete failing tests to make build pass - fix the code

##### 12.8.5.5 Implementation Workflow
1. **Analyze Request**: Identify required capabilities and spawn parallel exploration/librarian agents
2. **Plan with Agents**: Use planning agent with gathered context to create detailed work breakdown
3. **Execute with Verification**: Implement with continuous verification against original requirements
4. **Track Progress**: Use TODO list for every step, mark complete IMMEDIATELY after each
5. **Final Verification**: Re-read request after completion, verify ALL requirements met

##### 12.8.5.6 Ultrawork Checklist
- [ ] Ultrawork mode activated with "ultrawork" or "ulw" prompt
- [ ] Parallel agents launched for exploration/research
- [ ] Detailed TODO list created before implementation
- [ ] Success criteria defined explicitly
- [ ] Test plan created for non-trivial tasks
- [ ] All implementation steps tracked and marked complete
- [ ] Build verification passed (exit code 0)
- [ ] Test execution passed (all tests green)
- [ ] Manual verification demonstrated
- [ ] No scope reduction or mock implementations
- [ ] 100% feature completion achieved

#### 12.8.1 Clean Code
