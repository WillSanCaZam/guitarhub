import { SELECTORS } from '../utils/selectors';

describe('Filters', () => {
  it('toggles filter bar visibility', async () => {
    // Filter bar starts collapsed
    const toggle = await browser.$(SELECTORS.filterToggle);
    expect(await toggle.isDisplayed()).toBe(true);
    expect(await toggle.getAttribute('aria-expanded')).toBe('false');

    // Expand
    await toggle.click();
    expect(await toggle.getAttribute('aria-expanded')).toBe('true');
    const category = await browser.$(SELECTORS.filterCategory);
    expect(await category.isDisplayed()).toBe(true);

    // Collapse
    await toggle.click();
    expect(await toggle.getAttribute('aria-expanded')).toBe('false');
  });

  it('filters by category and clears it', async () => {
    // Expand filters
    await browser.$(SELECTORS.filterToggle).click();

    // Select a category
    const categorySelect = await browser.$(SELECTORS.filterCategory);
    await categorySelect.selectByAttribute('value', 'Guitar');
    expect(await categorySelect.getValue()).toBe('Guitar');

    // Clear category
    await browser.$(SELECTORS.clearCategory).click();
    expect(await categorySelect.getValue()).toBe('');
  });

  it('sets price range and clears individual fields', async () => {
    await browser.$(SELECTORS.filterToggle).click();

    // Set min price
    const minInput = await browser.$(SELECTORS.filterPriceMin);
    await minInput.setValue('100');
    expect(await minInput.getValue()).toBe('100');

    // Set max price
    const maxInput = await browser.$(SELECTORS.filterPriceMax);
    await maxInput.setValue('2000');
    expect(await maxInput.getValue()).toBe('2000');

    // Clear min price
    await browser.$(SELECTORS.clearPriceMin).click();
    expect(await minInput.getValue()).toBe('');

    // Clear max price
    await browser.$(SELECTORS.clearPriceMax).click();
    expect(await maxInput.getValue()).toBe('');
  });

  it('selects condition and currency', async () => {
    await browser.$(SELECTORS.filterToggle).click();

    // Select condition
    const conditionSelect = await browser.$(SELECTORS.filterCondition);
    await conditionSelect.selectByAttribute('value', 'new');
    expect(await conditionSelect.getValue()).toBe('new');
    await browser.$(SELECTORS.clearCondition).click();
    expect(await conditionSelect.getValue()).toBe('');

    // Select currency
    const currencySelect = await browser.$(SELECTORS.filterCurrency);
    await currencySelect.selectByAttribute('value', 'EUR');
    expect(await currencySelect.getValue()).toBe('EUR');
    await browser.$(SELECTORS.clearCurrency).click();
    expect(await currencySelect.getValue()).toBe('');
  });

  it('changes sort order and resets it', async () => {
    await browser.$(SELECTORS.filterToggle).click();

    const sortSelect = await browser.$(SELECTORS.filterSort);
    await sortSelect.selectByAttribute('value', 'price_asc');
    expect(await sortSelect.getValue()).toBe('price_asc');

    await browser.$(SELECTORS.clearSort).click();
    expect(await sortSelect.getValue()).toBe('relevance');
  });

  it('clears all filters at once', async () => {
    await browser.$(SELECTORS.filterToggle).click();

    // Set several filters
    const categorySelect = await browser.$(SELECTORS.filterCategory);
    await categorySelect.selectByAttribute('value', 'Pedal');

    const minInput = await browser.$(SELECTORS.filterPriceMin);
    await minInput.setValue('50');

    const conditionSelect = await browser.$(SELECTORS.filterCondition);
    await conditionSelect.selectByAttribute('value', 'used');

    // Verify filters are set
    expect(await categorySelect.getValue()).toBe('Pedal');
    expect(await minInput.getValue()).toBe('50');
    expect(await conditionSelect.getValue()).toBe('used');

    // Clear all
    await browser.$(SELECTORS.filterClearAll).click();

    // Verify all reset to defaults
    expect(await categorySelect.getValue()).toBe('');
    expect(await minInput.getValue()).toBe('');
    expect(await conditionSelect.getValue()).toBe('');
  });
});
