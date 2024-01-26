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

  const confirmRemove = '#headlessui-dialog-17 > div > div.relative.inline-block.w-full.text-left.align-middle.cardColor.rounded-lg.shadow-xl.transform.p-6.pt-4.mx-3.my-8.max-w-2xl > div.max-w-lg > div.mt-5.flex.justify-end.items-center.space-x-3 > button.btn.bg-primary.text-xs.w-full.sm\:w-20.uppercase.px-4.py-2.font-medium.text-white.hover\:bg-opacity-80';
  await newTab.waitForSelector(confirmRemove, { state: 'visible' });
  await newTab.click(confirmRemove);

  const addNew = '#app > div > div > div.flex-grow.overflow-auto > div > div.flex.justify-center > div > div > div.absolute.py-2.pl-2.pr-3.flex.justify-center.items-center.rounded-full.text-sm.bg-primary-80.hover\:bg-primary-100.text-white.cursor-pointer.bottom-14.right-3.shadow-xl';
  await newTab.waitForSelector(addNew, { state: 'visible' });
  await newTab.click(addNew);

  // Keeping the browser open for debugging and verifying (remove the timeout or adjust as needed)
  await newTab.waitForTimeout(300000); // Adjust the time as needed
  await new Promise(resolve => { /* never resolves */ });

});
