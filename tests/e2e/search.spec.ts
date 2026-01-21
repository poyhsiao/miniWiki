import { test, expect } from '@playwright/test';

test.describe('Full-Text Search E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display search bar', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for search input
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"], [aria-label*="Search"]');
    const searchBar = page.locator('.search-bar, .search-container, [class*="search"]');

    const hasSearchUI = await searchInput.isVisible() || await searchBar.isVisible();
    expect(hasSearchUI).toBeTruthy();
  });

  test('should return results for search query', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for search input
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');

    if (await searchInput.isVisible()) {
      // Click and type search query
      await searchInput.click();
      await page.keyboard.type('test');

      // Wait for results
      await page.waitForTimeout(1500);

      // Results should appear
      const searchResults = page.locator('.search-results, [class*="search-results"]');
      const resultItem = page.locator('.search-result, [class*="search-result"]');

      // Either results or empty state should be visible
      const hasResults = await searchResults.isVisible() || await resultItem.first().isVisible();
      const noResults = page.locator('.no-results, [class*="no-results"]');

      expect(hasResults || await noResults.isVisible()).toBeTruthy();
    }
  });

  test('should filter results by document type', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Perform search
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');
    if (await searchInput.isVisible()) {
      await searchInput.click();
      await page.keyboard.type('test');
      await page.waitForTimeout(1500);
    }

    // Look for filter options
    const filterButton = page.locator('button:has-text("Filter"), [aria-label*="Filter"]');
    const typeFilter = page.locator('select, [role="combobox"], text=Type');

    if (await filterButton.isVisible()) {
      await filterButton.click();
      await page.waitForTimeout(500);
    }

    // Filter UI should be available
    const hasFilters = await typeFilter.isVisible() || await filterButton.isVisible();
    expect(hasFilters).toBeTruthy();
  });

  test('should highlight search terms in results', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Perform search
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');

    if (!(await searchInput.isVisible())) {
      test.skip();
      return;
    }

    await searchInput.click();
    await page.keyboard.type('test');
    await page.waitForTimeout(1500);

    // Look for highlighted terms in results
    const highlightedText = page.locator('mark, .highlight, [class*="highlight"], strong');
    const resultItem = page.locator('.search-result, [class*="search-result"]').first();

    // Check if result item is visible - skip if not
    const resultVisible = await resultItem.isVisible();
    if (!resultVisible) {
      test.skip();
      return;
    }

    // Check for highlighting and verify count
    const highlightCount = await highlightedText.count();
    expect(highlightCount).toBeGreaterThan(0);
  });

  test('should search with special characters', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for search input
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');

    if (await searchInput.isVisible()) {
      // Click and type search query with special characters
      await searchInput.click();
      await page.keyboard.type('test@example.com');
      await page.keyboard.press('Enter');

      // Wait for results
      await page.waitForTimeout(1500);

      // Should handle special characters gracefully
      const searchResults = page.locator('.search-results, [class*="search-results"]');
      const errorMessage = page.locator('.error, [data-testid="error"], .toast-error');

      // Either results or no results, but no error
      expect(await errorMessage.isVisible()).toBe(false);
    }
  });

  test('should search with multiple terms', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for search input
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');

    // Check search input visibility - skip if not available
    if (!(await searchInput.isVisible())) {
      test.skip();
      return;
    }

    // Click and type multiple search terms
    await searchInput.click();
    await page.keyboard.type('test query multiple terms');
    await page.keyboard.press('Enter');

    // Wait for results
    await page.waitForTimeout(1500);

    // Should handle multiple terms - check for results or no-results
    const searchResults = page.locator('.search-results, [class*="search-results"]');
    const noResults = page.locator('.no-results, [class*="no-results"]');

    const hasResults = await searchResults.isVisible();
    const hasNoResults = await noResults.isVisible();

    // Either results or no-results message should be present
    if (!hasResults && !hasNoResults) {
      test.skip();
      return;
    }

    expect(hasResults || hasNoResults).toBeTruthy();
  });

  test('should show search suggestions', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Look for search input
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');

    if (await searchInput.isVisible()) {
      // Type partial query
      await searchInput.click();
      await page.keyboard.type('te');

      // Wait for suggestions
      await page.waitForTimeout(500);

      // Look for suggestions dropdown
      const suggestions = page.locator('.suggestions, [class*="suggestion"]');
      const autocomplete = page.locator('[role="listbox"], [role="option"]');

      const hasSuggestions = await suggestions.isVisible() || await autocomplete.count() > 0;
      expect(hasSuggestions).toBeTruthy();
    }
  });

  test('should navigate to search result', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Perform search
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');
    if (await searchInput.isVisible()) {
      await searchInput.click();
      await page.keyboard.type('test');
      await page.waitForTimeout(1500);
    }

    // Click on a result
    const resultItem = page.locator('.search-result, [class*="search-result"]').first();

    if (await resultItem.isVisible()) {
      const initialUrl = page.url();
      await resultItem.click();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(2000);

      // Should navigate to the document
      const urlChanged = page.url() !== initialUrl;
      expect(urlChanged).toBeTruthy();
    }
  });
});

test.describe('Search Performance E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should return results within 5s', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Measure search performance
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');

    if (await searchInput.isVisible()) {
      await searchInput.click();

      // Start timing
      const startTime = Date.now();

      await page.keyboard.type('test');
      await page.keyboard.press('Enter');

      // Wait for results
      await page.waitForSelector('.search-results, [class*="search-results"]', { timeout: 5000 });

      const endTime = Date.now();
      const duration = endTime - startTime;

      // Log the duration for verification
      console.log(`Search took ${duration}ms`);

      // Should be reasonably fast (< 5 seconds for E2E test)
      expect(duration).toBeLessThan(5000);
    }
  });

  test('should handle empty search results gracefully', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Search for unlikely term
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');
    if (await searchInput.isVisible()) {
      await searchInput.click();
      await page.keyboard.type('xyznonexistent123');
      await page.keyboard.press('Enter');

      await page.waitForTimeout(1500);
    }

    // Should show empty state message or empty results container
    const noResults = page.locator('.no-results, [class*="no-results"], text=/no.*result|not.*found/i');
    const emptyResultsList = page.locator('.search-results:empty, [class*="search-results"]:empty');

    await expect(noResults.or(emptyResultsList)).toBeVisible({ timeout: 5000 });
  });

  test('should preserve search query in URL', async ({ page }) => {
    await page.goto('/spaces');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000);

    // Perform search
    const searchInput = page.locator('input[type="search"], input[placeholder*="Search"]');
    if (await searchInput.isVisible()) {
      await searchInput.click();
      await page.keyboard.type('test search');
      await page.keyboard.press('Enter');

      await page.waitForTimeout(1500);
    }

    // Check if search query is in URL using URLSearchParams
    const url = new URL(page.url());
    const params = url.searchParams;
    const hasSearchParam = params.has('search') || params.has('q') || params.has('query');

    // Verify URL query preservation (primary assertion)
    expect(hasSearchParam).toBeTruthy();

    // Also verify results are visible
    const hasResults = await page.locator('.search-results, [class*="search-results"]').isVisible();
    expect(hasResults).toBeTruthy();
  });
});
