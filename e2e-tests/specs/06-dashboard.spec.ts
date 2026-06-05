import { SELECTORS } from '../utils/selectors';

describe('Dashboard', () => {
  it('shows bento grid with 9 cells', async () => {
    const grid = await browser.$(SELECTORS.dashboardGrid);
    expect(await grid.isDisplayed()).toBe(true);
    const cells = await grid.$$('.dashboard-cell');
    expect(cells.length).toBeGreaterThanOrEqual(9);
  });
});
