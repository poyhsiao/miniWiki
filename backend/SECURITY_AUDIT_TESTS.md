# Security Audit Integration Tests

## 測試 NULL 值處理

### 背景
`purge_old_pii` 方法現在使用 PostgreSQL 的 `IS DISTINCT FROM` 運算子來正確處理 NULL 值。

### 執行整合測試

整合測試需要實際的資料庫連線。請按照以下步驟執行:

1. **設定測試資料庫環境變數**:
```bash
export TEST_DATABASE_URL="postgresql://user:password@localhost:5432/miniwiki_test"
```

2. **執行整合測試**:
```bash
cargo test --manifest-path backend/Cargo.toml --features integration test_purge_old_pii_handles_null_values
```

### 測試涵蓋範圍

整合測試驗證以下場景:
- ✅ NULL `ip_address` + 有效 `user_id` → 兩者都被匿名化
- ✅ 有效 `ip_address` + NULL `user_id` → 兩者都被匿名化
- ✅ NULL `ip_address` + NULL `user_id` → 兩者都被匿名化

### SQL 變更說明

**修改前 (錯誤)**:
```sql
WHERE created_at < $1 AND (ip_address != 'anonymized' OR user_id != 'anonymized')
```
問題: `!=` 運算子在比較 NULL 時返回 NULL (不是 TRUE),導致 NULL 值的行被跳過。

**修改後 (正確)**:
```sql
WHERE created_at < $1
AND (ip_address IS DISTINCT FROM 'anonymized' OR user_id IS DISTINCT FROM 'anonymized')
```
優點: `IS DISTINCT FROM` 將 NULL 視為與 'anonymized' 不同,確保 NULL 值也會被更新。

### PostgreSQL IS DISTINCT FROM 語義

| 比較 | `!=` 結果 | `IS DISTINCT FROM` 結果 |
|------|-----------|------------------------|
| `NULL != 'anonymized'` | NULL (跳過) | TRUE (更新) |
| `'192.168.1.1' != 'anonymized'` | TRUE (更新) | TRUE (更新) |
| `'anonymized' != 'anonymized'` | FALSE (跳過) | FALSE (跳過) |

## 單元測試

基本的 PII 遮罩測試可以不需要資料庫連線:
```bash
cargo test --manifest-path backend/Cargo.toml test_audit_event_anonymization
```
