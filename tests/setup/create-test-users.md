# 测试用户创建指南

## 概述
为了运行 E2E 测试，您需要在数据库中创建以下角色的测试用户：

## 测试用户列表

### 1. Owner (所有者)
- **Email**: `${TEST_OWNER_EMAIL}` (环境变量)
- **Password**: `${TEST_OWNER_PASSWORD}` (环境变量)
- **权限**: 完全控制权限 - 可以创建、编辑、删除、分享、管理成员

### 2. Editor (编辑者)
- **Email**: `${TEST_EDITOR_EMAIL}` (环境变量)
- **Password**: `${TEST_EDITOR_PASSWORD}` (环境变量)
- **权限**: 可以编辑内容和添加评论，但不能删除或管理权限

### 3. Viewer (查看者)
- **Email**: `${TEST_VIEWER_EMAIL}` (环境变量)
- **Password**: `${TEST_VIEWER_PASSWORD}` (环境变量)
- **权限**: 只读访问 - 只能查看内容，不能编辑或评论

### 4. Commenter (评论者)
- **Email**: `${TEST_COMMENTER_EMAIL}` (环境变量)
- **Password**: `${TEST_COMMENTER_PASSWORD}` (环境变量)
- **权限**: 可以查看和添加评论，但不能编辑内容

## 手动创建步骤

### 方式 1: 通过应用 UI 创建

1. 启动应用
2. 注册上述每个测试账户
3. 为每个账户分配相应的角色

### 方式 2: 通过数据库脚本创建

如果您使用的是 SQL 数据库，可以运行以下脚本：

```sql
-- 创建测试用户 (示例 - 根据您的数据库架构调整)
INSERT INTO users (email, password_hash, role, created_at) VALUES
  ('owner@test.com', '$hashed_password', 'owner', NOW()),
  ('editor@test.com', '$hashed_password', 'editor', NOW()),
  ('viewer@test.com', '$hashed_password', 'viewer', NOW()),
  ('commenter@test.com', '$hashed_password', 'commenter', NOW());
```

### 方式 3: 通过 API 创建

创建一个脚本文件 `create-test-users.js` 或 `create-test-users.ts`：

```typescript
// 示例代码 - 根据您的 API 调整
async function createTestUsers() {
  // 验证必需的环境变量
  const requiredEnvVars = [
    'TEST_OWNER_EMAIL', 'TEST_OWNER_PASSWORD',
    'TEST_EDITOR_EMAIL', 'TEST_EDITOR_PASSWORD',
    'TEST_VIEWER_EMAIL', 'TEST_VIEWER_PASSWORD',
    'TEST_COMMENTER_EMAIL', 'TEST_COMMENTER_PASSWORD'
  ];

  const missingVars = requiredEnvVars.filter(varName => !process.env[varName]);
  if (missingVars.length > 0) {
    console.error('❌ 缺少必需的环境变量:', missingVars.join(', '));
    console.error('请在 .env.test 文件中设置这些变量');
    throw new Error('Missing required environment variables');
  }

  // 从环境变量读取凭证
  const users = [
    {
      email: process.env.TEST_OWNER_EMAIL,
      password: process.env.TEST_OWNER_PASSWORD,
      role: 'owner'
    },
    {
      email: process.env.TEST_EDITOR_EMAIL,
      password: process.env.TEST_EDITOR_PASSWORD,
      role: 'editor'
    },
    {
      email: process.env.TEST_VIEWER_EMAIL,
      password: process.env.TEST_VIEWER_PASSWORD,
      role: 'viewer'
    },
    {
      email: process.env.TEST_COMMENTER_EMAIL,
      password: process.env.TEST_COMMENTER_PASSWORD,
      role: 'commenter'
    },
  ];

  for (const user of users) {
    await fetch('http://localhost:3000/api/auth/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(user),
    });
  }
}

createTestUsers();
```

## 验证测试用户

运行以下测试以验证用户创建成功：

```bash
# 测试所有角色登录
npm run test:e2e -- tests/e2e/rbac.spec.ts --grep "should display user role indicator"
```

## CI/CD 配置

在您的 CI/CD 环境中设置环境变量：

```yaml
# GitHub Actions 示例
env:
  TEST_OWNER_EMAIL: ${{ secrets.TEST_OWNER_EMAIL }}
  TEST_OWNER_PASSWORD: ${{ secrets.TEST_OWNER_PASSWORD }}
  TEST_EDITOR_EMAIL: ${{ secrets.TEST_EDITOR_EMAIL }}
  TEST_EDITOR_PASSWORD: ${{ secrets.TEST_EDITOR_PASSWORD }}
  TEST_VIEWER_EMAIL: ${{ secrets.TEST_VIEWER_EMAIL }}
  TEST_VIEWER_PASSWORD: ${{ secrets.TEST_VIEWER_PASSWORD }}
  TEST_COMMENTER_EMAIL: ${{ secrets.TEST_COMMENTER_EMAIL }}
  TEST_COMMENTER_PASSWORD: ${{ secrets.TEST_COMMENTER_PASSWORD }}
```

## 注意事项

1. **密码安全**: 在生产环境中使用更强的密码
2. **数据隔离**: 确保测试用户只能访问测试数据
3. **定期清理**: 定期清理测试用户创建的数据
4. **环境分离**: 使用独立的测试数据库
