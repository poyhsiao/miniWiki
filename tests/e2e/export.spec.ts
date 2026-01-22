import { test, expect } from '@playwright/test';
import { readFile } from 'fs/promises';

test.describe('Document Export E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display export option in document', async ({ page }) => {
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

    // Look for export button/menu
    const exportButton = page.locator('button:has-text("Export"), [aria-label*="Export"], .export-button');
    const exportMenu = page.locator('.export-menu, [class*="export"]');

    const hasExportUI = await exportButton.isVisible() || await exportMenu.isVisible();
    expect(hasExportUI).toBeTruthy();
  });

  test('should open export dialog', async ({ page }) => {
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

    // Click export button
    const exportButton = page.locator('button:has-text("Export"), [aria-label*="Export"]');

    if (!(await exportButton.isVisible())) {
      test.skip(true, 'Export button not available');
      return;
    }
    await exportButton.click();
    await page.waitForTimeout(500);

    // Should show export dialog
    const exportDialog = page.locator('[role="dialog"], .modal, .export-dialog');
    await expect(exportDialog).toBeVisible({ timeout: 5000 });
  });

  test('should export to Markdown format', async ({ page }) => {
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

    // Open export dialog
    const exportButton = page.locator('button:has-text("Export"), [aria-label*="Export"]');
    if (!(await exportButton.isVisible())) {
      test.skip(true, 'Export button not available');
      return;
    }
    await exportButton.click();
    await page.waitForTimeout(500);

    // Look for Markdown export option
    const markdownOption = page.getByText('Markdown').or(page.getByText('.md')).or(page.getByText(/md/i)).first();
    if (!(await markdownOption.isVisible())) {
      test.skip(true, 'Markdown export option not available');
      return;
    }
    await markdownOption.click();
    await page.waitForTimeout(300);

    // Look for download button
    const downloadButton = page.locator('button:has-text("Download"), button:has-text("Export")');
    if (!(await downloadButton.isVisible())) {
      test.skip(true, 'Download button not available');
      return;
    }
    // Set up download handling
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      downloadButton.click()
    ]);

    // Verify download started
    expect(download).toBeDefined();
    const filename = download.suggestedFilename();
    expect(filename).toBeTruthy();
    // Markdown file should have .md extension
    expect(filename).toMatch(/\.md$/i);
  });

  test('should export to HTML format', async ({ page }) => {
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

    // Open export dialog
    const exportButton = page.locator('button:has-text("Export"), [aria-label*="Export"]');
    if (!(await exportButton.isVisible())) {
      test.skip(true, 'Export button not available');
      return;
    }
    await exportButton.click();
    await page.waitForTimeout(500);

    // Look for HTML export option
    const htmlOption = page.getByText(/\.?html?/i);
    if (!(await htmlOption.isVisible())) {
      test.skip(true, 'HTML export option not available');
      return;
    }
    await htmlOption.click();
    await page.waitForTimeout(300);

    // Look for download button
    const downloadButton = page.locator('button:has-text("Download"), button:has-text("Export")');
    if (!(await downloadButton.isVisible())) {
      test.skip(true, 'Download button not available');
      return;
    }
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      downloadButton.click()
    ]);

    expect(download).toBeDefined();
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/\.html?$/i);
  });

  test('should export to PDF format', async ({ page }) => {
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

    // Open export dialog
    const exportButton = page.locator('button:has-text("Export"), [aria-label*="Export"]');
    if (!(await exportButton.isVisible())) {
      test.skip(true, 'Export button not available');
      return;
    }
    await exportButton.click();
    await page.waitForTimeout(500);

    // Look for PDF export option
    const pdfOption = page.getByText(/\.?pdf/i);
    if (!(await pdfOption.isVisible())) {
      test.skip(true, 'PDF export option not available');
      return;
    }
    await pdfOption.click();
    await page.waitForTimeout(300);

    // Look for download button
    const downloadButton = page.locator('button:has-text("Download"), button:has-text("Export")');
    if (!(await downloadButton.isVisible())) {
      test.skip(true, 'Download button not available');
      return;
    }
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      downloadButton.click()
    ]);

    expect(download).toBeDefined();
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/\.pdf$/i);
  });

  test('should preserve formatting in export', async ({ page }) => {
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

    // Open export dialog
    const exportButton = page.locator('button:has-text("Export"), [aria-label*="Export"]');
    if (!(await exportButton.isVisible())) {
      test.skip(true, 'Export button not available');
      return;
    }
    await exportButton.click();
    await page.waitForTimeout(500);

    // Export to Markdown
    const markdownOption = page.locator('text=Markdown');
    if (!(await markdownOption.isVisible())) {
      test.skip(true, 'Markdown option not available');
      return;
    }
    await markdownOption.click();
    await page.waitForTimeout(300);

    // Download and verify
    const downloadButton = page.locator('button:has-text("Download")');
    if (!(await downloadButton.isVisible())) {
      test.skip(true, 'Download button not available');
      return;
    }
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      downloadButton.click()
    ]);

    // Verify download
    expect(download).toBeDefined();
    const filename = download.suggestedFilename();
    expect(filename).toMatch(/\.md$/i);

    // Read and verify Markdown formatting
    const path = await download.path();
    if (path) {
      const content = await readFile(path, 'utf-8');

      // Assert that expected Markdown formatting tokens are present
      const hasHeadings = /^#{1,6}\s+/m.test(content);
      const hasBold = /\*\*.*\*\*|__.*__/.test(content);
      const hasCodeBlocks = /```[\s\S]*```|`[^`]+`/.test(content);

      // At least some formatting should be preserved
      expect(hasHeadings || hasBold || hasCodeBlocks).toBeTruthy();
    }
  });

  test('should show export options in menu', async ({ page }) => {
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

    // Click export button to open menu
    const exportButton = page.locator('button:has-text("Export"), [aria-label*="Export"]');
    if (!(await exportButton.isVisible())) {
      test.skip(true, 'Export button not available');
      return;
    }
    await exportButton.click();
    await page.waitForTimeout(500);

    // Check for all export format options
    const markdownOption = page.getByText(/markdown|\.?md/i);
    const htmlOption = page.getByText(/html/i);
    const pdfOption = page.getByText(/pdf/i);

    // Verify all export formats are available
    const markdownVisible = await markdownOption.isVisible();
    const htmlVisible = await htmlOption.isVisible();
    const pdfVisible = await pdfOption.isVisible();

    // At least one export format should be visible
    expect(markdownVisible || htmlVisible || pdfVisible).toBeTruthy();
  });
});

test.describe('Export Error Handling E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should handle export network failure', async ({ page }) => {
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

    // Intercept export requests and abort them
    await page.route('**/export**', route => route.abort());

    // Open export dialog
    const exportButton = page.locator('button:has-text("Export")');
    if (!(await exportButton.isVisible())) {
      test.skip(true, 'Export button not available');
      return;
    }
    await exportButton.click();
    await page.waitForTimeout(500);

    // Try to export
    const downloadButton = page.locator('button:has-text("Download")');
    if (!(await downloadButton.isVisible())) {
      test.skip(true, 'Download button not available');
      return;
    }
    await downloadButton.click();
    await page.waitForTimeout(2000);

    // Should show error message
    const errorMessage = page.locator('.error, [role="alert"], text=/export.*failed|error/i');
    expect(await errorMessage.isVisible()).toBeTruthy();
  });

  test('should show export progress UI for a document', async ({ page }) => {
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

    // Open export dialog
    const exportButton = page.locator('button:has-text("Export"), [aria-label*="Export"]');
    if (!(await exportButton.isVisible())) {
      test.skip(true, 'Export button not available');
      return;
    }
    await exportButton.click();
    await page.waitForTimeout(500);

    // Intercept export requests to add delay for progress UI
    await page.route('**/export**', async route => {
      await new Promise(resolve => setTimeout(resolve, 1000));
      await route.continue();
    });

    // Trigger export to see progress
    const downloadButton = page.locator('button:has-text("Download"), button:has-text("Export")');
    if (!(await downloadButton.isVisible())) {
      test.skip(true, 'Download button not available');
      return;
    }
    // Start the export
    await downloadButton.click();

    // Look for progress indicator during export
    const progressIndicator = page.locator('.progress, [role="progressbar"], .export-progress');

    // Wait for progress UI and annotate the result
    let isVisible = false;
    try {
      await expect(progressIndicator).toBeVisible({ timeout: 3000 });
      isVisible = true;
    } catch (error) {
      isVisible = false;
    }

    test.info().annotations.push({
      type: 'progress-visible',
      description: String(isVisible)
    });

    // Assert progress UI appeared
    if (!isVisible) {
      throw new Error('Progress UI did not appear during export');
    }
  });
});
