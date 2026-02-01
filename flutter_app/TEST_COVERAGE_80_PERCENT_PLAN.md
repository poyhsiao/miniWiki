# Test Coverage Improvement Plan - Target 80%

## 执行摘要

| 指标 | 当前值 | 目标值 | 差距 |
|------|--------|--------|------|
| 整体覆盖率 | 34.4% | 80% | +45.6% |
| domain层 | 84.1% | 85% | ✅已达标 |
| core层 | 77.5% | 85% | +7.5% |
| services层 | 47.3% | 80% | +32.7% |
| data层 | 38.4% | 85% | +46.6% |
| presentation层 | 14.9% | 50% | +35.1% |

---

## 第一阶段：Data层测试（目标+15%，达到~50%）

### 优先级1：version_repository_impl.dart（0% → 85%）
**文件**: `lib/data/repositories/version_repository_impl.dart` (48行未覆盖)
**测试文件**: `test/version_repository_impl_test.dart`

```dart
// 需要覆盖的方法
Future<List<DocumentVersion>> getVersions(String documentId)
Future<DocumentVersion?> getVersion(String documentId, String versionId)
Future<DocumentVersion> restoreVersion(String documentId, String versionId)
Future<int> getVersionCount(String documentId)
Future<bool> compareVersions(String versionId1, String versionId2)
```

### 优先级2：share_repository_impl.dart（0% → 85%）
**文件**: `lib/data/repositories/share_repository_impl.dart` (65行未覆盖)
**测试文件**: `test/share_repository_impl_test.dart`

### 优先级3：search_repository_impl.dart（0% → 85%）
**文件**: `lib/data/repositories/search_repository_impl.dart` (25行未覆盖)
**测试文件**: `test/search_repository_impl_test.dart`

### 优先级4：comment_repository_impl.dart（补充测试）
**文件**: `lib/data/repositories/comment_repository_impl.dart`
**测试文件**: `test/comment_repository_impl_test.dart`

---

## 第二阶段：Services层测试（目标+10%，达到~60%）

### 优先级1：sync_service.dart（7.7% → 70%）
**文件**: `lib/services/sync_service.dart` (193行未覆盖)
**测试文件**: `test/sync_service_test.dart`

### 优先级2：share_service.dart（0% → 80%）
**文件**: `lib/services/share_service.dart` (29行未覆盖)
**测试文件**: `test/share_service_test.dart`

### 优先级3：document_service.dart（补充测试）
**文件**: `lib/services/document_service.dart`
**测试文件**: `test/document_service_test.dart`

### 优先级4：file_service.dart（补充测试）
**文件**: `lib/services/file_service.dart`
**测试文件**: `test/file_service_test.dart`

---

## 第三阶段：Provider层测试（目标+12%，达到~72%）

### 优先级1：search_provider.dart（0% → 75%）
**文件**: `lib/presentation/providers/search_provider.dart` (33行未覆盖)
**测试文件**: `test/search_provider_test.dart`

### 优先级2：file_provider.dart（1.6% → 70%）
**文件**: `lib/presentation/providers/file_provider.dart` (121行未覆盖)
**测试文件**: `test/file_provider_test.dart`

### 优先级3：document_provider.dart（1.8% → 70%）
**文件**: `lib/presentation/providers/document_provider.dart` (111行未覆盖)
**测试文件**: `test/document_provider_test.dart`

### 优先级4：export_provider.dart（3.6% → 70%）
**文件**: `lib/presentation/providers/export_provider.dart`
**测试文件**: `test/export_provider_test.dart`

---

## 第四阶段：Presentation Widget测试（目标+8%，达到~80%）

### 优先级1：file_upload_widget.dart（0% → 60%）
**文件**: `lib/presentation/widgets/file_upload_widget.dart` (125行未覆盖)
**测试文件**: `test/file_upload_widget_test.dart`

