import { SELECTORS } from '../utils/selectors';

describe('Sync', () => {
  it('syncs catalog and shows completion', async () => {
    await browser.$(SELECTORS.syncButton).click();
    await browser.waitUntil(
      async () => await browser.$(SELECTORS.toast).isDisplayed(),
      { timeout: 30000 }
    );
    const toastText = await browser.$(SELECTORS.toast).getText();
    expect(toastText).toMatch(/sync|drop|complete/i);
  });
});
