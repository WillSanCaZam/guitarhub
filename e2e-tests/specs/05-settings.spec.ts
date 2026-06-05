import { SELECTORS } from '../utils/selectors';

describe('Settings', () => {
  it('changes alert channel to ntfy', async () => {
    await browser.$(SELECTORS.settingsNav).click();
    await browser.waitUntil(
      async () => await browser.$(SELECTORS.settingsForm).isDisplayed(),
      { timeout: 10000 }
    );
    // Select ntfy radio
    const ntfyRadio = await browser.$('input[value="ntfy"]');
    await ntfyRadio.click();
    // Save
    const saveBtn = await browser.$('button[type="submit"]');
    await saveBtn.click();
    // Verify persistence (reload page)
    await browser.refresh();
    await browser.$(SELECTORS.settingsNav).click();
    const checked = await ntfyRadio.isSelected();
    expect(checked).toBe(true);
  });
});
