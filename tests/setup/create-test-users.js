#!/usr/bin/env node
/**
 * 测试用户创建脚本
 * 自动创建 E2E 测试所需的 4 个角色用户
 *
 * 使用方法:
 *   node tests/setup/create-test-users.js
 *
 * 或添加到 package.json:
 *   "scripts": {
 *     "test:create-users": "node tests/setup/create-test-users.js"
 *   }
 */

const dotenv = require('dotenv');
const path = require('path');

// 加载测试环境变量
dotenv.config({ path: path.resolve(__dirname, '../../.env.test') });

/**
 * 生成安全的随机密码
 */
function generateSecurePassword(length = 16) {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*';
  const crypto = require('crypto');
  let password = '';

  // Calculate the largest multiple of charset.length that fits in a byte (0-255)
  const charsetLength = charset.length;
  const maxValidValue = Math.floor(256 / charsetLength) * charsetLength;

  // Use rejection sampling to avoid modulo bias
  while (password.length < length) {
    const randomByte = crypto.randomBytes(1)[0];

    // Only accept bytes within the valid range to eliminate bias
    if (randomByte < maxValidValue) {
      password += charset[randomByte % charsetLength];
    }
    // If randomByte >= maxValidValue, reject and continue to get another byte
  }

  return password;
}

/**
 * 获取必需的环境变量，如果缺失则使用安全的随机密码
 */
function getTestCredential(envVarName, fallbackGenerator, role) {
  const value = process.env[envVarName];

  if (!value) {
    if (envVarName.includes('PASSWORD')) {
      const generated = fallbackGenerator();
      const fs = require('fs');
      const path = require('path');
      const credentialsPath = path.resolve(__dirname, '../../.generated-credentials.txt');

      try {
        // Append credential to secure file
        const credentialEntry = `${envVarName}=${generated}\n`;
        fs.appendFileSync(credentialsPath, credentialEntry, { mode: 0o600 });
        console.warn(`⚠️  ${envVarName} 未设置，已为 ${role} 生成随机密码`);
        console.warn(`   凭证已保存到: ${credentialsPath}`);
        console.warn(`   建议: 将此密码复制到 .env.test 文件中`);
      } catch (err) {
        console.error(`❌ 无法保存生成的密码到文件: ${err.message}`);
        console.warn(`⚠️  ${envVarName} 未设置，已为 ${role} 生成随机密码（请查看 ${credentialsPath}）`);
      }

      return generated;
    }
    throw new Error(
      `Missing required environment variable: ${envVarName} for ${role} role. ` +
      `Please set ${envVarName} in your .env.test file.`
    );
  }

  return value;
}

/**
 * 延迟构建测试用户配置以避免模块加载时的副作用
 */
function getTestUsers() {
  return [
    {
      role: 'owner',
      email: getTestCredential('TEST_OWNER_EMAIL', null, 'owner'),
      password: getTestCredential('TEST_OWNER_PASSWORD', generateSecurePassword, 'owner'),
      name: 'Test Owner',
      permissions: 'owner'
    },
    {
      role: 'editor',
      email: getTestCredential('TEST_EDITOR_EMAIL', null, 'editor'),
      password: getTestCredential('TEST_EDITOR_PASSWORD', generateSecurePassword, 'editor'),
      name: 'Test Editor',
      permissions: 'editor'
    },
    {
      role: 'viewer',
      email: getTestCredential('TEST_VIEWER_EMAIL', null, 'viewer'),
      password: getTestCredential('TEST_VIEWER_PASSWORD', generateSecurePassword, 'viewer'),
      name: 'Test Viewer',
      permissions: 'viewer'
    },
    {
      role: 'commenter',
      email: getTestCredential('TEST_COMMENTER_EMAIL', null, 'commenter'),
      password: getTestCredential('TEST_COMMENTER_PASSWORD', generateSecurePassword, 'commenter'),
      name: 'Test Commenter',
      permissions: 'commenter'
    }
  ];
}

// API 配置
const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000';

/**
 * 创建单个测试用户
 */