### 优先级2：export_dialog.dart（0% → 60%）
**文件**: `lib/presentation/dialogs/export_dialog.dart` (147行未覆盖)
**测试文件**: `test/export_dialog_test.dart`

### 优先级3：file_list.dart（0% → 60%）
**文件**: `lib/presentation/widgets/file_list.dart` (127行未覆盖)
**测试文件**: `test/file_list_test.dart`

---

## Rust后端测试补充计划

### 需要测试的服务

| 服务 | 现有覆盖率 | 目標覆盖率 | 優先級 | 现有测试 | 需要补充的测试 | 具体功能/模块 | 预估未覆蓋行數 |
|------|-----------|-----------|--------|---------|---------------|---------------|--------------|
| auth_service | ~45% | 85% | High | handlers_test.rs, password_verification_test.rs | rbac_tests.rs, token_tests.rs | handlers: register, login, token refresh<br/>rbac: role/permission checks<br/>password: validation edge cases | ~150 |
| document_service | ~30% | 80% | High | handlers_test.rs | export.rs, sharing.rs, versioning.rs | export: markdown, pdf export<br/>sharing: permission checks, link generation<br/>versioning: diff, restore operations | ~200 |
| space_service | ~10% | 80% | Medium | - | handlers.rs, repository.rs | handlers: create, update, delete spaces<br/>repository: CRUD operations, membership | ~120 |
| sync_service | ~25% | 80% | High | sync_handler_test.rs, lib_test.rs | conflict_resolver_tests.rs, state_vector_tests.rs | conflict_resolver: merge strategies, conflict types<br/>state_vector: clock operations, causality<br/>sync_handler: websocket message handling | ~180 |
| file_service | ~20% | 80% | High | file_service_test.rs (unit) | handlers_tests.rs, storage_tests.rs, s3_tests.rs | handlers: upload, download, chunked upload<br/>storage: S3 operations, error handling<br/>chunking: parallel uploads, resumption | ~250 |
| websocket_service | ~35% | 80% | Medium | presence_test.rs | connection_manager_tests.rs, actor_tests.rs | connection_manager: lifecycle, reconnection<br/>actor: message handling, broadcast<br/>presence: online/offline tracking | ~150 |
| search_service | ~5% | 75% | Medium | - | indexer_tests.rs, handlers_tests.rs | indexer: document indexing, search relevance<br/>handlers: query parsing, filtering<br/>pagination: offset/limit edge cases | ~180 |

---

## 测试执行命令

```bash
# Flutter测试
cd flutter_app
flutter test --coverage
dart coverage_analysis.dart

# 生成HTML覆盖率报告
genhtml coverage/lcov.info -o coverage/html
open coverage/html/index.html

# 运行特定测试文件
flutter test test/version_repository_impl_test.dart

# Rust测试
cd backend
cargo test --all
cargo llvm-cov --html --output-dir target/llvm-cov
cargo llvm-cov --json --output-path target/llvm-cov/llvm-cov.json
```

---

## 成功标准

1. 整体覆盖率 ≥ 80%
2. 所有新测试通过（无失败）
3. 关键业务逻辑覆盖率 ≥ 90%
4. 所有错误处理路径都有测试
5. **测试执行时间**：拆分為兩個等級
   - **快速測試（單元測試）**：< 5 分鐘
   - **完整測試（包含集成測試）**：< 15 分鐘

> **建議**：在 CI 管道中將快速測試和完整測試分離為不同的 job，以便開發者能夠快速獲得單元測試的反饋，同時確保完整的測試套件在合併前完成。

---

## 进度跟踪

| 阶段 | 目标覆盖率 | 状态 |
|------|-----------|------|
| 当前 | 34.4% | ✅ |
| Phase 1 (Data) | ~50% | 🔜 |
| Phase 2 (Services) | ~60% | 🔜 |
| Phase 3 (Providers) | ~72% | 🔜 |
| Phase 4 (Widgets) | 80% | 🔜 |
