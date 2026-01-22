import { test, expect } from '@playwright/test';

test.describe('Version History E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display version history option', async ({ page }) => {
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

    // Look for version history button/menu
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"], .version-button');
    const historyMenu = page.locator('.version-history, [class*="history"]');

    const hasVersionUI = await versionButton.isVisible() || await historyMenu.isVisible();
    expect(hasVersionUI).toBeTruthy();
  });

  test('should open version history panel', async ({ page }) => {
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

    // Click version history button
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"]');

    // Skip if version button is not visible
    if (!(await versionButton.isVisible())) {
      test.skip();
      return;
    }

    await versionButton.click();
    await page.waitForTimeout(500);

    // Should show version history panel
    const versionPanel = page.locator('.version-panel, [role="dialog"], .history-panel');
    await expect(versionPanel).toBeVisible({ timeout: 5000 });
  });

  test('should display list of versions', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('.document-item').first()).toBeVisible({ timeout: 5000 });

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('button:has-text("Version"), [aria-label*="Version"]')).toBeVisible({ timeout: 5000 });

    // Open version history
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"]');
    if (!(await versionButton.isVisible())) {
      test.skip();
      return;
    }

    await versionButton.click();
    await expect(page.locator('.version-panel, [role="dialog"], .history-panel')).toBeVisible({ timeout: 5000 });

    // Look for version list
    const versionList = page.locator('.version-list, [class*="version-list"]');
    const versionItem = page.locator('.version-item, [class*="version-item"]');

    // Either show version list or empty state
    await expect(versionList.or(versionItem.first())).toBeVisible({ timeout: 5000 });
  });

  test('should show version details', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('.document-item').first()).toBeVisible({ timeout: 5000 });

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('button:has-text("Version"), [aria-label*="Version"]')).toBeVisible({ timeout: 5000 });

    // Open version history
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"]');
    if (!(await versionButton.isVisible())) {
      test.skip();
      return;
    }
    await versionButton.click();
    await expect(page.locator('.version-panel, [role="dialog"], .history-panel')).toBeVisible({ timeout: 5000 });

    // Click on a version to see details
    const versionItem = page.locator('.version-item, [class*="version-item"]').first();

    // Check if version item is visible - skip if not
    if (!(await versionItem.isVisible())) {
      test.skip();
      return;
    }

    await versionItem.click();
    await expect(page.locator('.version-info, [class*="version-info"]')).toBeVisible({ timeout: 5000 });

    // Check for version details
    const versionInfo = page.locator('.version-info, [class*="version-info"]');
    const timestamp = page.locator('text=/\\d{4}|ago|hours|days/i');
    const author = page.locator('[class*="author"], text=/by/i');

    // Version info should be visible
    const hasDetails = await versionInfo.isVisible() ||
                       await timestamp.isVisible() ||
                       await author.isVisible();

    expect(hasDetails).toBeTruthy();
  });

  test('should restore previous version', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('.document-item').first()).toBeVisible({ timeout: 5000 });

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('button:has-text("Version"), [aria-label*="Version"]')).toBeVisible({ timeout: 5000 });

    // Open version history
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"]');
    if (!(await versionButton.isVisible())) {
      test.skip();
      return;
    }
    await versionButton.click();
    await expect(page.locator('.version-panel, [role="dialog"], .history-panel')).toBeVisible({ timeout: 5000 });

    // Look for restore button
    const restoreButton = page.locator('button:has-text("Restore"), [aria-label*="Restore"]');

    if (!(await restoreButton.isVisible())) {
      test.skip();
      return;
    }

    await restoreButton.click();
    await page.waitForLoadState('networkidle');

    // Should show confirmation dialog
    const confirmDialog = page.locator('[role="dialog"], .modal');
    const confirmButton = page.locator('button:has-text("Restore"), button:has-text("Confirm")');

    if (!(await confirmDialog.isVisible() || await confirmButton.isVisible())) {
      test.skip();
      return;
    }

    // Confirm restoration
    await confirmButton.click();
    await page.waitForLoadState('networkidle');

    // Verify restoration - should see success message or updated content
    const successMessage = page.locator('[role="alert"], .success:has-text("restore")');
    const restoredIndicator = page.locator('text=/restored|version.*updated/i');

    expect(await successMessage.isVisible() || await restoredIndicator.isVisible()).toBeTruthy();
  });

  test('should compare two versions', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('.document-item').first()).toBeVisible({ timeout: 5000 });

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('button:has-text("Version"), [aria-label*="Version"]')).toBeVisible({ timeout: 5000 });

    // Open version history
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"]');
    if (!(await versionButton.isVisible())) {
      test.skip();
      return;
    }
    await versionButton.click();
    await expect(page.locator('.version-panel, [role="dialog"], .history-panel')).toBeVisible({ timeout: 5000 });

    // Look for compare button or multi-select
    const compareButton = page.locator('button:has-text("Compare"), [aria-label*="Compare"]');
    const versionPanel = page.locator('.version-panel, [role="dialog"], .history-panel');
    const hasVisibleCheckbox = await versionPanel.locator('input[type="checkbox"]').isVisible();
    const hasCompareUI = await compareButton.isVisible() || hasVisibleCheckbox;

    if (hasCompareUI) {
      // If compare button exists, test comparison
      if (await compareButton.isVisible()) {
        await compareButton.click();
        await expect(page.locator('.diff-view, [class*="diff"], .comparison')).toBeVisible({ timeout: 5000 });
      }
    }

    expect(hasCompareUI).toBeTruthy();
  });

  test('should display version content preview', async ({ page }) => {
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

    // Open version history
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"]');
    if (!(await versionButton.isVisible())) {
      test.skip();
      return;
    }
    await versionButton.click();
    await page.waitForTimeout(500);

    // Click on a version to preview
    const versionItem = page.locator('.version-item, [class*="version-item"]').first();

    // Check if version item is visible - skip if not
    if (!(await versionItem.isVisible())) {
      test.info().annotations.push({
        type: 'skip',
        description: 'no version items to inspect'
      });
      test.skip();
      return;
    }

    await versionItem.click();
    await page.waitForTimeout(500);

    // Look for preview pane or content display
    const previewPane = page.locator('.preview, [class*="preview"]');
    const contentArea = page.locator('.content, .document-content');

    // Either preview or content should be visible after clicking
    await expect(previewPane.or(contentArea)).toBeVisible({ timeout: 5000 });
  });
});

