# 已應用的修復總結

本文件記錄了所有已應用的程式碼修復。

## 1. backend/services/document_service/src/comments.rs

### 修復 1: 移除 unresolve_comment 中未使用的變數 (行 421-424)

- **問題**: `author_name` 和 `author_avatar` 被聲明但從未使用
- **修復**: 移除這兩個未使用的變數聲明
- **影響**: 消除死代碼警告

### 修復 2: 改進 update_comment 的錯誤處理 (行 269-281)

- **問題**: 錯誤處理不當,使用 `_` 匹配所有情況導致錯誤被吞噬
- **修復**:
  - 將 `Ok(None)` 情況返回 404 NotFound 錯誤
  - 將 `Err(e)` 情況返回 500 InternalServerError 並記錄錯誤
  - 確保客戶端始終收到一致的響應結構
- **影響**: 提高錯誤處理的可靠性和一致性

### 修復 3: 修正 resolve_comment 的響應 (行 352-355)

- **問題**:
  - 綁定了 `Ok(_comment)` 但未使用
  - 聲明了未使用的 `author_name` 和 `author_avatar`
  - 返回空元組而非實際的評論數據
- **修復**:
  - 將綁定改為 `comment` (移除下劃線)
  - 將未使用的變數重命名為 `_author_name` 和 `_author_avatar`
  - 返回實際的 `comment` 數據
- **影響**: 提供更有用的 API 響應

## 2. backend/services/document_service/src/repository.rs

### 修復 4: 正確傳播數據庫錯誤 (行 826-839)

- **問題**: 使用 `.await.unwrap_or(None).unwrap_or(0)` 靜默吞噬數據庫錯誤
- **修復**: 改為 `.await?.unwrap_or(0)` 以正確傳播 SQL 錯誤
- **影響**: 數據庫錯誤現在會正確地向上傳播,而不是被轉換為 0

### 修復 5: 為 CommentRow 添加 Serialize trait (行 75)

- **問題**: `CommentRow` 缺少 `Serialize` trait,無法序列化為 JSON
- **修復**: 在 derive 宏中添加 `serde::Serialize`
- **影響**: 修復編譯錯誤,允許 CommentRow 在 HTTP 響應中序列化

## 3. backend/src/main.rs

### 修復 6: 修復重複日誌輸出 (行 20-41)

- **問題**:
  - 基礎訂閱器附加了默認的 `fmt::layer()`
  - 當設置 `RUST_LOG_JSON` 時會添加第二個 JSON 層
  - 導致重複輸出
- **修復**:
  - 移除基礎訂閱器中的 `fmt::layer()`
  - 根據 `RUST_LOG_JSON` 環境變數條件性地添加 JSON 或普通格式層
  - 確保只調用一次 `.init()`
- **影響**: 消除重複的日誌輸出

## 4. backend/tests/auth/integration_test.rs

### 修復 7: 修正測試初始化 (行 28)

- **問題**: 在調用 `test::call_service()` 之前未初始化 App
- **修復**: 在創建 App 後調用 `test::init_service(&mut app).await`
- **影響**: 確保測試服務正確初始化

### 修復 8: 修正測試斷言並添加新測試 (行 41)

- **問題**: `test_register_endpoint_returns_201` 接受 201 或 400,與測試名稱矛盾
- **修復**:
  - 修改測試使用有效輸入並只斷言 201
  - 添加新的 `test_register_endpoint_returns_400` 測試,使用無效輸入並斷言 400
- **影響**: 測試現在更準確地反映其意圖

## 5. README.md

### 修復 9: 更新 GitHub 倉庫 URL (行 52)

- **問題**: git clone URL 使用了錯誤的用戶名 "kimhsiao"
- **修復**: 將用戶名從 "kimhsiao" 改為 "poyhsiao"
- **影響**: 用戶可以從正確的倉庫克隆項目

## 6. backend/src/middleware/security_headers.rs

### 修復 10: 移除已棄用的 X-XSS-Protection 標頭 (行 68-71)

- **問題**: X-XSS-Protection 標頭已被棄用,可能導致 XS-Leak 風險
- **修復**: 刪除設置 X-XSS-Protection 標頭的代碼
- **影響**:
  - 消除潛在的安全風險
  - CSP (Content-Security-Policy) 已提供足夠的保護

## 7. backend/src/observability.rs

### 修復 11: 移除未使用的 imports (行 9-13)

- **問題**: 多個未使用的 imports 導致編譯器警告
- **修復**: 移除以下未使用的 imports:
  - `ServiceRequest` from `actix_web::dev`
  - `AtomicUsize` from `std::sync::atomic`
  - `Instant` from `std::time`
  - `event` from `tracing`
- **影響**: 清理代碼,消除編譯器警告

## 總結

所有要求的修復已成功應用:

- ✅ 修復死代碼警告
- ✅ 改進錯誤處理和響應一致性
- ✅ 修復數據庫錯誤傳播
- ✅ 修復日誌重複輸出
- ✅ 修正測試初始化和斷言
- ✅ 更新文檔中的 URL
- ✅ 移除已棄用的安全標頭
- ✅ 清理未使用的 imports
- ✅ 添加缺失的 Serialize trait

## 剩餘的 Lint 警告

以下 lint 警告不在本次修復範圍內,但已被識別:

- WebSocket 服務中的未使用字段和方法
- Redis 連接的已棄用方法
- 同步服務中的未使用變數
- 文檔導出中的未使用賦值

這些警告來自其他服務模組,可以在後續的清理工作中處理。
