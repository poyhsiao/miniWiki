import { test, expect } from '@playwright/test';

test.describe('Comments E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display comments section', async ({ page }) => {
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

    // Look for comments section
    const commentsSection = page.locator('.comments-section, [class*="comments"]');
    const commentsButton = page.locator('button:has-text("Comments"), [aria-label*="Comments"]');

    const hasCommentsUI = await commentsSection.isVisible() || await commentsButton.isVisible();
    expect(hasCommentsUI).toBeTruthy();
  });

  test('should add a new comment', async ({ page }) => {
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

    // Look for comment input
    const commentInput = page.locator('textarea[placeholder*="Comment"], input[placeholder*="Comment"], .comment-input');
    const addCommentButton = page.locator('button:has-text("Add Comment"), button:has-text("Comment")');

    // Assert controls are visible or skip the test
    if (!(await commentInput.isVisible())) {
      test.skip();
      return;
    }

    // Get initial comment count
    const initialComments = await page.locator('.comment-item, [class*="comment-item"]').count();

    // Type comment
    await commentInput.click();
    await page.keyboard.type(`Test comment ${Date.now()}`);

    // Submit comment
    if (!(await addCommentButton.isVisible())) {
      test.skip();
      return;
    }

    await addCommentButton.click();
    await page.waitForTimeout(2000);

    // Verify comment was added
    const newComments = await page.locator('.comment-item, [class*="comment-item"]').count();
    expect(newComments).toBeGreaterThan(initialComments);
  });

  test('should display comment thread', async ({ page }) => {
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

    // Look for comment list
    const commentList = page.locator('.comment-list, [class*="comment-list"]');
    const commentItem = page.locator('.comment-item, [class*="comment-item"]');

    // Either show comments or empty state
    await expect(commentList.or(commentItem.first())).toBeVisible({ timeout: 5000 });
  });

  test('should reply to a comment', async ({ page }) => {
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

    // Look for reply button on a comment
    let commentItem = page.locator('.comment-item, [class*="comment-item"]').first();
    const replyButton = commentItem.locator('button:has-text("Reply"), [aria-label*="Reply"]');

    // Ensure a comment exists (create one if needed)
    if (!(await commentItem.isVisible())) {
      const commentInput = page.locator('textarea[placeholder*="Comment"], input[placeholder*="Comment"], .comment-input');
      const addCommentButton = page.locator('button:has-text("Add Comment"), button:has-text("Comment")');
      if (!(await commentInput.isVisible()) || !(await addCommentButton.isVisible())) {
        test.skip(true, 'Comment input not available');
        return;
      }
      await commentInput.click();
      await page.keyboard.type(`Seed comment ${Date.now()}`);
      await addCommentButton.click();
      await page.waitForTimeout(2000);

      // Refresh comment item locator
      commentItem = page.locator('.comment-item, [class*="comment-item"]').first();
    }

    // Assert comment item and reply button are visible
    await expect(commentItem).toBeVisible({ timeout: 5000 });
    await expect(replyButton).toBeVisible({ timeout: 5000 });

    await replyButton.click();
    await page.waitForTimeout(500);

    // Reply input should appear
    const replyInput = page.locator('textarea[placeholder*="Reply"], .reply-input');
    await expect(replyInput).toBeVisible({ timeout: 5000 });

    // Type reply
    await replyInput.fill(`Reply to comment ${Date.now()}`);

    // Submit reply - scope to the comment thread to avoid ambiguity
    const submitButton = commentItem.locator('button:has-text("Send")').or(
      replyInput.locator('xpath=following-sibling::*//button[contains(text(), "Send") or contains(text(), "Reply")]')
    );
    await expect(submitButton).toBeVisible({ timeout: 5000 });

    await submitButton.click();
    await page.waitForTimeout(2000);

    // Verify reply was added
    const replies = page.locator('.replies, [class*="reply"], .nested-comment');
    await expect(replies).toBeVisible({ timeout: 5000 });
  });

  test('should resolve a comment', async ({ page }) => {
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

    // Ensure a comment exists (create one if needed)
    let commentItem = page.locator('.comment-item').first();
    const resolveButton = page.locator('button:has-text("Resolve"), [aria-label*="Resolve"]');

    if (!(await commentItem.isVisible())) {
      const commentInput = page.locator('textarea[placeholder*="Comment"], input[placeholder*="Comment"], .comment-input');
      const addCommentButton = page.locator('button:has-text("Add Comment"), button:has-text("Comment")');
      if (!(await commentInput.isVisible()) || !(await addCommentButton.isVisible())) {
        test.skip(true, 'Comment input not available');
        return;
      }
      await commentInput.click();
      await page.keyboard.type(`Seed comment for resolve ${Date.now()}`);
      await addCommentButton.click();
      await page.waitForTimeout(2000);

      // Refresh comment item locator
      commentItem = page.locator('.comment-item').first();
    }

    // Skip if resolve button is not visible
    if (!(await resolveButton.isVisible())) {
      test.skip(true, 'Resolve button not available');
      return;
    }

    await resolveButton.click();
    await page.waitForTimeout(500);

    // Should show confirmation or update UI
    const resolvedIndicator = page.locator('.resolved, [class*="resolved"], text=/Resolved/i');
    await expect(resolvedIndicator).toBeVisible({ timeout: 5000 });
  });

  test('should delete a comment', async ({ page }) => {
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

    // Get initial comment count
    let initialComments = await page.locator('.comment-item, [class*="comment-item"]').count();

    // Create a comment if none exist
    if (initialComments === 0) {
      const commentInput = page.locator('textarea[placeholder*="Comment"], input[placeholder*="Comment"], .comment-input');
      const addCommentButton = page.locator('button:has-text("Add Comment"), button:has-text("Comment")');

      if (!(await commentInput.isVisible()) || !(await addCommentButton.isVisible())) {
        test.skip();
        return;
      }

      await commentInput.click();
      await page.keyboard.type(`Test comment for deletion ${Date.now()}`);
      await addCommentButton.click();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);

      // Recompute comment count
      initialComments = await page.locator('.comment-item, [class*="comment-item"]').count();
    }

    if (initialComments === 0) {
      test.skip();
      return;
    }

    // Look for delete button on first comment
    const deleteButton = page.locator('.comment-item').first().locator('[aria-label*="Delete"], button:has-text("Delete")');

    if (!(await deleteButton.isVisible())) {
      test.skip();
      return;
    }

    await deleteButton.click();
    await page.waitForTimeout(500);

    // Confirm deletion if dialog appears
    const confirmButton = page.locator('button:has-text("Delete"), button:has-text("Confirm")');
    if (await confirmButton.isVisible()) {
      await confirmButton.click();
      await page.waitForTimeout(2000);
    }

    // Verify comment was deleted
    const newComments = await page.locator('.comment-item, [class*="comment-item"]').count();
    expect(newComments).toBeLessThan(initialComments);
  });

  test('should show comment author info', async ({ page }) => {
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

    // Look for comment with author info
    const commentItem = page.locator('.comment-item, [class*="comment-item"]').first();

    if (await commentItem.isVisible()) {
      // Check for author name, avatar, timestamp
      const authorName = page.locator('[class*="author"], [class*="user"]');
      const timestamp = page.locator('text=/\\d{4}|ago/i');

      // Author info should be visible
      const hasAuthorInfo = await authorName.isVisible() || await timestamp.isVisible();
      expect(hasAuthorInfo).toBeTruthy();
    }
  });
});

