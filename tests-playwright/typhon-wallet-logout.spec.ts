import { test, chromium, Browser, Page } from '@playwright/test';
import { getWalletCredentials } from './credentials';
import { getSeedPhrase } from './seed-phrase';

test('Logout', async ({ }) => {
  const extensionPath: string = '../../catalyst-core/extensions';
  // const extensionId: string = 'kfdniefadaanbjodldohaedphafoffoh'; // Replace with your extension's ID
  // const extensionPage: string = 'tab.html'; // Replace with the specific page
  const userDataDir = '../../catalyst-core/src/usrdatadir'; // Path to the user data directory

  // Launch Chromium with the extension
  const browser = await chromium.launchPersistentContext(userDataDir, {
    headless: false, // Extensions only work in headful mode
    args: [
      `--disable-extensions-except=${extensionPath}`,
      `--load-extension=${extensionPath}`,
    ],
  });

  const page = await browser.newPage();
  await page.waitForTimeout(1000); // Adjust the timeout as needed

  // Get all pages (tabs) in the contextg
  const pages = browser.pages();

  // The new tab is typically the last one in the list
  const newTab = pages[pages.length - 1];
  await newTab.bringToFront(); // Brings the new tab to the foreground

  const logOut = '//*[@id="app"]/div/div/div[3]/div/div/div[1]/div/div/div[2]/div[11]/div[2]';
  await newTab.waitForSelector(logOut, { state: 'visible' });
  await newTab.click(logOut);

  const chooseAccount = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[2]/div';
  await newTab.waitForSelector(chooseAccount, { state: 'visible' });
  await newTab.click(chooseAccount);

  const removeAccount = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[2]/div[4]/button';
  await newTab.waitForSelector(removeAccount, { state: 'visible' });
  await newTab.click(removeAccount);

  const confirmRemove = 'button.btn.bg-primary';
  await newTab.waitForSelector(confirmRemove, {state: 'visible'});
  await newTab.click(confirmRemove)

  const addNew = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[4]';
  await newTab.waitForSelector(addNew, { state: 'visible' });
  await newTab.click(addNew);

});
