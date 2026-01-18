import { Page } from '@playwright/test';

/**
 * 认证辅助函数 - 用于E2E测试中的登录/登出操作
 */

/**
 * 执行登录操作
 * @param page - Playwright页面对象
 * @param email - 用户邮箱
 * @param password - 用户密码
 * @returns 登录是否成功
 */
export async function login(
  page: Page,
  email: string = 'test@example.com',
  password: string = 'TestPass123!'
): Promise<boolean> {
  await page.goto('/login');
  await page.waitForLoadState('networkidle');

  const emailInput = page.locator('input[type="email"], input[name="email"], input[id*="email"]');
  const passwordInput = page.locator('input[type="password"], input[name="password"], input[id*="password"]');

  if (!(await emailInput.isVisible()) || !(await passwordInput.isVisible())) {
    return false;
  }

  await emailInput.fill(email);
  await passwordInput.fill(password);

  const loginButton = page.locator('button:has-text("Login"), button:has-text("Sign In")');
  await loginButton.click();

  // 等待登录完成 - 检查是否跳转到非登录页面或出现用户菜单
  const userMenu = page.locator('[aria-label*="user"], [aria-label*="profile"], .user-menu');

  try {
    // 等待用户菜单出现或URL离开登录页面
    await Promise.race([
      userMenu.waitFor({ state: 'visible', timeout: 5000 }),
      page.waitForURL(/^(?!.*login).*$/, { timeout: 5000 })
    ]);

    // 验证登录状态
    const isLoggedIn = await userMenu.isVisible() || !page.url().includes('login');
    return isLoggedIn;
  } catch (error) {
    // 超时或条件未满足,登录失败
    return false;
  }
}

/**
 * 执行登出操作
 * @param page - Playwright页面对象
 * @returns 登出是否成功
 */
export async function logout(page: Page): Promise<boolean> {
  const userMenu = page.locator('[aria-label*="user"], [aria-label*="profile"], .user-menu');
  const logoutButton = page.locator('button:has-text("Logout"), button:has-text("Sign Out")');

  // 先点击用户菜单（如果需要）
  if (await userMenu.isVisible() && !(await logoutButton.isVisible())) {
    await userMenu.click();

    // 等待登出按钮出现
    try {
      await logoutButton.waitFor({ state: 'visible', timeout: 2000 });
    } catch (error) {
      return false;
    }
  }

  if (!(await logoutButton.isVisible())) {
    return false;
  }

  await logoutButton.click();

  // 等待登出完成 - 等待跳转到登录页面或登录文本出现
  try {
    await Promise.race([
      page.waitForURL('**/login**', { timeout: 3000 }),
      page.locator('text=Login').waitFor({ state: 'visible', timeout: 3000 })
    ]);

    // 验证是否已登出
    const isLoggedOut = page.url().includes('login') ||
                        await page.locator('text=Login').isVisible();
    return isLoggedOut;
  } catch (error) {
    // 超时或条件未满足,登出失败
    return false;
  }
}

/**
 * 检查当前是否已登录
 * @param page - Playwright页面对象
 * @returns 是否已登录
 */
export async function isAuthenticated(page: Page): Promise<boolean> {
  const userMenu = page.locator('[aria-label*="user"], [aria-label*="profile"], .user-menu');
  const loginText = page.locator('text=Login');

  return (await userMenu.isVisible()) && !(await loginText.isVisible());
}
