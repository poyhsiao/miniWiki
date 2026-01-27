# 測試覆蓋率改進快速開始指南

## 立即開始的 5 個步驟

### 步驟 1: 運行覆蓋率分析（5 分鐘）

```bash
cd flutter_app

# 運行測試並生成覆蓋率報告
flutter test --coverage

# 運行覆蓋率分析腳本
dart coverage_analysis.dart

# 查看當前覆蓋率報告
cat coverage/REPORT.md
```

**當前狀態**: 34.4% 覆蓋率，61 個已測試文件

### 步驟 2: 選擇第一個測試目標（10 分鐘）

根據分析，推薦按以下順序開始：

**第一批（高優先級）**:
1. `file_repository_impl.dart` - 0% 覆蓋率，137 行未覆蓋
2. `version_repository_impl.dart` - 0% 覆蓋率，48 行未覆蓋
3. `search_repository_impl.dart` - 0% 覆蓋率，25 行未覆蓋

**為什麼從這些開始？**
- 已有完整的測試模板可用
- 影響範圍明確
- 容易快速看到成果
- 為其他測試提供基礎

### 步驟 3: 使用測試模板（15 分鐘）

已為您創建的測試模板：

1. **查看 file_repository_impl 測試模板**:
   ```bash
   cat test/file_repository_impl_test.dart
   ```

2. **查看 version_repository_impl 測試模板**:
   ```bash
   cat test/version_repository_impl_test.dart
   ```

3. **了解測試輔助工具**:
   ```bash
   cat test/helpers/test_helpers.dart
   cat test/helpers/mocks.dart
   cat test/fixtures/fixtures.dart
   ```

### 步驟 4: 運行並驗證測試（5 分鐘）

```bash
# 運行特定測試文件
flutter test test/file_repository_impl_test.dart

# 運行所有測試
flutter test

# 重新生成覆蓋率報告
flutter test --coverage
dart coverage_analysis.dart
```

### 步驟 5: 追蹤進度（持續）

每週執行以下命令檢查進度：

```bash
# 生成覆蓋率報告
flutter test --coverage && dart coverage_analysis.dart

# 查看總體覆蓋率
grep "總體覆蓋率" coverage/REPORT.md
```

---

## 第一週具體任務

### Day 1-2: file_repository_impl.dart

**目標**: 達到 85% 覆蓋率

**檢查清單**:
- [ ] 上傳文件（成功路徑）
- [ ] 上傳文件（失敗路徑）
- [ ] 上傳進度回調
- [ ] 下載文件
- [ ] 下載進度回調
- [ ] 獲取預簽名 URL（上傳和下載）
- [ ] 獲取文件信息
- [ ] 列出文件（含分頁和過濾）
- [ ] 刪除文件
- [ ] 恢復文件
- [ ] 永久刪除文件
- [ ] 分塊上傳（初始化、上傳、完成、取消）
- [ ] 批量刪除文件

**驗證命令**:
```bash
flutter test test/file_repository_impl_test.dart --coverage
lcov --summary coverage/lcov.info | grep file_repository_impl
```

### Day 3: version_repository_impl.dart

**目標**: 達到 85% 覆蓋率

**檢查清單**:
- [ ] 列出版本
- [ ] 獲取特定版本
- [ ] 創建版本
- [ ] 恢復版本
- [ ] 比較版本
- [ ] 獲取版本計數
- [ ] 獲取當前版本
- [ ] 錯誤處理（所有方法）

### Day 4: search_repository_impl.dart

**目標**: 達到 85% 覆蓋率

**檢查清單**:
- [ ] 全文搜索
- [ ] 高級搜索（過濾器）
- [ ] 搜索結果分頁
- [ ] 空結果處理
- [ ] 搜索錯誤處理

### Day 5-7: 其他低覆蓋率文件

根據優先級繼續：
1. share_repository_impl.dart
2. share_service.dart
3. search_provider.dart

---

## 測試編寫最佳實踐

### 1. AAA 模式（Arrange-Act-Assert）

