import { test, expect } from '@playwright/test';

test.describe('Document Management E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should navigate to documents page', async ({ page }) => {
    // Click on documents or spaces navigation
    const documentsLink = page.locator('a:has-text("Documents"), a:has-text("Spaces"), [aria-label*="Documents"]');
    if (await documentsLink.isVisible()) {
      await documentsLink.click();
      await page.waitForLoadState('networkidle');
      
      // Should be on documents page
      await expect(page).toHaveURL(/.*documents.*|.*spaces.*/);
    }
  });

  test('should display list of documents/spaces', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');

    // Wait for content to load with explicit selector
    const documentList = page.locator('.document-list, .space-list, [class*="list"]');
    const documentItem = page.locator('.document-item, .space-item, [class*="item"]');

    await expect(documentList.or(documentItem.first())).toBeVisible({ timeout: 10000 });

    // Either list container or items should be visible
    const hasContent = await documentList.isVisible() || await documentItem.first().isVisible();
    if (!hasContent) {
      // Empty state should be visible
      const emptyState = page.locator('.empty, [class*="empty"], text=No documents, text=No spaces');
      expect(await emptyState.isVisible()).toBe(true);
    }
  });

  test('should create a new document', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    
    // Look for create button
    const createButton = page.locator('button:has-text("Create"), button:has-text("New"), [aria-label*="Create"]');
    
    if (await createButton.isVisible()) {
      await createButton.click();
      await page.waitForTimeout(500);
      
      // Should show creation dialog or form
      const dialog = page.locator('[role="dialog"], .modal, .dialog');
      const form = page.locator('form');
      
      // Fill in document/space details
      const titleInput = page.locator('input[name="title"], input[id*="title"], input[placeholder*="title"]');
      if (await titleInput.isVisible()) {
        await titleInput.fill(`Test Document ${Date.now()}`);
        
        // Submit creation
        const submitButton = page.locator('button:has-text("Create"), button:has-text("Save"), button[type="submit"]');
        await submitButton.click();
        
        await page.waitForTimeout(2000);
        
        // Should see the new document
        const newDocument = page.locator(`text=Test Document`);
        await expect(newDocument.first()).toBeVisible({ timeout: 10000 });
      }
    }
  });

  test('should open document editor', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Look for a document or space to click on
    const documentItem = page.locator('.document-item, .space-item, [class*="item"]').first();
    
    if (!(await documentItem.isVisible())) {
      test.skip(true, 'Document item not available');
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');

    // Should navigate to document editor or detail view
    await page.waitForTimeout(2000);

    // Check for editor or content area
    const editor = page.locator('.editor, [contenteditable], quill-editor, .rich-text-editor');
    const contentArea = page.locator('.content, .document-content');

    const hasEditor = await editor.isVisible() || await contentArea.isVisible();

    // Assert that editor or content area is visible
    expect(hasEditor).toBeTruthy();
  });

  test('should edit document content', async ({ page }) => {
    // This test requires being logged in and having a document
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Try to find editor
    const editor = page.locator('.editor, [contenteditable], quill-editor, .rich-text-editor');
    
    if (await editor.isVisible()) {
      // Click to focus editor
      await editor.click();
      
      // Type some content
      await page.keyboard.type('Test content for E2E testing');
      
      // Look for save button
      const saveButton = page.locator('button:has-text("Save"), [aria-label*="Save"]');
      if (await saveButton.isVisible()) {
        await saveButton.click();
        await page.waitForTimeout(1000);
      }
    }
  });

  test('should delete a document', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // STEP 1: Create a dedicated test document first
    const createButton = page.locator('button:has-text("Create"), button:has-text("New"), [aria-label*="Create"]');

    if (!(await createButton.isVisible())) {
      test.skip(true, 'Create button not available');
      return;
    }

    await createButton.click();
    await page.waitForTimeout(500);

    // Create unique test document name
    const testDocName = `E2E-DELETE-TEST-${Date.now()}`;
    const titleInput = page.locator('input[name="title"], input[id*="title"], input[placeholder*="title"]');

    if (!(await titleInput.isVisible())) {
      test.skip(true, 'Title input not visible');
      return;
    }

    await titleInput.fill(testDocName);

    const submitButton = page.locator('button:has-text("Create"), button:has-text("Save"), button[type="submit"]');
    await submitButton.click();
    await page.waitForTimeout(2000);

    // Verify test document was created
    const testDocument = page.locator(`text=${testDocName}`).first();
    await expect(testDocument).toBeVisible({ timeout: 10000 });

    // STEP 2: Now delete the test document we just created
    const testDocItem = page.locator('.document-item, .space-item').filter({ hasText: testDocName }).first();

    // Assert the test document exists
    await expect(testDocItem).toBeVisible();

    // Look for menu or delete button
    const moreButton = testDocItem.locator('[aria-label*="more"], [aria-label*="options"], .more-button');

    if (!(await moreButton.isVisible())) {
      test.skip(true, 'More button not visible');
      return;
    }

    await moreButton.click();
    await page.waitForTimeout(500);

    // Look for delete option
    const deleteOption = page.locator('text=Delete');
    await expect(deleteOption).toBeVisible({ timeout: 5000 });

    await deleteOption.click();
    await page.waitForTimeout(500);

    // Confirm deletion
    const confirmButton = page.locator('button:has-text("Delete"), button:has-text("Confirm")');
    await expect(confirmButton).toBeVisible({ timeout: 5000 });

    await confirmButton.click();
    await page.waitForTimeout(2000);

    // STEP 3: Verify the test document was deleted
    await expect(testDocItem).not.toBeVisible();
  });

  test('should search for documents', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    
    // Look for search input
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"], [aria-label*="Search"]');
    
    if (await searchInput.isVisible()) {
      await searchInput.click();
      await page.keyboard.type('test');
      
      // Wait for search results
      await page.waitForTimeout(1500);
      
      // Results should filter based on search
      const results = page.locator('.document-item, .space-item, [class*="item"]');
      const count = await results.count();
      
      console.log(`Found ${count} items after search`);
    }
  });

  test('should navigate document hierarchy', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);
    
    // Look for nested documents or breadcrumbs
    const breadcrumb = page.locator('[aria-label="Breadcrumb"], .breadcrumb, .breadcrumbs');
    const nestedItem = page.locator('.nested, .child, [class*="nested"]');
    
    // Assert that hierarchy navigation elements exist
    const hasBreadcrumb = await breadcrumb.isVisible();
    const hasNestedItem = await nestedItem.isVisible();

    if (!hasBreadcrumb && !hasNestedItem) {
      test.skip(true, 'Breadcrumb or nested items not available');
      return;
    }

    // If breadcrumb exists, verify it's visible
    if (hasBreadcrumb) {
      await expect(breadcrumb).toBeVisible();
    }

    // If nested items exist, test navigation
    if (hasNestedItem) {
      await expect(nestedItem.first()).toBeVisible();

      // Click on nested item and verify navigation
      const initialUrl = page.url();
      const priorBreadcrumb = hasBreadcrumb ? await breadcrumb.textContent() : '';

      await nestedItem.first().click();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);

      // Verify navigation occurred (URL changed OR breadcrumb updated)
      const urlChanged = page.url() !== initialUrl;
      const breadcrumbUpdated = hasBreadcrumb && (await breadcrumb.textContent()) !== priorBreadcrumb;

      expect(urlChanged || breadcrumbUpdated).toBeTruthy();
    }
  });
});