test.describe('Version History Auto-Creation E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should auto-create version on significant changes', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('.document-item').first()).toBeVisible({ timeout: 5000 });

    const documentItem = page.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('button:has-text("Version"), [aria-label*="Version"]')).toBeVisible({ timeout: 5000 });

    // Open version history to get initial count
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"]');
    if (!(await versionButton.isVisible())) {
      test.skip();
      return;
    }
    await versionButton.click();
    await expect(page.locator('.version-panel, [role="dialog"], .history-panel')).toBeVisible({ timeout: 5000 });

    // Wait for version panel to be visible
    const versionPanel = page.locator('.version-panel, [role="dialog"], .history-panel');
    await versionPanel.waitFor({ state: 'visible', timeout: 5000 });

    // Get initial version count
    const initialVersions = await page.locator('.version-item, [class*="version-item"]').count();

    // Close version panel
    const closeButton = page.locator('[aria-label*="Close"], button:has-text("Close")');
    if (await closeButton.isVisible()) {
      await closeButton.click();
      await expect(versionPanel).toBeHidden({ timeout: 5000 });
    }

    // Make significant changes to document
    const editor = page.locator('.editor, [contenteditable], quill-editor');

    // Assert editor is visible before editing
    expect(await editor.isVisible()).toBeTruthy();

    await editor.click();

    // Type significant content in one call
    await page.keyboard.type('This is significant change number 1. This is significant change number 2. This is significant change number 3. This is significant change number 4. This is significant change number 5. ');

    // Save changes
    const saveButton = page.locator('button:has-text("Save")');
    if (await saveButton.isVisible()) {
      await saveButton.click();
      await page.waitForLoadState('networkidle');
    }

    // Reopen version history
    if (await versionButton.isVisible()) {
      await versionButton.click();
      await expect(versionPanel).toBeVisible({ timeout: 5000 });
    }

    // Check if new version was created
    const newVersions = await page.locator('.version-item, [class*="version-item"]').count();
    expect(newVersions).toBeGreaterThan(initialVersions);
  });

  test('should limit version history size', async ({ page }) => {
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

    // Open version history
    const versionButton = page.locator('button:has-text("Version"), [aria-label*="Version"]');
    if (!(await versionButton.isVisible())) {
      test.skip();
      return;
    }
    await versionButton.click();
    await page.waitForTimeout(500);

    // Check for version limit setting or indicator
    const limitIndicator = page.locator('text=/\\d+ versions|maximum.*version|version.*limit/i');
    const settingsButton = page.locator('button:has-text("Settings"), [aria-label*="version"]');

    // Either limit indicator exists or settings for it exist
    const hasLimitUI = await limitIndicator.isVisible() || await settingsButton.isVisible();
    expect(hasLimitUI).toBeTruthy();
  });
});
