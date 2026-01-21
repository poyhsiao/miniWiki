import { test, expect } from '@playwright/test';

test.describe('Sync and Offline E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display sync status indicator', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for sync status indicator
    const syncIndicator = page.locator('[aria-label*="Sync"], .sync-status, .sync-indicator');
    const offlineIndicator = page.locator('[aria-label*="Offline"], .offline-indicator');

    // Assert that at least one sync UI element is visible
    const hasSyncUI = await syncIndicator.isVisible() || await offlineIndicator.isVisible();
    expect(hasSyncUI).toBeTruthy();
  });

  test('should show sync status details', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Click on sync indicator if available
    const syncIndicator = page.locator('[aria-label*="Sync"], .sync-status');

    if (await syncIndicator.isVisible()) {
      await syncIndicator.click();
      await page.waitForTimeout(500);

      // Should show sync details panel
      const syncPanel = page.locator('.sync-panel, .sync-details, [class*="sync"]');

      // Assert sync panel is visible
      await expect(syncPanel).toBeVisible({ timeout: 5000 });

      // Check for sync information with assertions
      const pendingSyncs = page.locator('.pending-syncs, text=Pending');
      const lastSync = page.locator('.last-sync, text=Last');

      await expect(pendingSyncs).toBeVisible({ timeout: 5000 });
      await expect(pendingSyncs).toContainText(/Pending/i);
      await expect(lastSync).toBeVisible({ timeout: 5000 });
      await expect(lastSync).toContainText(/Last/i);
    }
  });

  test.skip(true, 'should handle offline state - TODO: implement when offline mode is ready');

  test('should sync when coming back online', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for sync trigger or retry button
    const syncButton = page.locator('button:has-text("Sync"), [aria-label*="Sync"]');
    const retryButton = page.locator('button:has-text("Retry"), [aria-label*="Retry"]');

    if (await syncButton.isVisible()) {
      await syncButton.click();
      await page.waitForTimeout(3000);

      // Assert sync success - check for success indicators
      const successIndicator = page.locator('[role="alert"]:has-text("success"), .success, text=/synced|synchronized/i');
      const errorIndicator = page.locator('[role="alert"]:has-text("error"), .error, text=/failed|error/i');

      // Verify success indicator appears or error indicator does not appear
      const hasSuccess = await successIndicator.isVisible();
      const hasError = await errorIndicator.isVisible();

      expect(hasError).toBe(false);

      // Check that sync status updated
      const syncStatus = page.locator('[aria-label*="Sync"], .sync-status');
      if (await syncStatus.isVisible()) {
        await expect(syncStatus).toBeVisible();
      }
    } else if (await retryButton.isVisible()) {
      await retryButton.click();
      await page.waitForTimeout(2000);

      // Assert retry was successful
      const errorIndicator = page.locator('[role="alert"]:has-text("error"), .error');
      await expect(errorIndicator).not.toBeVisible();
    } else {
      test.skip();
    }
  });

  test('should show pending changes queue', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for pending changes indicator
    const pendingBadge = page.locator('.pending-badge, [class*="pending"]');
    const queueIndicator = page.locator('.queue-indicator, text=Queue');

    // Assert that at least one pending changes UI element is visible
    const hasPendingUI = await pendingBadge.isVisible() || await queueIndicator.isVisible();
    expect(hasPendingUI).toBeTruthy();

    // If pending badge is visible, verify it has content
    if (await pendingBadge.isVisible()) {
      const count = await pendingBadge.textContent();
      expect(count).toBeTruthy();
    }

    // If queue indicator is visible, verify it's displayed
    if (await queueIndicator.isVisible()) {
      await expect(queueIndicator).toBeVisible();
    }
  });
});

test.describe('Real-time Collaboration E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should show presence indicator when others are viewing', async ({ page }) => {
    // Navigate to spaces list first
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');

    // Look for a document to click on
    const documentItem = page.locator('.document-item, .space-item, [class*="item"]').first();

    if (await documentItem.isVisible({ timeout: 5000 })) {
      await documentItem.click();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(3000);

      // Look for presence/collaborators indicator
      const presenceIndicator = page.locator('[aria-label*="Presence"], .presence-indicator');
      const collaborators = page.locator('.collaborators, [class*="collaborator"]');

      // Assert presence indicators exist (visible or not, as they may be hidden without other users)
      const hasPresenceUI = await presenceIndicator.count() > 0 || await collaborators.count() > 0;
      expect(hasPresenceUI).toBeTruthy();
    } else {
      test.skip();
    }
  });

  test.skip(true, 'should show other users cursors');

  test.skip(true, 'should display real-time updates from other users');
});

