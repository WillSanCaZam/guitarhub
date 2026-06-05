import { SELECTORS } from '../utils/selectors';

describe('Search', () => {
  it('searches for Fender and shows results', async () => {
    await browser.$(SELECTORS.searchInput).setValue('Fender');
    await browser.$(SELECTORS.searchButton).click();
    await browser.waitUntil(
      async () => (await browser.$$(SELECTORS.productCard)).length > 0,
      { timeout: 10000 }
    );
    const cards = await browser.$$(SELECTORS.productCard);
    expect(cards.length).toBeGreaterThan(0);
  });
});
