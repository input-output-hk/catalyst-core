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
    // Construct the selector dynamically for each element in the seedPhrase
    const ftSeedPhraseSelector = `#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(${i + 1}) > div > input`;

    await newTab.waitForSelector(ftSeedPhraseSelector, { state: 'visible' });
    await newTab.fill(ftSeedPhraseSelector, seedPhrase[i]);
}
const blankSpace = '#app > div > div > div.flex-grow.overflow-auto > div > div.container.mx-auto.flex.justify-between.items-center > a > img';
await newTab.waitForSelector(blankSpace, { state: 'visible' });
await newTab.click(blankSpace);

const unlockWallet = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div.mt-6.text-center.flex.justify-center > button';
await newTab.waitForSelector(unlockWallet, { state: 'visible' });
await newTab.click(unlockWallet);

const logOut = '#app > div > div > div.flex-grow.overflow-auto > div > div > div.hidden.h-full.overflow-y-scroll.flex-grow-0.flex-shrink-0.lg\:block.lg\:w-64 > div > div > div.mt-5.flex-grow.flex.flex-col.pl-3.pr-7.text-sm.labelColor2 > div.p-3.flex.space-x-3.items-center.cursor-pointer.hover\:text-primary > div:nth-child(2)';
await newTab.waitForSelector(logOut, { state: 'visible' });
await newTab.click(logOut);

const chooseAccount = '#app > div > div > div.flex-grow.overflow-auto > div > div.flex.justify-center > div > div > div.w-full.flex-grow.flex.flex-col.overflow-scroll.dark\:bg-dark-2.border-b.dark\:border-dark-3.pt-7.pb-2.px-4.space-y-2 > div';
await newTab.waitForSelector(chooseAccount, { state: 'visible' });
await newTab.click(chooseAccount);

const removeAccount = '#app > div > div > div.flex-grow.overflow-auto > div > div.flex.justify-center > div > div > div.flex-grow.flex.flex-col.items-center.pt-5.w-full > div.flex.items-center.justify-between.border-t.border-primary-8.dark\:border-dark-3.bg-primary-2.dark\:bg-dark-2.pl-3.pr-2.py-3.space-x-2 > button';
await newTab.waitForSelector(removeAccount, { state: 'visible' });
await newTab.click(removeAccount);

const confirmRemove = '#headlessui-dialog-17 > div > div.relative.inline-block.w-full.text-left.align-middle.cardColor.rounded-lg.shadow-xl.transform.p-6.pt-4.mx-3.my-8.max-w-2xl > div.max-w-lg > div.mt-5.flex.justify-end.items-center.space-x-3 > button.btn.bg-primary.text-xs.w-full.sm\:w-20.uppercase.px-4.py-2.font-medium.text-white.hover\:bg-opacity-80';
await newTab.waitForSelector(confirmRemove, { state: 'visible' });
await newTab.click(confirmRemove);

const addNew = '#app > div > div > div.flex-grow.overflow-auto > div > div.flex.justify-center > div > div > div.absolute.py-2.pl-2.pr-3.flex.justify-center.items-center.rounded-full.text-sm.bg-primary-80.hover\:bg-primary-100.text-white.cursor-pointer.bottom-14.right-3.shadow-xl';
await newTab.waitForSelector(addNew, { state: 'visible' });
await newTab.click(addNew);

// Keeping the browser open for debugging and verifying (remove the timeout or adjust as needed)
await page.waitForTimeout(3000000); // Adjust the time as needed
await new Promise(resolve => { /* never resolves */ });
});
