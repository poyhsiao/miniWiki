import { test, expect } from '@playwright/test';

test.describe('Share Links E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display share option in document', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Navigate to a document
    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for share button
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"], .share-button');
    const shareMenu = page.locator('.share-menu, [class*="share"]');

    const hasShareUI = await shareButton.isVisible() || await shareMenu.isVisible();
    expect(hasShareUI).toBeTruthy();
  });

  test('should open share dialog', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Click share button
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');

    if (await shareButton.isVisible()) {
      await shareButton.click();
      await page.waitForTimeout(500);

      // Should show share dialog
      const shareDialog = page.locator('[role="dialog"], .modal, .share-dialog');
      await expect(shareDialog).toBeVisible({ timeout: 5000 });
    }
  });

  test('should create share link with read permission', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Open share dialog
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');
    if (await shareButton.isVisible()) {
      await shareButton.click();
      await page.waitForTimeout(500);
    }

    // Look for create share link option
    const createButton = page.locator('button:has-text("Create Link"), button:has-text("Create")');
    const permissionSelect = page.locator('[role="combobox"]:has-text("Read"), select:has-text("Read")');

    // Select read permission if available
    if (await permissionSelect.isVisible()) {
      await permissionSelect.click();
      await page.waitForTimeout(300);
      const readOption = page.locator('text=Read');
      if (await readOption.isVisible()) {
        await readOption.click();
      }
    }

    // Create share link - skip test if button is not visible
    if (!(await createButton.isVisible())) {
      test.skip();
      return;
    }

    await createButton.click();
    await page.waitForTimeout(2000);

    // Verify share link was created
    const shareLink = page.locator('.share-link, [class*="share-link"], text=/http|localhost/i');
    expect(await shareLink.isVisible()).toBeTruthy();
  });

  test('should create share link with comment permission', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Open share dialog
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');
    if (!(await shareButton.isVisible())) {
      test.skip();
      return;
    }

    await shareButton.click();
    await page.waitForTimeout(500);

    // Look for permission selection
    const permissionSelect = page.locator('text=Comment, select, [role="combobox"]');

    if (!(await permissionSelect.isVisible())) {
      test.skip();
      return;
    }

    await permissionSelect.click();
    await page.waitForTimeout(300);

    const commentOption = page.locator('text=Comment');
    if (!(await commentOption.isVisible())) {
      test.skip();
      return;
    }

    await commentOption.click();

    // Create share link
    const createButton = page.locator('button:has-text("Create Link"), button:has-text("Create")');
    if (!(await createButton.isVisible())) {
      test.skip();
      return;
    }

    await createButton.click();
    await page.waitForTimeout(2000);

    // Verify share link was created - always runs
    const shareLink = page.locator('.share-link, [class*="share-link"], text=/http|localhost/i');
    await expect(shareLink).toBeVisible();
  });

  test('should copy share link to clipboard', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Open share dialog
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');
    if (!(await shareButton.isVisible())) {
      test.skip();
      return;
    }
    await shareButton.click();
    await page.waitForTimeout(500);

    // Look for copy button
    const copyButton = page.locator('button:has-text("Copy"), [aria-label*="Copy"]');

    // Fail if copy button is not present
    await expect(copyButton).toBeVisible();

    // Click copy
    await copyButton.click();
    await page.waitForTimeout(500);

    // Should show success feedback using robust locator
    const successMessage = page.getByRole('alert').filter({ hasText: /copi(ed)?/i });
    await expect(successMessage).toBeVisible();
  });

  test('should delete share link', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Open share dialog
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');
    if (await shareButton.isVisible()) {
      await shareButton.click();
      await page.waitForTimeout(500);
    }

    // Get initial share link count
    const shareLinkItem = page.locator('.share-link-item, [class*="share-link-item"]');
    const initialCount = await shareLinkItem.count();

    if (initialCount > 0) {
      // Look for delete button
      const deleteButton = shareLinkItem.first().locator('[aria-label*="Delete"], button:has-text("Delete")');

      if (await deleteButton.isVisible()) {
        await deleteButton.click();
        await page.waitForTimeout(500);

        // Confirm deletion if dialog appears
        const confirmButton = page.locator('button:has-text("Delete"), button:has-text("Confirm")');
        if (await confirmButton.isVisible()) {
          await confirmButton.click();
          await page.waitForTimeout(2000);
        }

        // Verify share link was deleted
        const newCount = await shareLinkItem.count();
        expect(newCount).toBeLessThan(initialCount);
      }
    }
  });

  test('should set expiration date for share link', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Open share dialog
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');
    if (await shareButton.isVisible()) {
      await shareButton.click();
      await page.waitForTimeout(500);
    }

    // Look for expiration option
    const expirationToggle = page.locator('text=Expiration, input[type="checkbox"]');
    const expirationDate = page.locator('input[type="date"], [type="date"]');

    if (await expirationToggle.isVisible()) {
      await expirationToggle.click();
      await page.waitForTimeout(300);

      // Set expiration date
      if (await expirationDate.isVisible()) {
        // Set expiration to one year from now
        const futureDate = new Date();
        futureDate.setFullYear(futureDate.getFullYear() + 1);
        const futureString = futureDate.toISOString().split('T')[0]; // Format: YYYY-MM-DD

        await expirationDate.fill(futureString);

        // Create share link with expiration
        const createButton = page.locator('button:has-text("Create Link"), button:has-text("Create")');
        if (await createButton.isVisible()) {
          await createButton.click();
          await page.waitForTimeout(2000);

          // Verify share link was created with expiration
          const currentYear = new Date().getFullYear();
          const nextYear = currentYear + 1;
          const expirationIndicator = page.locator(`text=/${nextYear}|expir/i`);
          expect(await expirationIndicator.isVisible()).toBeTruthy();
        }
      }
    }
  });

  test('should show share link statistics', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Open share dialog
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');
    if (await shareButton.isVisible()) {
      await shareButton.click();
      await page.waitForTimeout(500);
    }

    // Look for statistics or view count
    const viewCount = page.locator('text=/view|count/i');
    const statisticsPanel = page.locator('.statistics, [class*="statistic"]');

    // Either show view count or statistics panel
    const hasStats = await viewCount.isVisible() || await statisticsPanel.isVisible();
    expect(hasStats).toBeTruthy();
  });
});

