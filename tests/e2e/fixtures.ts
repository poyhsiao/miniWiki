import { test as base, Page, BrowserContext } from '@playwright/test';

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

      // Wait for login to complete
      await page.waitForTimeout(3000);

      // Verify login succeeded (check for redirect away from login or presence of user indicator)
      const userIndicatorVisible = await page.locator('[aria-label*="user"], [class*="user-menu"]').isVisible();
      const isLoggedIn = page.url().includes('/spaces') || userIndicatorVisible;

      if (!isLoggedIn) {
        console.warn(`Login as ${role} may have failed - no redirect or user indicator found`);
      }
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
    // Navigate to a document or create one
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Store created document name for cleanup
    let createdDocName = '';

    // Create a test document
    const createButton = page.locator('button:has-text("Create"), button:has-text("New"), [aria-label*="Create"]');
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

        // Navigate to the document
        const docItem = page.locator('.document-item').filter({ hasText: createdDocName }).first();
        if (await docItem.isVisible()) {
          await docItem.click();
          await page.waitForLoadState('networkidle');
          await page.waitForTimeout(2000);
        }
      }
    }

    await use(page);

    // Cleanup: Delete the test document by name
    try {
      if (createdDocName) {
        // Navigate back to spaces to find the document
        await page.goto('/spaces');
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);

        // Locate the specific document by its unique name
        const createdDoc = page.locator(`.document-item:has-text("${createdDocName}")`);
        if (await createdDoc.isVisible()) {
          const moreButton = createdDoc.locator('[aria-label*="more"], [aria-label*="options"]');
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
              }
            }
          }
        }
      }
    } catch (error) {
      console.log('Cleanup failed:', error);
    }
  },

  // Fixture for page with clean space state
  cleanSpacePage: async ({ page }, use) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Create a test space with unique name
    let createdSpaceName = '';
    const createButton = page.locator('button:has-text("Create Space"), button:has-text("New Space")');
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
      }
    }

    await use(page);

    // Cleanup: Delete the test space by name
    try {
      if (createdSpaceName) {
        // Navigate back to spaces list
        await page.goto('/spaces');
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);

        // Locate the specific space by its unique name
        const createdSpace = page.locator(`.space-item:has-text("${createdSpaceName}")`);
        if (await createdSpace.isVisible()) {
          const moreButton = createdSpace.locator('[aria-label*="More"], [aria-label*="Options"]');
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
              }
            }
          }
        }
      }
    } catch (error) {
      console.log('Cleanup failed:', error);
    }
  },
});

export { expect } from '@playwright/test';
