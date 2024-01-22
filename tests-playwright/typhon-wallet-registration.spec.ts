import { test, chromium, BrowserContext, Page } from '@playwright/test';
import { getWalletCredentials } from './credentials';
import { getSeedPhrase } from './seed-phrase';

test('Open Extension Page and Click Button with XPath', async ({}) => {
  const extensionPath: string = '/Users/alicechaiyakul/typhon-wallet-registration/catalyst-core/extensions';
  // const extensionId: string = 'kfdniefadaanbjodldohaedphafoffoh'; // Replace with your extension's ID
  // const extensionPage: string = 'tab.html'; // Replace with the specific page
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

  const firstButtonSelector = '//*[@id="headlessui-menu-button-1"]';
  await newTab.waitForSelector(firstButtonSelector, { state: 'visible' });
  await newTab.click(firstButtonSelector);

  const secondButtonSelector = '#headlessui-menu-item-6';
  await newTab.waitForSelector(secondButtonSelector, { state: 'visible' });
  await newTab.click(secondButtonSelector);

  const thirdButtonSelector = '//*[text()="Import"]';
  await newTab.waitForSelector(thirdButtonSelector, { state: 'visible' });
  await newTab.click(thirdButtonSelector);

  const WalletCredentials = getWalletCredentials('WALLET1');
  const usernameInput = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(2) > div > input';
  const passwordInput = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(2) > div > div:nth-child(2) > input';
  const cfpwInput = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(2) > div > div:nth-child(3) > input';
  await newTab.waitForSelector(usernameInput, { state: 'visible' });
  await newTab.waitForSelector(passwordInput, { state: 'visible' });
  await newTab.waitForSelector(cfpwInput, { state: 'visible' });
  await newTab.fill(usernameInput, WalletCredentials.username);
  await newTab.fill(passwordInput, WalletCredentials.password);
  await newTab.fill(cfpwInput, WalletCredentials.password);

  const agreeToTC = '#termsAndConditions'
  await newTab.waitForSelector(agreeToTC, { state: 'visible' });
  await newTab.click(agreeToTC);

  const continueButton = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(2) > div > button';
  await newTab.waitForSelector(continueButton, { state: 'visible' });
  await newTab.click(continueButton);

  const seedPhrase = getSeedPhrase();
  for (let i = 0; i < seedPhrase.length; i++) {
  const ftSeedPhrase1 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(1) > div > input';
  await newTab.waitForSelector(ftSeedPhrase1, { state: 'visible' });
  await newTab.fill(ftSeedPhrase1, seedPhrase[i]);
  
  const ftSeedPhrase2 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(2) > div > input';
  const ftSeedPhrase3 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(3) > div > input';
  const ftSeedPhrase4 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(4) > div > input';
  const ftSeedPhrase5 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(5) > div > input';
  const ftSeedPhrase6 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(6) > div > input';
  const ftSeedPhrase7 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(7) > div > input';
  const ftSeedPhrase8 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(8) > div > input';
  const ftSeedPhrase9 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(9) > div > input';
  const ftSeedPhrase10 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(10) > div > input';
  const ftSeedPhrase11 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(11) > div > input';
  const ftSeedPhrase12 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(12) > div > input';
  const ftSeedPhrase13 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(13) > div > input';
  const ftSeedPhrase14 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(14) > div > input';
  const ftSeedPhrase15 = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(15) > div > input';

  // Reset the state of user
  // for (const page of browser.pages()) {
  //   await page.close();
  // }

  // Keeping the browser open for debugging and verifying (remove the timeout or adjust as needed)
  await page.waitForTimeout(300000); // Adjust the time as needed
  await new Promise(resolve => { /* never resolves */ });
}});