```dart
test('should upload file successfully', () async {
  // Arrange - 準備測試數據
  const testFileId = 'test-file-id';
  final mockResponse = MockResponse();
  when(() => mockClient.post(any())).thenAnswer((_) async => mockResponse);

  // Act - 執行被測試的代碼
  final result = await repository.uploadFile(...);

  // Assert - 驗證結果
  expect(result.id, testFileId);
  verify(() => mockClient.post(any())).called(1);
});
```

### 2. 測試命名約定

```dart
// ✅ 好的測試名稱
test('should return file when file exists', () async {});
test('should throw 404 when file not found', () async {});
test('should upload file with progress callback', () async {});

// ❌ 不好的測試名稱
test('test upload', () async {});
test('file test', () async {});
```

### 3. 測試所有路徑

```dart
// 成功路徑
test('should successfully delete file', () async {});

// 失敗路徑
test('should throw error when delete fails', () async {});

// 邊界條件
test('should handle empty file list', () async {});
test('should handle pagination edge cases', () async {});
```

### 4. 使用測試輔助工具

```dart
// 使用夾具創建測試數據
final testFile = TestFixtures.createTestFile();

// 使用 Mock 工廠
final mockResponse = MockResponseFactory.createSuccessResponse(data);

// 使用輔助函數
await TestHelpers.delay();
```

---

## 常見問題排查

### 問題 1: 測試運行失敗

**症狀**:
```
Failed to load mock class
```

**解決方案**:
```dart
// 在測試文件開頭添加
void main() {
  // 註冊所有 fallback 值
  TestHelpers.registerFallbacks();

  setUp(() {
    // 設置代碼
  });
}
```

### 問題 2: 異步測試超時

**症狀**:
```
Test timed out after 0:00:30.000
```

**解決方案**:
```dart
test('async test', () async {
  // 添加超時設置
  await repository.uploadFile(...).timeout(
    const Duration(seconds: 5),
  );
});
```

### 問題 3: Mock 驗證失敗

**症狀**:
```
No matching calls found
```

**解決方案**:
```dart
// 確保使用 any() 匹配任何參數
verify(() => mockClient.get(
  any(),  // 使用 any()
  queryParams: any(named: 'queryParams'),  // 使用 any(named: 'key')
)).called(1);
```

---

## 進度追蹤模板

### 週報模板

```markdown
# 測試進度報告 - 週 X

## 覆蓋率變化
- 本週開始: XX%
- 本週結束: XX%
- 提升: +XX%

## 完成的測試
- [x] file_repository_impl.dart (0% → 85%)
- [x] version_repository_impl.dart (0% → 85%)
- [ ] search_repository_impl.dart (0% → 60%)

## 遇到的問題
1. 問題描述
   - 解決方案

## 下週計劃
1. 完成 search_repository_impl.dart
2. 開始 share_repository_impl.dart
```

### 日常檢查清單

```bash
# 每天開始前
git pull origin main
flutter pub get

# 完成測試後
flutter test test/your_test.dart
flutter test test/your_test.dart --coverage

# 提交前
git add test/
git commit -m "test: add tests for XXX repository"
```

---

## 快速參考

### 常用命令

```bash
# 運行所有測試
flutter test

# 運行特定測試
flutter test test/file_repository_impl_test.dart

# 運行特定測試組
flutter test --name "upload"

# 生成覆蓋率報告
flutter test --coverage

# 查看覆蓋率摘要
dart coverage_analysis.dart

# 生成 HTML 報告
genhtml coverage/lcov.info -o coverage/html
open coverage/html/index.html
```

### 重要文件

- 📋 測試計劃: `test/TEST_IMPROVEMENT_PLAN.md`
- 📊 覆蓋率報告: `coverage/REPORT.md`
- 🔧 分析腳本: `coverage_analysis.dart`
- 📦 測試輔助: `test/helpers/`
- 🎯 測試模板: `test/*_test.dart`

---

## 需要幫助？

如果遇到問題：

1. **查看完整報告**: `COVERAGE_IMPROVEMENT_SUMMARY.md`
2. **查看測試計劃**: `test/TEST_IMPROVEMENT_PLAN.md`
3. **檢查測試模板**: `test/*_test.dart`
4. **使用測試輔助**: `test/helpers/`

---

**開始時間**: 2026-01-26
**預計完成**: 2026-03-23（8 週）
**目標覆蓋率**: 80%

讓我們開始吧！🚀
