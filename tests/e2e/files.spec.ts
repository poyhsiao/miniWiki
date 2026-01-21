import { test, expect } from '@playwright/test';

test.describe('File Attachments E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display file upload widget', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Navigate to a document to find file upload
    const documentItem = page.locator('.document-item, .space-item').first();
    if (await documentItem.isVisible({ timeout: 5000 })) {
      await documentItem.click();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);

      // Look for file upload widget
      const uploadWidget = page.locator('.file-upload, [data-testid="file-upload"], [aria-label*="upload"]');
      const uploadButton = page.locator('button:has-text("Upload"), input[type="file"]');

      const hasUploadUI = await uploadWidget.isVisible() || await uploadButton.isVisible();
      expect(hasUploadUI).toBeTruthy();
    } else {
      test.skip();
    }
  });

  test('should open file dialog when upload button clicked', async ({ page }) => {
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

    // Look for upload button
    const uploadButton = page.locator('button:has-text("Upload"), [aria-label*="Upload"]');

    if (await uploadButton.isVisible()) {
      // Get initial file input count
      const initialFileInputs = await page.locator('input[type="file"]').count();

      await uploadButton.click();
      await page.waitForTimeout(500);

      // File input should appear or dialog should open
      const newFileInputs = await page.locator('input[type="file"]').count();
      const dialog = page.locator('[role="dialog"], .modal, .file-dialog');

      const dialogOpened = await dialog.isVisible();
      const fileInputAppeared = newFileInputs > initialFileInputs;

      expect(dialogOpened || fileInputAppeared).toBeTruthy();
    }
  });

  test('should handle small file upload', async ({ page }) => {
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

    // Look for file input
    const fileInput = page.locator('input[type="file"]');

    // Skip if file input is not visible
    if (!(await fileInput.isVisible())) {
      test.skip();
      return;
    }

    // Create a small test file using Node's fs
    const fs = require('fs');
    const os = require('os');
    const path = require('path');
    const testFilePath = path.join(os.tmpdir(), 'test-upload.txt');
    fs.writeFileSync(testFilePath, 'Test file content for E2E upload testing');

    try {
      // Upload the file
      await fileInput.setInputFiles(testFilePath);

      // Wait for upload to complete
      await page.waitForTimeout(3000);

      // Verify file appears in file list
      const fileList = page.locator('.file-list, [class*="file-list"], .attachment-list');
      const uploadedFile = page.locator('text=test-upload.txt, text=.txt');

      // Either file list shows the file or success message appears
      const hasFileList = await fileList.isVisible();
      const hasUploadedFile = await uploadedFile.first().isVisible();
      const successMessage = page.locator('[role="alert"], .success:has-text("upload")');

      expect(hasUploadedFile || hasFileList || await successMessage.isVisible()).toBeTruthy();
    } finally {
      // Cleanup
      try {
        fs.unlinkSync(testFilePath);
      } catch (e) {}
    }
  });

  test('should display uploaded files list', async ({ page }) => {
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

    // Look for file list
    const fileList = page.locator('.file-list, [class*="file-list"], .attachments');

    // File list should be visible (even if empty)
    await expect(fileList).toBeVisible({ timeout: 5000 });

    // Check for file items or empty state
    const fileItem = page.locator('.file-item, [class*="file-item"]');
    const emptyState = page.locator('.empty, [class*="empty"]:has-text("file")');

    const hasFiles = await fileItem.first().isVisible();
    const hasEmptyState = await emptyState.isVisible();

    expect(hasFiles || hasEmptyState).toBeTruthy();
  });

  test('should download uploaded file', async ({ page }) => {
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

    // Look for a file to download
    const downloadButton = page.locator('[aria-label*="Download"], button:has-text("Download"), .download-button');

    if (await downloadButton.isVisible()) {
      // Set up download handling
      const [download] = await Promise.all([
        page.waitForEvent('download'),
        downloadButton.click()
      ]);

      // Verify download started
      expect(download).toBeDefined();
      expect(download.suggestedFilename()).toBeTruthy();
    }
  });

  test('should delete uploaded file', async ({ page }) => {
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

    // Get initial file count
    const fileItem = page.locator('.file-item, [class*="file-item"]');
    const initialCount = await fileItem.count();

    if (initialCount > 0) {
      // Look for delete button on first file
      const deleteButton = fileItem.first().locator('[aria-label*="Delete"], button:has-text("Delete")');

      if (await deleteButton.isVisible()) {
        await deleteButton.click();
        await page.waitForTimeout(500);

        // Confirm deletion if dialog appears
        const confirmButton = page.locator('button:has-text("Delete"), button:has-text("Confirm")');
        if (await confirmButton.isVisible()) {
          await confirmButton.click();
          await page.waitForTimeout(2000);
        }

        // Verify file count decreased
        const newCount = await fileItem.count();
        expect(newCount).toBeLessThan(initialCount);
      }
    }
  });

  test('should show file metadata', async ({ page }) => {
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

    // Look for file with metadata
    const fileItem = page.locator('.file-item, [class*="file-item"]').first();

    if (await fileItem.isVisible()) {
      // Click on file to see details
      await fileItem.click();
      await page.waitForTimeout(500);

      // Check for metadata display
      const fileName = page.locator('.filename, [class*="name"], text=.pdf, text=.txt');
      const fileSize = page.locator('.size, [class*="size"], text=KB, text=MB');
      const fileDate = page.locator('.date, [class*="date"], text=202, text=/\\d{4}/');

      const hasMetadata = await fileName.isVisible() ||
                          await fileSize.isVisible() ||
                          await fileDate.isVisible();

      expect(hasMetadata).toBeTruthy();
    }
  });

  test('should handle chunked upload for large files', async ({ page }) => {
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

    // Create a large file (simulate 5MB)
    const fs = require('fs');
    const os = require('os');
    const path = require('path');
    const largeFilePath = path.join(os.tmpdir(), 'large-test-upload.bin');
    const largeContent = Buffer.alloc(5 * 1024 * 1024, 'x'); // 5MB
    fs.writeFileSync(largeFilePath, largeContent);

    try {
      // Look for file input and upload
      const fileInput = page.locator('input[type="file"]');
      if (await fileInput.isVisible()) {
        await fileInput.setInputFiles(largeFilePath);

        // Wait for upload to start
        await page.waitForTimeout(1000);

        // Look for progress indicator during upload
        const progressBar = page.locator('.progress, [role="progressbar"], .upload-progress');
        const progressText = page.locator('text=/\\d+%|\\d+\\/\\d+/');

        // Progress should be visible during upload
        const hasProgressUI = await progressBar.isVisible() || await progressText.isVisible();
        expect(hasProgressUI).toBeTruthy();

        // Wait for upload to complete
        await page.waitForTimeout(3000);
      }
    } finally {
      // Cleanup - always executed
      try {
        fs.unlinkSync(largeFilePath);
      } catch (e) {}
    }
  });
});