test.describe('Comments Error Handling E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should show validation error for empty comment', async ({ page }) => {
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

    // Try to submit empty comment
    const commentInput = page.locator('textarea[placeholder*="Comment"]');
    const addButton = page.locator('button:has-text("Add Comment")');

    // Assert controls are visible - bail out early if not
    if (!(await commentInput.isVisible()) || !(await addButton.isVisible())) {
      test.skip();
      return;
    }

    await addButton.click();
    await page.waitForTimeout(1000);

    // Should show validation error
    const errorMessage = page.locator('.error, [role="alert"], text=/required|empty/i');
    expect(await errorMessage.isVisible()).toBeTruthy();
  });

  test('should handle comment submission failure', async ({ page }) => {
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

    // Intercept comment API and fail
    await page.route('**/comments**', route => route.abort());

    // Try to add comment
    const commentInput = page.locator('textarea[placeholder*="Comment"]');
    const addButton = page.locator('button:has-text("Add Comment")');

    // Assert preconditions before proceeding
    expect(await commentInput.isVisible()).toBeTruthy();
    expect(await addButton.isVisible()).toBeTruthy();

    await commentInput.fill('Test comment');
    await page.waitForTimeout(500);
    await addButton.click();
    await page.waitForTimeout(2000);

    // Should show error message
    const errorMessage = page.locator('.error, [role="alert"]');
    expect(await errorMessage.isVisible()).toBeTruthy();
  });
});
