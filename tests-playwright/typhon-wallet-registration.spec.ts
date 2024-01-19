import { test, chromium, BrowserContext, Page } from '@playwright/test';

test('Open Extension Page and Click Button with XPath', async ({}) => {
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
  await page.waitForTimeout(1000); // Adjust the timeout as needed

  // Get all pages (tabs) in the context
  const pages = browser.pages();

  // The new tab is typically the last one in the list
  const newTab = pages[pages.length - 1];

  // Now you can interact with the new tab
  await newTab.bringToFront(); // Brings the new tab to the foreground

  // Example: Get the title of the new tab
  const title = await newTab.title();
  console.log(`Title of the new tab: ${title}`);

  // XPath selector for the button
  const buttonXPath = '//*[@id="headlessui-menu-button-1"]';

  // Wait for the button to be visible
  await newTab.waitForSelector(buttonXPath, { state: 'visible' });

  // Click the button using XPath
  await newTab.click(buttonXPath);

  const secondButtonSelector = '#headlessui-menu-item-6';

  // Wait for the second button to be visible
  await newTab.waitForSelector(secondButtonSelector, { state: 'visible' });

  // Click the second button
  await newTab.click(secondButtonSelector);

  // Reset the state of user
  // for (const page of browser.pages()) {
  //   await page.close();
  // }

  // Keeping the browser open for debugging and verifying (remove the timeout or adjust as needed)
  await page.waitForTimeout(300000); // Adjust the time as needed
  await new Promise(resolve => { /* never resolves */ });
});
