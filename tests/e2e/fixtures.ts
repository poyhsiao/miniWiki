import { test as base, Page, BrowserContext, expect } from '@playwright/test';

/**
 * Test Fixtures for miniWiki E2E Tests
 *
 * Provides reusable fixtures for:
 * - Authenticated page state
 * - Role-based authenticated pages (Owner, Editor, Viewer, Commenter)
 * - Clean document/space state for tests
 * - Common page objects
 */

type UserRole = 'owner' | 'editor' | 'viewer' | 'commenter';

interface RoleCredentials {
  email: string;
  password: string;
}

interface TestFixtures {
  authenticatedPage: Page;
  ownerPage: Page;
  editorPage: Page;
  viewerPage: Page;
  commenterPage: Page;
  cleanDocumentPage: Page;
  cleanSpacePage: Page;
}

/**
 * Role-specific test credentials
 * Must be set via environment variables for security
 */
function getRequiredEnvVar(name: string, role: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(
      `Missing required environment variable: ${name} for ${role} role. ` +
      `Please set ${name} in your .env.test file or environment.`
    );
  }
  return value;
}

/**
 * Lazily retrieve role credentials to avoid module-load-time failures
 */
function getRoleCredentials(role: UserRole): RoleCredentials {
  const emailEnvVar = `TEST_${role.toUpperCase()}_EMAIL`;
  const passwordEnvVar = `TEST_${role.toUpperCase()}_PASSWORD`;

  return {
    email: getRequiredEnvVar(emailEnvVar, role),
    password: getRequiredEnvVar(passwordEnvVar, role),
  };
}

/**
 * Strictly assert that the user is authenticated after a login attempt.
 * Fails fast with a clear error message if the expected post-login state is not reached.
 */
async function assertAuthenticated(page: Page, role: UserRole): Promise<void> {
  const userMenu = page.locator('[data-testid="user-menu"], [data-test-id="user-menu"], [aria-label*="User menu"], [class*="user-menu"]');

  try {
    if (await userMenu.count()) {
      await userMenu.first().waitFor({ state: 'visible', timeout: 10_000 });
      return;
    }

    await page.waitForURL(/\/spaces/, { timeout: 10_000 });
  } catch {
    const currentUrl = page.url();
    throw new Error(
      `Authentication failed for role "${String(role)}": expected a logged-in UI state or navigation to "/spaces", but current URL is "${currentUrl}".`
    );
  }
}

/**
 * Helper function to authenticate as a specific role
 */
async function authenticateAs(page: Page, role: UserRole): Promise<void> {
  const credentials = getRoleCredentials(role);

  try {
    await page.goto('/login');
    await page.waitForLoadState('networkidle');

    const emailInput = page.locator('input[type="email"], input[name="email"], input[id*="email"]');
    const passwordInput = page.locator('input[type="password"], input[name="password"], input[id*="password"]');

    if (await emailInput.isVisible() && await passwordInput.isVisible()) {
      await emailInput.fill(credentials.email);
      await passwordInput.fill(credentials.password);

      const loginButton = page.locator('button:has-text("Login"), button:has-text("Sign In")');
      await loginButton.click();

      await page.waitForTimeout(3000);

      await assertAuthenticated(page, role);
    } else {
      throw new Error(`Login form not found for ${role} authentication`);
    }
  } catch (error) {
    console.error(`Failed to authenticate as ${role}:`, error);
    throw error;
  }
}