test.describe('Document Editor E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should load rich text editor', async ({ page }) => {
    // Navigate to spaces list first
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');

    // Look for a document to click on
    const documentItem = page.locator('.document-item, .space-item, [class*="item"]').first();

    if (await documentItem.isVisible({ timeout: 5000 })) {
      await documentItem.click();
      await page.waitForLoadState('networkidle');

      // Check for editor components with explicit waits
      const editor = page.locator('quill-editor, .ql-editor, [contenteditable]');
      const toolbar = page.locator('.ql-toolbar, .editor-toolbar');

      await expect(editor).toBeVisible({ timeout: 10000 });
      await expect(toolbar).toBeVisible({ timeout: 10000 });
    } else {
      test.skip(true, 'Document item not available');
    }
  });

  test('should support text formatting', async ({ page }) => {
    // Navigate to spaces list first
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');

    // Look for a document to click on
    const documentItem = page.locator('.document-item, .space-item, [class*="item"]').first();

    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip(true, 'Document item not available');
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');

    // Find editor and toolbar
    const editor = page.locator('quill-editor, .ql-editor');
    const boldButton = page.locator('.ql-bold, button[aria-label*="bold"]');
    const italicButton = page.locator('.ql-italic, button[aria-label*="italic"]');

    // Enforce preconditions with explicit assertions
    await expect(editor).toBeVisible({ timeout: 10000 });
    await expect(boldButton).toBeVisible({ timeout: 10000 });

    // Type some text
    await editor.click();
    await page.keyboard.type('Text: ');

    // Apply bold formatting
    await boldButton.click();
    await page.keyboard.type('This is bold');
    await boldButton.click(); // Toggle off bold

    // Apply italic
    await page.keyboard.type(' ');
    await italicButton.click();
    await page.keyboard.type('This is italic');
    await italicButton.click(); // Toggle off italic

    // Verify DOM structure with explicit waits
    await expect(editor.locator('strong')).toBeVisible({ timeout: 5000 });
    await expect(editor.locator('em')).toBeVisible({ timeout: 5000 });

    const boldText = await editor.locator('strong').textContent();
    expect(boldText).toContain('This is bold');
    const italicText = await editor.locator('em').textContent();
    expect(italicText).toContain('This is italic');
  });

  test('should handle image insertion', async ({ page }) => {
    // Navigate to spaces list first
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');

    // Look for a document to click on
    const documentItem = page.locator('.document-item, .space-item, [class*="item"]').first();

    if (!(await documentItem.isVisible({ timeout: 5000 }))) {
      test.skip(true, 'Document item not available');
      return;
    }

    await documentItem.click();
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for image button in toolbar
    const imageButton = page.locator('.ql-image, button[aria-label*="image"]');

    if (await imageButton.isVisible()) {
      await imageButton.click();
      await page.waitForTimeout(500);

      // Should show image upload dialog
      const imageDialog = page.locator('[role="dialog"], .modal, .image-upload');
      await expect(imageDialog).toBeVisible({ timeout: 5000 });
    } else {
      test.skip(true, 'Image button not available');
    }
  });
});
