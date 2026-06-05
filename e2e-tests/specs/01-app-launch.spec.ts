describe('App Launch', () => {
  it('opens a window titled GuitarHub with the dashboard visible', async () => {
    const title = await browser.getTitle();
    expect(title).toContain('GuitarHub');

    const dashboardCells = await browser.$$('.dashboard-cell');
    expect(dashboardCells.length).toBeGreaterThan(0);
  });
});