export const test = base.extend<TestFixtures>({
  // Fixture for authenticated page (generic auth)
  authenticatedPage: async ({ page }, use) => {
    // Check for required environment variables first
    const testEmail = process.env.TEST_EMAIL;
    const testPassword = process.env.TEST_PASSWORD;

    if (!testEmail || !testPassword) {
      console.warn('TEST_EMAIL and TEST_PASSWORD environment variables are not set. Tests will run unauthenticated.');
      await use(page);
      return;
    }

    // Try to login before test
    try {
      await page.goto('/login');
      await page.waitForLoadState('networkidle');

      const emailInput = page.locator('input[type="email"], input[name="email"], input[id*="email"]');
      const passwordInput = page.locator('input[type="password"], input[name="password"], input[id*="password"]');

      if (await emailInput.isVisible() && await passwordInput.isVisible()) {
        await emailInput.fill(testEmail);
        await passwordInput.fill(testPassword);

        const loginButton = page.locator('button:has-text("Login"), button:has-text("Sign In")');
        await loginButton.click();

        // Wait for login to complete
        await page.waitForTimeout(3000);
      }
    } catch (error) {
      console.log('Login attempt failed, continuing with unauthenticated state');
    }

    await use(page);
  },

  // Fixture for Owner role
  ownerPage: async ({ browser }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
      await authenticateAs(page, 'owner');
      await use(page);
    } finally {
      await context.close();
    }
  },

  // Fixture for Editor role
  editorPage: async ({ browser }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
      await authenticateAs(page, 'editor');
      await use(page);
    } finally {
      await context.close();
    }
  },

  // Fixture for Viewer role
  viewerPage: async ({ browser }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
      await authenticateAs(page, 'viewer');
      await use(page);
    } finally {
      await context.close();
    }
  },

  // Fixture for Commenter role
  commenterPage: async ({ browser }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
      await authenticateAs(page, 'commenter');
      await use(page);
    } finally {
      await context.close();
    }
  },

  // Fixture for page with clean document state
  cleanDocumentPage: async ({ page }, use) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    let createdDocName = '';
    let createdDocLocator: ReturnType<Page['locator']>;

    const createButton = page.locator('button:has-text("Create"), button:has-text("New"), [aria-label*="Create"]');

    await expect(
      createButton,
      'Expected to find a Create/New button to initialize a clean document for this test run.'
    ).toBeVisible({ timeout: 5000 });

    if (await createButton.isVisible()) {
      await createButton.click();
      await page.waitForTimeout(500);

      const titleInput = page.locator('input[name="title"], input[id*="title"], input[placeholder*="title"]');
      if (await titleInput.isVisible()) {
        createdDocName = `Test Doc ${Date.now()}`;
        await titleInput.fill(createdDocName);

        const submitButton = page.locator('button:has-text("Create"), button:has-text("Save")');
        await submitButton.click();

        await page.waitForTimeout(2000);

        createdDocLocator = page.locator('.document-item').filter({ hasText: createdDocName }).first();

        await expect(
          createdDocLocator,
          'Expected the newly created document to be visible.'
        ).toBeVisible({ timeout: 10000 });

        if (await createdDocLocator.isVisible()) {
          await createdDocLocator.click();
          await page.waitForLoadState('networkidle');
          await page.waitForTimeout(2000);
        }
      }
    }

    await use(page);

    try {
      if (createdDocName) {
        await page.goto('/spaces');
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);

        const createdDoc = page.locator(`.document-item:has-text("${createdDocName}")`);
        if (await createdDoc.isVisible()) {
          const moreButton = createdDoc.locator('[aria-label*="more"], [aria-label*="options"]');

          await expect(
            moreButton,
            'Expected the more options button to be visible for cleanup.'
          ).toBeVisible({ timeout: 5000 });

          if (await moreButton.isVisible()) {
            await moreButton.click();
            await page.waitForTimeout(500);

            const deleteOption = page.locator('text=Delete');
            if (await deleteOption.isVisible()) {
              await deleteOption.click();
              await page.waitForTimeout(500);

              const confirmButton = page.locator('button:has-text("Delete"), button:has-text("Confirm")');
              if (await confirmButton.isVisible()) {
                await confirmButton.click();
                await page.waitForTimeout(1000);

                await expect(createdDoc).not.toBeVisible();
              }
            }
          }
        }
      }
    } catch (error) {
      console.log('Cleanup failed:', error);
      throw error;
    }
  },

  // Fixture for page with clean space state
  cleanSpacePage: async ({ page }, use) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    let createdSpaceName = '';
    let createdSpaceLocator: ReturnType<Page['locator']>;

    const createButton = page.locator('button:has-text("Create Space"), button:has-text("New Space")');

    await expect(
      createButton,
      'Expected to find a Create Space/New Space button to initialize a clean space for this test run.'
    ).toBeVisible({ timeout: 5000 });

    if (await createButton.isVisible()) {
      await createButton.click();
      await page.waitForTimeout(500);

      const nameInput = page.locator('input[name="name"], input[id*="name"]');
      if (await nameInput.isVisible()) {
        createdSpaceName = `Test Space ${Date.now()}`;
        await nameInput.fill(createdSpaceName);

        const submitButton = page.locator('button:has-text("Create"), button:has-text("Save")');
        await submitButton.click();

        await page.waitForTimeout(2000);

        createdSpaceLocator = page.locator(`.space-item:has-text("${createdSpaceName}")`);

        await expect(
          createdSpaceLocator,
          'Expected the newly created space to be visible.'
        ).toBeVisible({ timeout: 10000 });
      }
    }

    await use(page);

    try {
      if (createdSpaceName) {
        await page.goto('/spaces');
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);

        const createdSpace = page.locator(`.space-item:has-text("${createdSpaceName}")`);
        if (await createdSpace.isVisible()) {
          const moreButton = createdSpace.locator('[aria-label*="More"], [aria-label*="Options"]');

          await expect(
            moreButton,
            'Expected the more options button to be visible for cleanup.'
          ).toBeVisible({ timeout: 5000 });

          if (await moreButton.isVisible()) {
            await moreButton.click();
            await page.waitForTimeout(500);

            const deleteOption = page.locator('text=Delete');
            if (await deleteOption.isVisible()) {
              await deleteOption.click();
              await page.waitForTimeout(500);

              const confirmButton = page.locator('button:has-text("Delete")');
              if (await confirmButton.isVisible()) {
                await confirmButton.click();
                await page.waitForTimeout(1000);

                await expect(createdSpace).not.toBeVisible();
              }
            }
          }
        }
      }
    } catch (error) {
      console.log('Cleanup failed:', error);
      throw error;
    }
  },
});

export { expect } from '@playwright/test';
