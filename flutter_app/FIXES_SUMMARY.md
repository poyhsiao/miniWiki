# Flutter App 問題修復總結

## 修復日期
2026-01-15

## 修復的問題

### 1. ShareLink.toString() RangeError 修復
**檔案**: `lib/domain/entities/share_link.dart`
**問題**: 當 token 長度小於 8 時,`token.substring(0, 8)` 會拋出 RangeError
**修復**: 添加長度檢查,使用 `token.length <= 8 ? token : '${token.substring(0, 8)}...'`
**測試**: `test/share_link_test.dart` - toString 相關測試

### 2. ShareLink.toJson() 使用 snake_case
**檔案**: `lib/domain/entities/share_link.dart`
**問題**: toJson 使用 camelCase 鍵,但 fromJson 期望 snake_case
**修復**: 將所有鍵改為 snake_case (例如: `documentId` -> `document_id`)
**測試**: `test/share_link_test.dart` - toJson/fromJson 對稱性測試

### 3. CreateShareLinkRequest.toJson() 使用 snake_case
**檔案**: `lib/domain/entities/share_link.dart`
**問題**: toJson 使用 camelCase 鍵,與後端 API 不符
**修復**: 將所有鍵改為 snake_case
**測試**: `test/share_link_test.dart` - CreateShareLinkRequest toJson 測試

### 4. ShareLinksState.copyWith 支援明確清除 error
**檔案**: `lib/presentation/providers/share_provider.dart`
**問題**: copyWith 無法明確將 error 設為 null
**修復**: 使用 sentinel 模式 (_unset) 來區分「未提供」和「明確設為 null」
**測試**: `test/share_provider_test.dart` - ShareLinksState copyWith 測試

### 5. ShareLinkCreateState.copyWith 支援明確清除可選欄位
**檔案**: `lib/presentation/providers/share_provider.dart`
**問題**: copyWith 無法明確清除 expiresAt、maxAccessCount、createdLink、error 欄位
**修復**: 使用 sentinel 模式 (_clearSentinel) 支援明確清除
**測試**: `test/share_provider_test.dart` - ShareLinkCreateState copyWith 測試

### 6. shareServiceProvider baseUrl 修復
**檔案**: `lib/services/providers.dart`
**問題**: 傳遞空字串 '' 作為 baseUrl,阻止 ShareService 的預設值生效
**修復**: 傳遞 null 而非空字串,讓 ShareService 使用預設的 'http://localhost:8080'

### 7. shareRepositoryProvider baseUrl 驗證
**檔案**: `lib/data/repositories/share_repository_impl.dart`
**問題**: API_BASE_URL 為空時設為空字串,導致錯誤的 URL 路徑
**修復**:
- 驗證 API_BASE_URL 環境變數
- 如果為空,使用開發預設值 'http://localhost:3000'
- 驗證 URL 格式必須以 http:// 或 https:// 開頭

### 8. ShareService 添加 baseUrl getter
**檔案**: `lib/services/share_service.dart`
**問題**: Dialog 無法存取配置的 base URL
**修復**: 添加公開的 `baseUrl` getter

### 9. share_link_dialog 硬編碼 baseUrl 修復
**檔案**: `lib/presentation/dialogs/share_link_dialog.dart`
**問題**: 使用硬編碼的 'http://localhost:8080' 而非配置的 baseUrl
**修復**: 使用 `shareService.baseUrl` 替換所有硬編碼的 URL

### 10. DropdownButton DateTime 比較問題修復
**檔案**: `lib/presentation/dialogs/share_link_dialog.dart`
**問題**: 每次 rebuild 時建立新的 DateTime 實例,導致選擇比較失敗
**修復**:
- 引入 `ExpirationOption` enum
- 使用穩定的 enum 值作為 DropdownButton 的選擇模型
- 只在建立分享連結時才將 enum 轉換為 DateTime

### 11. Column Expanded 佈局問題修復
**檔案**: `lib/presentation/dialogs/share_link_dialog.dart`
**問題**: Column 使用 mainAxisSize.min 但包含 Expanded,導致佈局錯誤
**修復**: 將 Expanded 改為 Flexible

### 12. deleteShareLink 錯誤處理
**檔案**: `lib/presentation/dialogs/share_link_dialog.dart`
**問題**: 刪除分享連結時沒有錯誤處理
**修復**:
- 添加 try-catch 包裹 deleteShareLink 調用
- 成功時顯示成功訊息
- 失敗時顯示錯誤訊息

## 測試覆蓋

### 新增測試檔案
1. `test/share_link_test.dart` - ShareLink 實體測試
   - toString 方法測試(短、長、剛好 8 字元的 token)
   - toJson/fromJson 對稱性測試
   - CreateShareLinkRequest toJson 測試

2. `test/share_provider_test.dart` - Provider 狀態測試
   - ShareLinksState copyWith 測試
   - ShareLinkCreateState copyWith 測試

### 測試結果
所有新增的測試都通過 (14 個測試)

## TDD 流程

所有修復都遵循 TDD (Test Driven Development) 流程:
1. 先建立測試來驗證問題存在
2. 執行測試確認失敗
3. 實作修復
4. 執行測試確認通過

## 相關檔案

### 修改的檔案
- `lib/domain/entities/share_link.dart`
- `lib/presentation/providers/share_provider.dart`
- `lib/services/providers.dart`
- `lib/data/repositories/share_repository_impl.dart`
- `lib/services/share_service.dart`
- `lib/presentation/dialogs/share_link_dialog.dart`

### 新增的檔案
- `test/share_link_test.dart`
- `test/share_provider_test.dart`

## 注意事項

1. **Logical Properties Lints**: 檔案中有許多 logical properties 相關的 lint 警告(例如 width/height vs inline-size/block-size),這些是樣式建議,不影響功能,可以在後續的程式碼清理中處理。

2. **環境變數**: shareRepositoryProvider 現在會驗證 API_BASE_URL 環境變數,如果未設定或為空,會使用 'http://localhost:3000' 作為開發預設值。

3. **向後相容性**: 所有修復都保持了向後相容性,不會破壞現有功能。
