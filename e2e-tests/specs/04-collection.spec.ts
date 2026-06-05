import { SELECTORS } from '../utils/selectors';

describe('Collection', () => {
  it('adds first search result to collection', async () => {
    // Search first
    await browser.$(SELECTORS.searchInput).setValue('Gibson');
    await browser.$(SELECTORS.searchButton).click();
    await browser.waitUntil(
      async () => (await browser.$$(SELECTORS.productCard)).length > 0,
      { timeout: 10000 }
    );
    // Add to collection
    await browser.$(SELECTORS.addToCollectionBtn).click();
    // Verify collection cell updated
    await browser.waitUntil(
      async () => {
        const cell = await browser.$(SELECTORS.collectionCell);
        const text = await cell.getText();
        return text.includes('1');
      },
      { timeout: 10000 }
    );
  });
});
