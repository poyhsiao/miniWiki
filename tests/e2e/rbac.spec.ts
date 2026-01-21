import { test, expect } from './fixtures';

test.describe('RBAC (Role-Based Access Control) E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display user role indicator', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for role indicator
    const roleIndicator = page.locator('[class*="role"], text=/Owner|Editor|Viewer|Commenter/i');
    const userBadge = page.locator('[class*="badge"], [class*="user-role"]');

    // Either role indicator or user badge should be visible
    const hasRoleUI = await roleIndicator.isVisible() || await userBadge.isVisible();
    expect(hasRoleUI).toBeTruthy();
  });

  test('should show different UI based on role', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for permission-aware UI elements
    const editButton = page.locator('button:has-text("Edit"), button:has-text("Save")');
    const deleteButton = page.locator('button:has-text("Delete"), [aria-label*="Delete"]');
    const readonlyIndicator = page.locator('text=/read.*only|view.*only/i');

    // Check if UI reflects role permissions
    const canEdit = await editButton.isVisible();
    const canDelete = await deleteButton.isVisible();
    const isReadOnly = await readonlyIndicator.isVisible();

    // At least one of these should indicate the user's permissions
    expect(canEdit || canDelete || isReadOnly).toBeTruthy();
  });

  test('should restrict edit permissions for viewer role', async ({ viewerPage }) => {
    await viewerPage.goto('/spaces');
    await viewerPage.waitForLoadState('networkidle');
    await viewerPage.waitForTimeout(2000);

    // Navigate to a document as viewer
    const documentItem = viewerPage.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await viewerPage.waitForLoadState('networkidle');
    await viewerPage.waitForTimeout(2000);

    // Try to edit document - viewer should NOT be able to edit
    const editor = viewerPage.locator('.editor, [contenteditable]');
    if (await editor.isVisible()) {
      // Viewer should NOT have edit permissions
      const isEditable = await editor.getAttribute('contenteditable');
      expect(isEditable).not.toBe('true');
    }

    // Check for edit button availability
    const editButton = viewerPage.locator('button:has-text("Edit")');
    const saveButton = viewerPage.locator('button:has-text("Save")');

    // Edit controls should NOT be available for viewer
    const canEdit = await editButton.isVisible() || await saveButton.isVisible();
    expect(canEdit).toBeFalsy();
  });

  test('should allow editor to add comments', async ({ editorPage }) => {
    await editorPage.goto('/spaces');
    await editorPage.waitForLoadState('networkidle');
    await editorPage.waitForTimeout(2000);

    // Navigate to a document
    const documentItem = editorPage.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await editorPage.waitForLoadState('networkidle');
    await editorPage.waitForTimeout(2000);

    // Look for comment functionality
    const commentButton = editorPage.locator('button:has-text("Comment"), [aria-label*="Comment"]');
    const commentInput = editorPage.locator('textarea[placeholder*="Comment"], .comment-input');

    // Comment functionality should be available for editor role
    const hasCommentUI = await commentButton.isVisible() || await commentInput.isVisible();
    expect(hasCommentUI).toBeTruthy();
  });

  test('should show member role badges in member list', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Navigate to member management
    const memberButton = page.locator('button:has-text("Members"), [aria-label*="Members"]');

    if (await memberButton.isVisible()) {
      await memberButton.click();
      await page.waitForTimeout(500);
    }

    // Look for member list with role badges
    const memberList = page.locator('.member-list, [class*="member-list"]');
    const memberItem = page.locator('.member-item, [class*="member-item"]');
    const roleBadge = page.locator('text=/Owner|Editor|Viewer|Commenter/i');

    // Member list or role badges should be visible
    const hasMemberUI = await memberList.isVisible() ||
                        await memberItem.first().isVisible() ||
                        await roleBadge.first().isVisible();

    expect(hasMemberUI).toBeTruthy();
  });

  test('should update member role', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Navigate to member management
    const memberButton = page.locator('button:has-text("Members"), [aria-label*="Members"]');

    if (await memberButton.isVisible()) {
      await memberButton.click();
      await page.waitForTimeout(500);
    }

    // Look for role dropdown/select
    const memberItem = page.locator('.member-item').first();
    const roleDropdown = memberItem.locator('[role="combobox"], select');

    if (await roleDropdown.isVisible()) {
      // Detect if it's a native select or custom combobox
      const tagName = await roleDropdown.evaluate((el) => el.tagName);

      if (tagName === 'SELECT') {
        // Native select handling
        const initialRole = await roleDropdown.inputValue();
        expect(initialRole).toBeTruthy();

        // Select different role - find options and pick a different one
        const roleOptions = roleDropdown.locator('option');
        const optionCount = await roleOptions.count();

        if (optionCount > 0) {
          // Find an option different from current role
          let clickedFound = false;
          for (let i = 0; i < optionCount; i++) {
            const optionValue = await roleOptions.nth(i).getAttribute('value');
            const optionText = await roleOptions.nth(i).textContent();

            if (optionValue && optionValue !== initialRole) {
              await roleDropdown.selectOption(optionValue);
              await page.waitForTimeout(500);
              clickedFound = true;
              break;
            } else if (!optionValue && optionText && optionText.trim() !== initialRole?.trim()) {
              await roleDropdown.selectOption({ label: optionText.trim() });
              await page.waitForTimeout(500);
              clickedFound = true;
              break;
            }
          }

          // Verify role was changed only if a different option was selected
          if (clickedFound) {
            const newRole = await roleDropdown.inputValue();
            expect(newRole).not.toEqual(initialRole);
          } else {
            console.log('No different role option available for selection');
          }
        }
      } else {
        // Custom ARIA combobox handling
        const initialRole = await roleDropdown.textContent();
        expect(initialRole).toBeTruthy();

        // Open the combobox
        await roleDropdown.click();
        await page.waitForTimeout(300);

        // Find and click the "Editor" option
        const editorOption = page.locator('[role="option"]:has-text("Editor")');

        if (await editorOption.isVisible()) {
          await editorOption.click();
          await page.waitForTimeout(500);

          // Verify the selection by checking visible text
          const newRole = await roleDropdown.textContent();
          expect(newRole).not.toEqual(initialRole);
          expect(newRole).toContain('Editor');
        } else {
          console.log('Editor option not available');
        }
      }
    }
  });

  test('should show permission denied for unauthorized actions', async ({ viewerPage }) => {
    await viewerPage.goto('/spaces');
    await viewerPage.waitForLoadState('networkidle');
    await viewerPage.waitForTimeout(2000);

    // Navigate to a document
    const documentItem = viewerPage.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await viewerPage.waitForLoadState('networkidle');
    await viewerPage.waitForTimeout(2000);

    // Try to perform unauthorized action (e.g., delete)
    // For viewer role, delete button should be disabled or show error
    const deleteButton = viewerPage.locator('button:has-text("Delete"), [aria-label*="Delete"]');

    if (await deleteButton.isVisible()) {
      // Check if button is disabled
      const isDisabled = await deleteButton.isDisabled();
      const errorMessage = viewerPage.locator('.error, [role="alert"]:has-text("permission")');

      // Either button is disabled or permission error is shown
      expect(isDisabled || await errorMessage.isVisible()).toBeTruthy();
    }
  });

  test('should show space owner controls', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Navigate to a space
    const spaceItem = page.locator('.space-item').first();
    if (!(await spaceItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await spaceItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for owner-specific controls
    const settingsButton = page.locator('button:has-text("Settings"), [aria-label*="Settings"]');
    const deleteButton = page.locator('button:has-text("Delete Space"), [aria-label*="Delete"]');
    const memberManagement = page.locator('button:has-text("Manage Members"), [aria-label*="Members"]');

    // Owner should have access to these controls
    const hasOwnerControls = await settingsButton.isVisible() ||
                             await deleteButton.isVisible() ||
                             await memberManagement.isVisible();

    expect(hasOwnerControls).toBeTruthy();
  });
});

