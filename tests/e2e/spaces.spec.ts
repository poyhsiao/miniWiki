import { test, expect } from '@playwright/test';

test.describe('Space Organization E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display spaces list', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Check for spaces list
    const spacesList = page.locator('.spaces-list, [class*="space-list"]');
    const spaceItem = page.locator('.space-item, [class*="space-item"]');

    // Should see either list or empty state
    const hasContent = await spacesList.isVisible() || await spaceItem.first().isVisible();
    expect(hasContent).toBeTruthy();
  });

  test('should create a new space', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');

    // Find and click create space button
    const createButton = page.locator('button:has-text("Create Space"), button:has-text("New Space"), [aria-label*="Create Space"]');

    if (!(await createButton.isVisible())) {
      test.skip('Create button not available');
      return;
    }

    await createButton.click();
    await page.waitForTimeout(500);

    // Should show creation dialog
    const dialog = page.locator('[role="dialog"], .modal');

    // Fill space name
    const nameInput = page.locator('input[name="name"], input[id*="name"], input[placeholder*="Space"]');
    if (await nameInput.isVisible()) {
      const spaceName = `Test Space ${Date.now()}`;
      await nameInput.fill(spaceName);

      // Submit creation
      const submitButton = page.locator('button:has-text("Create"), button:has-text("Save")');
      await submitButton.click();

      await page.waitForTimeout(2000);

      // Verify space was created
      const newSpace = page.locator(`text=${spaceName}`);
      await expect(newSpace.first()).toBeVisible({ timeout: 10000 });
    }
  });

  test('should navigate to space detail', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Look for a space to click
    const spaceItem = page.locator('.space-item, [class*="space-item"]').first();
    
    // Assert space item is visible, fail early if not found
    await expect(spaceItem).toBeVisible({ timeout: 5000 });

    await spaceItem.click();
    await page.waitForLoadState('networkidle');
    
    // Should show space detail or documents within space
    await page.waitForTimeout(2000);
    
    // Check for breadcrumb or space header
    const breadcrumb = page.locator('[aria-label="Breadcrumb"], .breadcrumb');
    const spaceHeader = page.locator('h1, h2, .space-header');
    
    // Assert navigation succeeded - verify URL contains space identifier OR UI elements are visible
    const urlPattern = /\/spaces\/[\w-]+/;
    const hasValidUrl = urlPattern.test(page.url());
    const hasBreadcrumb = await breadcrumb.isVisible();
    const hasHeader = await spaceHeader.isVisible();
    
    expect(hasValidUrl || hasBreadcrumb || hasHeader).toBeTruthy();
    
    // Additionally verify at least one of the post-navigation elements is visible
    if (!hasValidUrl) {
      const navigationSuccess = hasBreadcrumb || hasHeader;
      expect(navigationSuccess).toBeTruthy();
    }
  });

  test('should add members to space', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Look for space with settings or members option
    const spaceItem = page.locator('.space-item').first();
    
    if (!(await spaceItem.isVisible())) {
      test.skip();
      return;
    }

    // Look for member or settings button
    const memberButton = page.locator('[aria-label*="Member"], button:has-text("Members"), .member-button');
    const settingsButton = page.locator('[aria-label*="Settings"], button:has-text("Settings")');

    if (await memberButton.isVisible()) {
      await memberButton.click();
      await page.waitForTimeout(500);
    } else if (await settingsButton.isVisible()) {
      await settingsButton.click();
      await page.waitForTimeout(500);
    } else {
      test.skip();
      return;
    }
    
    // Should show member management
    const memberDialog = page.locator('[role="dialog"], .modal');
    const addMemberButton = page.locator('button:has-text("Add Member"), button:has-text("Invite")');
    
    await expect(addMemberButton).toBeVisible({ timeout: 5000 });
    
    await addMemberButton.click();
    await page.waitForTimeout(500);
    
    // Fill in email
    const emailInput = page.locator('input[type="email"], input[name="email"]');
    await expect(emailInput).toBeVisible({ timeout: 5000 });
    
    await emailInput.fill('test@example.com');
    
    // Select role
    const roleSelect = page.locator('select, [role="combobox"]');
    if (await roleSelect.isVisible()) {
      await roleSelect.click();
      await page.waitForTimeout(300);
    }
    
    // Send invite
    const sendButton = page.locator('button:has-text("Send"), button:has-text("Invite")');
    await expect(sendButton).toBeVisible({ timeout: 5000 });
    
    await sendButton.click();
    await page.waitForTimeout(2000);
    
    // Verify member was added (check for success message or updated member list)
    const successMessage = page.locator('[role="alert"], .success, text=/invited|added/i');
    const memberList = page.locator('.member-list, [class*="member"]');

    const hasSuccessMessage = await successMessage.isVisible();
    const memberListText = (await memberList.isVisible())
      ? await memberList.textContent()
      : null;
    const inviteSuccess = hasSuccessMessage || (memberListText?.includes('test@example.com') || false);
    expect(inviteSuccess).toBeTruthy();
  });

  test('should change member role', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Navigate to members section
    const memberButton = page.locator('[aria-label*="Member"], button:has-text("Members")');

    const visible = await memberButton.isVisible();
    if (!visible) {
      test.skip('Members button not present');
      return;
    }

    await memberButton.click();
    await page.waitForTimeout(500);

    // Look for a member with role dropdown
    const roleDropdown = page.locator('[role="combobox"], select').first();

    if (await roleDropdown.isVisible()) {
      await roleDropdown.click();
      await page.waitForTimeout(300);

      // Select different role
      const roleOption = page.locator('text=Editor')
        .or(page.locator('text=Viewer'))
        .or(page.locator('text=Commenter'))
        .first();

      if (await roleOption.isVisible()) {
        const selectedRoleText = await roleOption.textContent();
        expect(selectedRoleText).toBeTruthy(); // Ensure text was retrieved
        await roleOption.click();
        await page.waitForTimeout(500);

        // Verify role was changed
        await expect(roleDropdown).toContainText(selectedRoleText!, { timeout: 5000 });
      }
    }
  });

  test('should remove member from space', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Navigate to members
    const memberButton = page.locator('[aria-label*="Member"], button:has-text("Members")');

    if (await memberButton.isVisible()) {
      await memberButton.click();
      await page.waitForTimeout(500);

      // Get initial member count
      const memberList = page.locator('.member-list, [class*="member"]');
      const initialMemberCount = await memberList.locator('.member-item, [class*="member-item"]').count();

      // Look for remove button
      const removeButton = page.locator('[aria-label*="Remove"], button:has-text("Remove"), .remove-button');

      if (await removeButton.isVisible()) {
        await removeButton.first().click();
        await page.waitForTimeout(500);

        // Confirm removal if dialog appears
        const confirmButton = page.locator('button:has-text("Remove"), button:has-text("Confirm")');
        if (await confirmButton.isVisible()) {
          await confirmButton.click();
          await page.waitForTimeout(2000);

          // Verify member was removed - check member count decreased
          const newMemberCount = await memberList.locator('.member-item, [class*="member-item"]').count();
          expect(newMemberCount).toBeLessThan(initialMemberCount);
        }
      }
    }
  });

  test('should delete a space', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // First create a test space to delete
    const createButton = page.locator('button:has-text("Create Space"), button:has-text("New Space"), [aria-label*="Create Space"]');

    if (await createButton.isVisible()) {
      await createButton.click();
      await page.waitForTimeout(500);

      const nameInput = page.locator('input[name="name"], input[id*="name"], input[placeholder*="Space"]');
      if (await nameInput.isVisible()) {
        const testSpaceName = `Test Space to Delete ${Date.now()}`;
        await nameInput.fill(testSpaceName);

        const submitButton = page.locator('button:has-text("Create"), button:has-text("Save")');
        await submitButton.click();

        await page.waitForTimeout(2000);

        // Now find and delete the test space we just created
        const testSpaceItem = page.locator('.space-item').filter({ hasText: testSpaceName }).first();

        if (await testSpaceItem.isVisible()) {
          // Look for more options
          const moreButton = testSpaceItem.locator('[aria-label*="More"], [aria-label*="Options"], .more-button');

          if (await moreButton.isVisible()) {
            await moreButton.click();
            await page.waitForTimeout(500);

            // Look for delete option
            const deleteOption = page.locator('text="Delete"').or(page.locator('text="Delete Space"'));

            if (await deleteOption.isVisible()) {
              await deleteOption.click();
              await page.waitForTimeout(500);

              // Confirm deletion
              const confirmButton = page.locator('button:has-text("Delete"), button[type="submit"]');
              if (await confirmButton.isVisible()) {
                await confirmButton.click();
                await page.waitForTimeout(2000);

                // Verify the test space was deleted
                await expect(testSpaceItem).not.toBeVisible();
              }
            }
          }
        }
      }
    }
  });

  test('should organize documents in hierarchy', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Navigate to a space
    const spaceItem = page.locator('.space-item').first();
    
    if (!(await spaceItem.isVisible())) {
      test.skip();
      return;
    }

    await spaceItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Look for nested document structure
    const nestedItem = page.locator('.nested, [class*="nested"]');
    const treeView = page.locator('.tree-view, [role="tree"]');
    
    // Assert tree view or nested structure exists
    const hasTreeView = await treeView.isVisible();
    const hasNestedItems = await nestedItem.isVisible();
    
    if (hasTreeView) {
      await expect(treeView).toBeVisible();
    }
    
    if (hasNestedItems) {
      await expect(nestedItem.first()).toBeVisible();
    }
    
    // Try creating a nested document
    const createButton = page.locator('button:has-text("Create"), [aria-label*="Create"]');
    
    if (!(await createButton.isVisible())) {
      // If no create button, verify hierarchy structure exists and skip creation part
      expect(hasTreeView || hasNestedItems).toBeTruthy();
      return;
    }

    await createButton.click();
    await page.waitForTimeout(500);
    
    // Look for parent document selector
    const parentSelect = page.locator('[role="combobox"], select, input[placeholder*="Parent"]');
    const newDocumentForm = page.locator('form, [role="dialog"]');
    
    // Assert document creation form appeared
    await expect(newDocumentForm).toBeVisible({ timeout: 5000 });
    
    // If parent selector exists, verify it's visible (indicates hierarchy support)
    if (await parentSelect.isVisible()) {
      await expect(parentSelect).toBeVisible();
    }
  });
});

