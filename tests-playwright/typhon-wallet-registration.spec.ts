import { test, chromium } from '@playwright/test';

test('Open Extension Page', async ({}) => {
  const extensionPath: string = '/Users/alicechaiyakul/typhon-wallet-registration/catalyst-core/extensions';
  const extensionId: string = 'kfdniefadaanbjodldohaedphafoffoh'; // Replace with your extension's ID
  const extensionPage: string = 'tab.html'; // Replace with the specific page
  const userDataDir = '/Users/alicechaiyakul/typhon-wallet-registration/catalyst-core/src/usrdatadir'; // Path to the user data directory

  // Launch Chromium with the extension
  const browser = await chromium.launchPersistentContext(userDataDir, {
    headless: false, // Extensions only work in headful mode
    args: [
      `--disable-extensions-except=${extensionPath}`,
      `--load-extension=${extensionPath}`,
    ],
  });

  // Creating a new context and page
  const page = await browser.newPage();
  const buttonSelector = '#headlessui-menu-button-1';

  // Navigating to the extension's page
  await page.goto(`chrome-extension://kfdniefadaanbjodldohaedphafoffoh/tab.html#/wallet/access`);
  await page.waitForSelector(buttonSelector);
  await page.click(buttonSelector);

  // Keeping the browser open to debug(remove the timeout or adjust as needed)
  await page.waitForTimeout(300000); // Adjust the time as needed
  await new Promise(resolve => { /* never resolves */ });
});