async function createUser(userData) {
  try {
    // 方式 1: 使用 fetch API（如果您的项目支持）
    const response = await fetch(`${API_BASE_URL}/api/auth/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        email: userData.email,
        password: userData.password,
        name: userData.name,
        role: userData.permissions
      })
    });

    if (response.ok) {
      const result = await response.json();
      console.log(`✅ 成功创建用户: ${userData.role} (${userData.email})`);
      return { success: true, user: result };
    } else {
      // Clone response before consuming body to handle non-JSON responses gracefully
      let error;
      try {
        const respClone = response.clone();
        error = await respClone.json();
      } catch (parseError) {
        // If JSON parsing fails, fall back to text from original response
        const errorText = await response.text();
        error = { message: errorText || `HTTP ${response.status}` };
      }

      // 如果用户已存在，不视为错误
      if (response.status === 409 || error.message?.includes('already exists')) {
        console.log(`⚠️  用户已存在: ${userData.role} (${userData.email})`);
        return { success: true, existed: true };
      }

      console.error(`❌ 创建用户失败: ${userData.role}`, error);
      return { success: false, error };
    }
  } catch (error) {
    console.error(`❌ 创建用户时发生错误: ${userData.role}`, error.message);
    return { success: false, error: error.message };
  }
}

/**
 * 创建所有测试用户
 */
async function createAllUsers() {
  console.log('🚀 开始创建测试用户...\n');
  console.log(`API 地址: ${API_BASE_URL}\n`);

  const results = [];
  const TEST_USERS = getTestUsers();

  for (const user of TEST_USERS) {
    console.log(`创建 ${user.role} 用户...`);
    const result = await createUser(user);
    results.push({ ...user, ...result });

    // 添加延迟避免请求过快
    await new Promise(resolve => setTimeout(resolve, 500));
  }

  console.log('\n📊 创建结果汇总:');
  console.log('─'.repeat(60));

  const successful = results.filter(r => r.success);
  const failed = results.filter(r => !r.success);
  const existed = results.filter(r => r.existed);

  console.log(`✅ 成功创建: ${successful.length - existed.length} 个用户`);
  console.log(`⚠️  已存在: ${existed.length} 个用户`);
  console.log(`❌ 失败: ${failed.length} 个用户`);

  if (failed.length > 0) {
    console.log('\n失败的用户:');
    failed.forEach(user => {
      console.log(`  - ${user.role} (${user.email}): ${user.error}`);
    });
  }

  console.log('\n✨ 测试用户创建完成！');

  return {
    total: results.length,
    successful: successful.length,
    failed: failed.length,
    results
  };
}

/**
 * 验证 API 连接
 */
async function verifyApiConnection() {
  try {
    console.log('🔍 验证 API 连接...');

    // Create AbortController for timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000);

    try {
      const response = await fetch(`${API_BASE_URL}/health`, {
        method: 'GET',
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      if (response.ok) {
        console.log('✅ API 连接成功\n');
        return true;
      } else {
        console.warn('⚠️  API 响应异常，但将继续尝试创建用户\n');
        return true;
      }
    } catch (fetchError) {
      clearTimeout(timeoutId);
      throw fetchError;
    }
  } catch (error) {
    console.warn('⚠️  无法连接到 API，请确保应用正在运行');
    console.warn(`   API 地址: ${API_BASE_URL}`);
    console.warn(`   错误: ${error.message}\n`);

    // Check if running in CI environment
    const isCI = process.env.CI || process.env.GITHUB_ACTIONS || process.env.CONTINUOUS_INTEGRATION;

    if (isCI) {
      // In CI, check for auto-confirm environment variable
      const autoConfirm = process.env.AUTO_CONFIRM_CREATE_USERS;
      if (autoConfirm === 'true' || autoConfirm === '1') {
        console.log('🤖 CI 环境: 自动确认继续创建用户');
        return true;
      } else {
        console.log('🤖 CI 环境: 自动跳过用户创建（设置 AUTO_CONFIRM_CREATE_USERS=true 以强制继续）');
        return false;
      }
    }

    const readline = require('readline').createInterface({
      input: process.stdin,
      output: process.stdout
    });

    return new Promise((resolve) => {
      readline.question('是否继续尝试创建用户？(y/n) ', (answer) => {
        readline.close();
        resolve(answer.toLowerCase() === 'y');
      });
    });
  }
}

/**
 * 显示使用说明
 */
function showUsage() {
  console.log('\n📖 使用说明:');
  console.log('─'.repeat(60));
  console.log('1. 确保应用正在运行');
  console.log(`2. API 地址配置在 .env.test 中: ${API_BASE_URL}`);
  console.log('3. 用户凭证也配置在 .env.test 中');
  console.log('4. 运行此脚本创建测试用户');
  console.log('\n如果创建失败，您也可以手动创建用户或通过应用 UI 注册');
  console.log('参考文档: tests/setup/create-test-users.md\n');
}

/**
 * 主函数
 */
async function main() {
  console.log('╔═══════════════════════════════════════════════════════════╗');
  console.log('║         miniWiki E2E 测试用户创建工具                    ║');
  console.log('╚═══════════════════════════════════════════════════════════╝\n');

  showUsage();

  // 验证 API 连接
  const shouldContinue = await verifyApiConnection();

  if (!shouldContinue) {
    console.log('\n❌ 用户取消操作');
    process.exit(0);
  }

  // 创建用户
  const summary = await createAllUsers();

  // 退出码
  process.exit(summary.failed > 0 ? 1 : 0);
}

// 处理未捕获的错误
process.on('unhandledRejection', (error) => {
  console.error('\n❌ 发生未处理的错误:', error);
  process.exit(1);
});

// 运行脚本
if (require.main === module) {
  main().catch(error => {
    console.error('\n❌ 脚本执行失败:', error);
    process.exit(1);
  });
}

module.exports = { createUser, createAllUsers, getTestUsers };