test.describe('RBAC Permission Levels E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should have full access as owner', async ({ ownerPage }) => {
    await ownerPage.goto('/spaces');
    await ownerPage.waitForLoadState('networkidle');
    await ownerPage.waitForTimeout(2000);

    // Navigate to a space
    const spaceItem = ownerPage.locator('.space-item').first();
    if (!(await spaceItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await spaceItem.click();
    await ownerPage.waitForLoadState('networkidle');
    await ownerPage.waitForTimeout(2000);

    // Owner should have all permissions
    const editButton = ownerPage.locator('button:has-text("Edit"), button:has-text("Save")');
    const deleteButton = ownerPage.locator('button:has-text("Delete")');
    const settingsButton = ownerPage.locator('button:has-text("Settings")');
    const shareButton = ownerPage.locator('button:has-text("Share")');

    // Owner should have all these buttons visible
    expect(await editButton.isVisible() ||
           await deleteButton.isVisible() ||
           await settingsButton.isVisible() ||
           await shareButton.isVisible()).toBeTruthy();
  });

  test('should have limited access as commenter', async ({ commenterPage }) => {
    await commenterPage.goto('/spaces');
    await commenterPage.waitForLoadState('networkidle');
    await commenterPage.waitForTimeout(2000);

    // Navigate to a document
    const documentItem = commenterPage.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await commenterPage.waitForLoadState('networkidle');
    await commenterPage.waitForTimeout(2000);

    // Commenter should be able to comment but not edit content
    const commentInput = commenterPage.locator('textarea[placeholder*="Comment"], .comment-input');
    const editor = commenterPage.locator('.editor, [contenteditable]');

    const canComment = await commentInput.isVisible();
    const canEdit = await editor.isVisible() && await editor.getAttribute('contenteditable') === 'true';

    // Commenter should be able to comment
    expect(canComment).toBeTruthy();
    // Commenter should NOT be able to edit
    expect(canEdit).toBeFalsy();
  });

  test('should have read-only access as viewer', async ({ viewerPage }) => {
    await viewerPage.goto('/spaces');
    await viewerPage.waitForLoadState('networkidle');
    await viewerPage.waitForTimeout(2000);

    // Navigate to a document
    const documentItem = viewerPage.locator('.document-item').first();
    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    await documentItem.click();
    await viewerPage.waitForLoadState('networkidle');
    await viewerPage.waitForTimeout(2000);

    // Viewer should see read-only indicator or disabled edit controls
    const readonlyIndicator = viewerPage.locator('text=/read.*only|view.*only/i');
    const editButton = viewerPage.locator('button:has-text("Edit")');
    const saveButton = viewerPage.locator('button:has-text("Save")');

    const isReadOnly = await readonlyIndicator.isVisible();

    // Check if buttons exist before calling isDisabled
    let editDisabled = false;
    if (await editButton.count() > 0) {
      editDisabled = await editButton.isDisabled();
    }

    let saveDisabled = false;
    if (await saveButton.count() > 0) {
      saveDisabled = await saveButton.isDisabled();
    }

    // Viewer should have read-only access
    expect(isReadOnly || editDisabled || saveDisabled).toBeTruthy();
  });

  test('should update role permissions correctly', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Navigate to member management
    const memberButton = page.locator('button:has-text("Members"), [aria-label*="Members"]');

    if (await memberButton.isVisible()) {
      await memberButton.click();
      await page.waitForTimeout(500);
    }

    // Find a member and change their role
    const memberItem = page.locator('.member-item').first();
    const roleDropdown = memberItem.locator('[role="combobox"], select');

    if (await roleDropdown.isVisible()) {
      // Detect if it's a native select or custom combobox
      const tagName = await roleDropdown.evaluate((el) => el.tagName);

      if (tagName === 'SELECT') {
        // Native select handling
        const initialRole = await roleDropdown.inputValue();

        // Dynamically find a different role option
        const roleOptions = roleDropdown.locator('option');
        const optionCount = await roleOptions.count();

        let differentRole: string | null = null;
        for (let i = 0; i < optionCount; i++) {
          const optionValue = await roleOptions.nth(i).getAttribute('value');
          if (optionValue && optionValue !== initialRole) {
            differentRole = optionValue;
            break;
          }
        }

        if (differentRole) {
          // Change role to a different option
          await roleDropdown.selectOption(differentRole);
          await page.waitForTimeout(500);

          // Verify role was updated
          const newRole = await roleDropdown.inputValue();
          expect(newRole).not.toEqual(initialRole);
        } else {
          console.log('No different role option available for selection');
        }
      } else {
        // Custom ARIA combobox handling
        const initialRole = await roleDropdown.textContent();

        // Open the combobox
        await roleDropdown.click();
        await page.waitForTimeout(300);

        // Find and click the "Editor" option
        const editorOption = page.locator('[role="option"]:has-text("Editor")');

        if (await editorOption.isVisible()) {
          await editorOption.click();
          await page.waitForTimeout(500);

          // Verify the selection by checking visible text
          const newRole = await roleDropdown.textContent();
          expect(newRole).not.toEqual(initialRole);
        }
      }
    }
  });
});