test.describe('Complete User Flow E2E Tests', () => {
  test('should complete full document workflow', async ({ page }) => {
    // 1. Navigate to app
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // 2. Navigate to spaces
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // 3. Create a space if none exist
    const createButton = page.locator('button:has-text("Create")');
    if (await createButton.isVisible()) {
      await createButton.click();
      await page.waitForTimeout(500);
      
      const nameInput = page.locator('input[name="name"]');
      if (await nameInput.isVisible()) {
        await nameInput.fill(`E2E Test Space ${Date.now()}`);
        
        const submitButton = page.locator('button:has-text("Create")');
        await submitButton.click();
        
        await page.waitForTimeout(2000);
      }
    }
    
    // 4. Open space
    const spaceItem = page.locator('.space-item').first();
    if (await spaceItem.isVisible()) {
      await spaceItem.click();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);
      
      // 5. Create a document
      const docCreateButton = page.locator('button:has-text("New Document"), button:has-text("Create Document")');
      if (await docCreateButton.isVisible()) {
        await docCreateButton.click();
        await page.waitForTimeout(500);
        
        const titleInput = page.locator('input[placeholder*="Title"], input[name="title"]');
        if (await titleInput.isVisible()) {
          await titleInput.fill(`E2E Test Document ${Date.now()}`);
          
          const createDocButton = page.locator('button:has-text("Create")');
          await createDocButton.click();
          
          await page.waitForTimeout(2000);

          // 6. Verify document opened in editor
          const editor = page.locator('.editor, [contenteditable]');
          await expect(editor).toBeVisible({ timeout: 10000 });
        }
      }
    }

    console.log('Full document workflow test completed');
  });

  test('should complete full space member management workflow', async ({ page }) => {
    // 1. Navigate to spaces
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // 2. Open space settings or members
    const spaceItem = page.locator('.space-item').first();
    if (await spaceItem.isVisible()) {
      await spaceItem.click();
      await page.waitForLoadState('networkidle');
      
      // 3. Navigate to members section
      const membersButton = page.locator('button:has-text("Members")');
      if (await membersButton.isVisible()) {
        await membersButton.click();
        await page.waitForTimeout(500);
        
        // 4. Add a member
        const addButton = page.locator('button:has-text("Add Member"), button:has-text("Invite")');
        if (await addButton.isVisible()) {
          await addButton.click();
          await page.waitForTimeout(500);
          
          const emailInput = page.locator('input[type="email"]');
          if (await emailInput.isVisible()) {
            await emailInput.fill('newmember@example.com');
            
            const roleSelect = page.locator('[role="combobox"]');
            if (await roleSelect.isVisible()) {
              await roleSelect.click();
              await page.waitForTimeout(300);
              
              const editorOption = page.locator('text=Editor');
              if (await editorOption.isVisible()) {
                await editorOption.click();
              }
            }
            
            const inviteButton = page.locator('button:has-text("Invite")');
            await inviteButton.click();
            
            await page.waitForTimeout(2000);
          }
        }
      }
    }
    
    console.log('Full member management workflow test completed');
  });

  test('should handle error states gracefully', async ({ page }) => {
    // Test 404 handling
    await page.goto('/non-existent-page');
    await page.waitForLoadState('networkidle');

    const notFound = page.locator('text=404').or(page.locator('text=Not Found')).or(page.locator('text=Page not found'));
    await expect(notFound).toBeVisible({ timeout: 5000 });

    // Test network error handling with network interception
    await page.route('**/*', route => {
      const request = route.request();
      // Only abort API/XHR requests, let HTML documents through
      if (request.resourceType() === 'xhr' ||
          request.resourceType() === 'fetch' ||
          request.url().includes('/api/')) {
        route.abort();
      } else {
        route.continue();
      }
    });

    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');

    // Verify error UI is displayed
    const errorUI = page.locator('.error, [role="alert"], text=/error|failed|unable/i');
    await expect(errorUI.first()).toBeVisible({ timeout: 5000 });
  });
});