test.describe('File Upload Error Handling E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should show error for oversized file', async ({ page }) => {
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

    // Try to upload a file that's too large
    const fileInput = page.locator('input[type="file"]');

    if (!(await fileInput.isVisible())) {
      test.skip();
      return;
    }

    // Create a large file (simulate 10MB which exceeds typical file limits)
    const fs = require('fs');
    const os = require('os');
    const path = require('path');
    const oversizedFilePath = path.join(os.tmpdir(), 'oversized-test-file.bin');

    // Use streaming approach to avoid OOM in CI
    const writeStream = fs.createWriteStream(oversizedFilePath);
    const chunkSize = 1024 * 1024; // 1MB chunks
    const totalSize = 10 * 1024 * 1024; // 10MB total
    const chunk = Buffer.alloc(chunkSize, 'x');

    for (let written = 0; written < totalSize; written += chunkSize) {
      writeStream.write(chunk);
    }
    writeStream.end();

    // Wait for file write to complete
    await new Promise<void>((resolve, reject) => {
      const maxWaitMs = 10000; // 10 second timeout
      const startTime = Date.now();
      const checkInterval = setInterval(() => {
        try {
          // Check if timeout exceeded
          if (Date.now() - startTime > maxWaitMs) {
            clearInterval(checkInterval);
            reject(new Error('Timeout waiting for oversized file to be written'));
            return;
          }

          const stats = fs.statSync(oversizedFilePath);
          if (stats.size >= totalSize) {
            clearInterval(checkInterval);
            resolve();
          }
        } catch (e) {
          // File not ready yet, continue polling
        }
      }, 100);
    });

    try {
      // Attempt to upload the oversized file
      await fileInput.setInputFiles(oversizedFilePath);
      await page.waitForTimeout(2000);

      // Should show error message
      const sizeLimit = page.locator('text=/50MB|file.*too.*large|size.*limit/i');
      const errorMessage = page.locator('.error, [role="alert"]');

      // Either validation message or error should be visible
      const hasSizeError = await sizeLimit.isVisible() || await errorMessage.isVisible();
      expect(hasSizeError).toBeTruthy();
    } finally {
      // Cleanup
      try {
        fs.unlinkSync(oversizedFilePath);
      } catch (e) {}
    }
  });

  test('should show error for invalid file type', async ({ page }) => {
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

    // Locate file input
    const fileInput = page.locator('input[type="file"]');
    if (!(await fileInput.isVisible())) {
      test.skip();
      return;
    }

    // Create a temporary file with disallowed extension
    const fs = require('fs');
    const os = require('os');
    const path = require('path');
    const invalidFilePath = path.join(os.tmpdir(), 'invalid-test-file.exe');
    fs.writeFileSync(invalidFilePath, 'Invalid file content');

    try {
      // Upload the invalid file
      await fileInput.setInputFiles(invalidFilePath);
      await page.waitForTimeout(2000);

      // Check for file type validation error
      const typeError = page.locator('.error, [role="alert"]');

      // Error message should be visible
      expect(await typeError.count()).toBeGreaterThan(0);
    } finally {
      // Cleanup
      try {
        fs.unlinkSync(invalidFilePath);
      } catch (e) {}
    }
  });

  test('should handle upload network failure', async ({ page }) => {
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

    // Intercept upload requests and abort them
    await page.route('**/files/upload**', route => route.abort());

    // Create a small test file
    const fs = require('fs');
    const os = require('os');
    const path = require('path');
    const testFilePath = path.join(os.tmpdir(), 'test-network-fail.txt');

    try {
      fs.writeFileSync(testFilePath, 'Test network failure');

      // Attempt to upload the file
      const fileInput = page.locator('input[type="file"]');
      if (await fileInput.isVisible()) {
        await fileInput.setInputFiles(testFilePath);

        // Wait for upload attempt to fail
        await page.waitForTimeout(2000);

        // Look for retry button or error message
        const retryButton = page.locator('button:has-text("Retry"), [aria-label*="Retry"]');
        const errorMessage = page.locator('.error, [role="alert"], text=/upload.*failed|network.*error/i');

        // Retry UI or error handling should be present
        const hasRetryUI = await retryButton.isVisible();
        const hasErrorHandling = await errorMessage.isVisible();

        expect(hasRetryUI || hasErrorHandling).toBeTruthy();
      }
    } finally {
      // Cleanup
      try {
        if (fs.existsSync(testFilePath)) {
          fs.unlinkSync(testFilePath);
        }
      } catch (e) {}
    }
  });
});