test.describe('Space Settings E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should access space settings', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Find settings
    const spaceItem = page.locator('.space-item').first();
    
    if (!(await spaceItem.isVisible())) {
      test.skip();
      return;
    }

    const settingsButton = page.locator('[aria-label*="Settings"], button:has-text("Settings")');
    
    if (!(await settingsButton.isVisible())) {
      test.skip();
      return;
    }

    await settingsButton.click();
    await page.waitForTimeout(500);
    
    // Should show settings panel or dialog
    const settingsPanel = page.locator('.settings-panel, [role="dialog"]');
    
    // Assert settings panel is visible
    await expect(settingsPanel).toBeVisible({ timeout: 5000 });
    
    // Verify expected content inside the panel
    const settingsHeading = settingsPanel.locator('text=/Settings|Configure|Options/i');
    await expect(settingsHeading).toBeVisible({ timeout: 5000 });
  });

  test('should update space name', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Navigate to settings
    const settingsButton = page.locator('[aria-label*="Settings"], button:has-text("Settings")');
    
    if (await settingsButton.isVisible()) {
      await settingsButton.click();
      await page.waitForTimeout(500);
      
      // Find name input
      const nameInput = page.locator('input[name="name"], input[id*="name"]');
      
      if (await nameInput.isVisible()) {
        // Update name using platform-agnostic fill method
        const newName = `Updated Space ${Date.now()}`;
        await nameInput.fill(newName);

        // Save
        const saveButton = page.locator('button:has-text("Save"), button[type="submit"]');
        await saveButton.click();
        
        await page.waitForTimeout(2000);
        
        // Verify update persisted
        await page.reload();
        await page.waitForLoadState('networkidle');
        
        // Re-open settings panel after reload
        await settingsButton.click();
        await page.waitForTimeout(500);
        
        await expect(nameInput).toHaveValue(newName);
      }
    }
  });

  test('should toggle space visibility', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Navigate to settings
    const settingsButton = page.locator('[aria-label*="Settings"], button:has-text("Settings")');
    
    if (await settingsButton.isVisible()) {
      await settingsButton.click();
      await page.waitForTimeout(500);
      
      // Look for visibility toggle
      const checkboxToggle = page.locator('input[type="checkbox"][name*="public"]');
      const switchToggle = page.locator('.toggle, [role="switch"]');

      const getToggleState = async () => {
        if (await checkboxToggle.isVisible()) return await checkboxToggle.isChecked();
        const ariaChecked = await switchToggle.getAttribute('aria-checked');
        return ariaChecked === 'true';
      };
      
      if (await checkboxToggle.isVisible() || await switchToggle.isVisible()) {
        // Get initial state
        const initialState = await getToggleState();
        
        // Toggle
        await (await checkboxToggle.isVisible() ? checkboxToggle : switchToggle).click();
        await page.waitForTimeout(500);
        
        // Save changes
        const saveButton = page.locator('button:has-text("Save")');
        await saveButton.click();
        
        await page.waitForTimeout(2000);
        
        // Verify toggle persisted
        await page.reload();
        await page.waitForLoadState('networkidle');

        // Re-open settings panel after reload
        await settingsButton.click();
        await page.waitForTimeout(500);

        // Re-query the toggle after reload
        const persistedCheckbox = page.locator('input[type="checkbox"][name*="public"]');
        const persistedSwitch = page.locator('.toggle, [role="switch"]');
        await expect(persistedCheckbox.or(persistedSwitch)).toBeVisible({ timeout: 5000 });

        // Verify state is flipped using re-queried locators
        const finalState = await (async () => {
          if (await persistedCheckbox.isVisible()) return await persistedCheckbox.isChecked();
          const ariaChecked = await persistedSwitch.getAttribute('aria-checked');
          return ariaChecked === 'true';
        })();
        expect(finalState).not.toBe(initialState);
      }
    }
  });
});
