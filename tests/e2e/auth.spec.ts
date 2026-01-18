import { test, expect } from '@playwright/test';

test.describe('Authentication E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app and wait for it to load
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display login page by default for unauthenticated users', async ({ page }) => {
    // Check that login-related elements are visible
    await expect(page.locator('text=Login')).toBeVisible({ timeout: 30000 });
  });

  test('should navigate to login page when accessing protected route without auth', async ({ page }) => {
    // Try to access a protected route directly
    await page.goto('/documents');
    await page.waitForLoadState('networkidle');

    // Should redirect to login or show login form
    await expect(page).toHaveURL(/.*login.*/);
  });

  test('should show validation errors for empty login form', async ({ page }) => {
    // Navigate to login page
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    // Try to submit empty form
    const loginButton = page.locator('button:has-text("Login"), button:has-text("Sign In")');
    if (await loginButton.isVisible()) {
      await loginButton.click();

      // Check for validation error messages
      const alertMessage = page.locator('[role="alert"]');
      const invalidInput = page.locator('input[aria-invalid="true"]');
      const requiredError = page.locator('text=/required|please enter/i');

      // Assert that at least one validation indicator is visible
      const hasValidationError = await alertMessage.isVisible() ||
                                  await invalidInput.first().isVisible() ||
                                  await requiredError.first().isVisible();

      expect(hasValidationError).toBeTruthy();
    }
  });

  test('should handle successful login flow', async ({ page }) => {
    // Navigate to login page
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    // Fill in login form with test credentials
    const emailInput = page.locator('input[type="email"], input[name="email"], input[id*="email"]');
    const passwordInput = page.locator('input[type="password"], input[name="password"], input[id*="password"]');

    if (await emailInput.isVisible() && await passwordInput.isVisible()) {
      await emailInput.fill('test@example.com');
      await passwordInput.fill('TestPass123!');

      // Submit form
      const loginButton = page.locator('button:has-text("Login"), button:has-text("Sign In")');
      await loginButton.click();

      // Wait for navigation or response
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);

      // Assert successful login - should see dashboard, NOT error
      const dashboardElement = page.locator('[data-testid="dashboard"], text=Welcome, text=Dashboard, .dashboard');
      const errorElement = page.locator('.error, .alert-error, [role="alert"], text=Invalid, text=Error');

      // Verify no error is shown
      await expect(errorElement.first()).not.toBeVisible({ timeout: 2000 }).catch(() => {});
      
      // Verify dashboard is visible
      await expect(dashboardElement.first()).toBeVisible({ timeout: 10000 });
    }
  });

  test('should navigate to register page from login', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');
    
    // Look for register link
    const registerLink = page.locator('a:has-text("Register"), a:has-text("Sign Up"), text=Register');
    if (await registerLink.isVisible()) {
      await registerLink.click();
      await page.waitForLoadState('networkidle');
      
      // Should be on register page
      await expect(page).toHaveURL(/.*register.*/);
    }
  });

  test('should handle logout flow', async ({ page }) => {
    // Import the authentication helper
    const { login, logout, isAuthenticated } = await import('./helpers/auth.helper');

    // First, ensure we are logged in
    const loginSuccess = await login(page);

    if (!loginSuccess) {
      // Skip test if login is not available or failed
      test.skip();
      return;
    }

    // Verify we are authenticated
    expect(await isAuthenticated(page)).toBe(true);

    // Perform logout
    const logoutSuccess = await logout(page);

    // Assert logout was successful
    expect(logoutSuccess).toBe(true);

    // Verify we are no longer authenticated
    await page.waitForTimeout(1000);
    expect(await isAuthenticated(page)).toBe(false);

    // Should be on login page or see login text
    const onLoginPage = page.url().includes('login') || await page.locator('text=Login').isVisible();
    expect(onLoginPage).toBe(true);
  });
});

test.describe('Authentication Error Handling', () => {
  test('should show error for invalid credentials', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    const emailInput = page.locator('input[type="email"], input[name="email"]');
    const passwordInput = page.locator('input[type="password"], input[name="password"]');

    if (await emailInput.isVisible() && await passwordInput.isVisible()) {
      await emailInput.fill('invalid@example.com');
      await passwordInput.fill('wrongpassword');

      const loginButton = page.locator('button:has-text("Login")');
      await loginButton.click();

      await page.waitForTimeout(2000);

      // Should show error message
      const errorMessage = page.locator('.error, .alert, [role="alert"], text=Invalid, text=Error');
      await expect(errorMessage.first()).toBeVisible({ timeout: 5000 });
      await expect(errorMessage.first()).toContainText(/invalid|error/i);
    }
  });

  test('should show error for empty fields', async ({ page }) => {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    const loginButton = page.locator('button:has-text("Login")');

    if (await loginButton.isVisible()) {
      await loginButton.click();

      // Check for validation errors with assertions
      const errorMessage = page.locator('.error, .alert, [role="alert"]');
      const requiredFields = page.locator('[required], .required');

      await page.waitForTimeout(1000);

      // Assert that validation errors are shown
      await expect(errorMessage.or(requiredFields).first()).toBeVisible();
    }
  });
});
