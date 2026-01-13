## 已完成的修复

感谢 @sourcery-ai 的详细审查！以下是对所有评论的回复：

### 1. hyper 移至 dev-dependencies ✅
hyper 已从 `[dependencies]` 移至 `[dev-dependencies]`，仅用于测试场景。

### 2. 请求体现在附加到请求中 ✅
在 `TestRequest::send` 中，现在使用 `req.set_payload(body)` 将 JSON body 正确附加到请求。

### 3. Fixture 错误传播 ✅
所有 fixture 函数现在返回 `Result<T, sqlx::Error>`，调用方必须处理可能的错误：
- `create_test_user()` → `sqlx::Result<TestUser>`
- `create_test_space()` → `sqlx::Result<TestSpace>`  
- `create_test_document()` → `sqlx::Result<TestDocument>`

### 4. TestResponse 返回类型 ✅
`send()` 现在返回自定义的 `TestResponse` 类型，使其 `json()` 辅助方法可用。

### 5. 早期失败处理 ✅
`create_test_document` 现在在找不到 space owner 时返回 `sqlx::Error::RowNotFound`，而不是返回不一致的状态。

---

**已推送修复提交**: b6bb094
