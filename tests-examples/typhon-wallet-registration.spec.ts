import { test, chromium } from '@playwright/test';

test('Open Extension Page', async ({}) => {
  const extensionPath: string = 'extensions/kfdniefadaanbjodldohaedphafoffoh/3.0.23_0/manifest.json';
  const extensionId: string = 'kfdniefadaanbjodldohaedphafoffoh'; // Replace with your extension's ID
  const extensionPage: string = 'tab.html'; // Replace with the specific page

  // Launch Chromium with the extension
  const browser = await chromium.launch({
    headless: false, // Extensions only work in headful mode
    args: [
      `--disable-extensions-except=${extensionPath}`,
      `--load-extension=${extensionPath}`,
    ],
  });

  // Creating a new context and page
  const context = await browser.newContext();
  const page = await context.newPage();

  // Navigating to the extension's page
  await page.goto(`chrome-extension://kfdniefadaanbjodldohaedphafoffoh/tab.html#/wallet/access`);

  // Keeping the browser open (remove the timeout or adjust as needed)
  await page.waitForTimeout(300000); // Adjust the time as needed

  // Optionally, close the context if you want to close the browser programmatically
  // await context.close();
});