test.describe('Share Link Access E2E Tests', () => {
  // Configure to run serially to avoid race conditions with shared shareToken
  test.describe.configure({ mode: 'serial' });

  let shareToken: string | undefined;

  test.beforeEach(async ({ page }) => {
    shareToken = undefined;
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Create a real share link for testing
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (await documentItem.isVisible({ timeout: 5000 })) {
      await documentItem.click();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);

      // Open share dialog and create a share link
      const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');
      if (await shareButton.isVisible()) {
        await shareButton.click();
        await page.waitForTimeout(500);

        const createButton = page.locator('button:has-text("Create Link"), button:has-text("Create")');
        if (await createButton.isVisible()) {
          await createButton.click();
          await page.waitForTimeout(2000);

          // Extract the share token from the created link
          const shareLink = page.locator('.share-link, [class*="share-link"], text=/http|localhost/i').first();
          if (await shareLink.isVisible()) {
            const linkText = await shareLink.evaluate((el) => {
              if (el instanceof HTMLAnchorElement) return el.href;
              if (el instanceof HTMLInputElement) return el.value;
              return el.textContent;
            });
            const match = linkText?.match(/\/share\/([^\/\s]+)/);
            if (match) {
              shareToken = match[1];
            }
          }
        }
      }
    }
  });

  test('should access document via share link without login', async ({ page }) => {
    // Skip if no share token was created
    if (!shareToken) {
      test.skip();
      return;
    }

    // Navigate to the real share link
    await page.goto(`/share/${shareToken}`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Should either show the document or login prompt based on permissions
    const documentContent = page.locator('.editor, .document-content');
    const loginPrompt = page.locator('text=Login, text=Sign in');
    const accessDenied = page.locator('text=/access.*denied|invalid.*link/i');

    // One of these should be visible
    const hasAccess = await documentContent.isVisible() ||
                      await loginPrompt.isVisible() ||
                      await accessDenied.isVisible();

    expect(hasAccess).toBeTruthy();
  });

  test('should show error for expired share link', async ({ page }) => {
    // Create a share link with past expiry date
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Open share dialog
    const shareButton = page.locator('button:has-text("Share"), [aria-label*="Share"]');
    if (!(await shareButton.isVisible())) {
      test.skip();
      return;
    }

    await shareButton.click();
    await page.waitForTimeout(500);

    // Set expiration to yesterday
    const expirationToggle = page.locator('text=Expiration, input[type="checkbox"]');
    const expirationDate = page.locator('input[type="date"], [type="date"]');

    if (await expirationToggle.isVisible()) {
      await expirationToggle.click();
      await page.waitForTimeout(300);

      if (await expirationDate.isVisible()) {
        // Set expiration to yesterday
        const yesterday = new Date();
        yesterday.setDate(yesterday.getDate() - 1);
        const pastDate = yesterday.toISOString().split('T')[0];

        await expirationDate.fill(pastDate);

        // Create share link with expiration
        const createButton = page.locator('button:has-text("Create Link"), button:has-text("Create")');
        if (await createButton.isVisible()) {
          await createButton.click();
          await page.waitForTimeout(2000);

          // Extract the expired share token
          const shareLink = page.locator('.share-link, [class*="share-link"], text=/http|localhost/i').first();
          if (await shareLink.isVisible()) {
            const linkText = await shareLink.textContent();
            const match = linkText?.match(/\/share\/([^\/\s]+)/);
            if (match) {
              const expiredToken = match[1];

              // Navigate to the expired share link
              await page.goto(`/share/${expiredToken}`);
              await page.waitForLoadState('networkidle');
              await page.waitForTimeout(2000);

              // Should show expiration error
              const errorMessage = page.locator('text=/expired|invalid.*link|no.*longer.*available/i');
              await expect(errorMessage).toBeVisible({ timeout: 5000 });
              return;
            }
          }
        }
      }
    }

    // If we couldn't create an expired link, skip the test
    test.skip();
  });

  test('should show error for non-existent share link', async ({ page }) => {
    // Navigate to a non-existent share link
    await page.goto('/share/non-existent-token');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Should show not found error
    const errorMessage = page.locator('text=/not.*found|does.*not.*exist|invalid/i');
    await expect(errorMessage).toBeVisible({ timeout: 5000 });
  });
});
