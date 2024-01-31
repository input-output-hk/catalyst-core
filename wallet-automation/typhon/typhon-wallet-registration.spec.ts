import { test, chromium, BrowserContext, Page } from '@playwright/test';
import { getWalletCredentials } from './credentials';
import { getSeedPhrase } from './seed-phrase';

test('import wallet', async ({ }) => {
  const extensionPath: string = '../../wallet-automation/typhon/extensions';
  // const extensionId: string = 'kfdniefadaanbjodldohaedphafoffoh'; // Replace with your extension's ID
  // const extensionPage: string = 'tab.html'; // Replace with the specific page
  const userDataDir = '../../wallet-automation/typhon/usrdatadir'; // Path to the user data directory

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
  const pages = browser.pages();
  const newTab = pages[pages.length - 1];
  await newTab.bringToFront(); // Brings the new tab to the foreground

  // Example: Get the title of the new tab
  const title = await newTab.title();
  console.log(`title of the new tab: ${title}`);

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

  async function clickBlankSpace(newTab) {
    const blankSpace = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div.flex.justify-between.items-start > div.flex-initial.flex.flex-col.mr-2 > span.text-primary.font-medium.text-xl';
    await newTab.waitForSelector(blankSpace, { state: 'visible' });
    await newTab.click(blankSpace);
  }

  const seedPhrase = getSeedPhrase();

  for (let i = 0; i < seedPhrase.length; i++) {
    const ftSeedPhraseSelector = `#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(${i + 1}) > div > input`;
    await newTab.waitForSelector(ftSeedPhraseSelector, { state: 'visible' });
    await newTab.fill(ftSeedPhraseSelector, seedPhrase[i]);
  }

  const unlockWallet = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div.mt-6.text-center.flex.justify-center > button';
  await clickBlankSpace(newTab);
  await newTab.waitForSelector(unlockWallet, { state: 'visible' });
  await newTab.click(unlockWallet);

  const divSelector = '//*[@id="lc"]/div[2]/div[1]/div[2]/div/div[1]/div[1]/div/div[2]/div[1]/div/span[1]';
  await newTab.waitForSelector(divSelector, { state: 'visible' });

  // Use the selector to retrieve the element handle
  const elementHandle = await newTab.$(divSelector);
  if (elementHandle) {
    // Retrieve the text content of the element
    const textContent = await elementHandle.textContent();
    if (textContent !== null) {
      // Remove any formatting that might interfere with parseFloat
      const cleanedText = textContent.replace(/,/g, '').trim();
      const floatValue = parseFloat(cleanedText);
      if (!isNaN(floatValue)) {
        if (floatValue < 500) {
          // Log the message if the float value is less than 500
          console.log('not eligible for voting ☹️');
        } else {
          // Log the message if the float value is equal to or more than 500
          console.log('eligible for voting ☺');
        }
      } else {
        console.log('text content is not a valid float:', textContent);
      }
    } else {
      console.log('no text content found for the specified selector:', divSelector);
    }
  } else {
    console.log('element not found for the specified XPath:', divSelector);
  }
  
  // const copyAddress = '#receiveAddress > div > div.grow > button';
  // await newTab.waitForSelector(copyAddress, { state: 'visible' });
  // await newTab.click(copyAddress);
  // await newTab.waitForTimeout(500);
  // const copiedText = await newTab.evaluate(() => navigator.clipboard.readText());
  // console.log('Copied Address:', copiedText);

  // const newTab2 = await browser.newPage();
  // await newTab2.goto('https://docs.cardano.org/cardano-testnet/tools/faucet/');

  // async function clickBlankSpace2(newTab) {
  //   const blankSpace2 = '#gatsby-focus-wrapper > div > div > div.css-14y15z9.eh2b2dx0 > div > main > div > div.pageWrap > div.titleWrapper.css-0.ejiqw051 > h1';
  //   await newTab.waitForSelector(blankSpace2, { state: 'visible' });
  //   await newTab.click(blankSpace2);
  // }

  // await clickBlankSpace2(newTab2);
  // await newTab2.evaluate(() => window.scrollBy(0, window.innerHeight+100));
  // await newTab2.waitForTimeout(100);

  // const addressField = ('//*[@id="gatsby-focus-wrapper"]/div/div/div[2]/div/main/div/div[1]/div[2]/div/form/div[3]/div/div/input');
  // await newTab2.waitForSelector(addressField, { state: 'visible' });
  // await newTab2.click(addressField);
  // await newTab2.keyboard.down('Meta'); // Use 'Meta' on Mac
  // await newTab2.keyboard.press('V');
  // await newTab2.keyboard.up('Meta'); // Use 'Meta' on Mac

  // const captcha = '//*[@id="rc-anchor-container"]/div[3]/div[1]/div/div';
  // await newTab2.waitForSelector(captcha, { state: 'visible' });
  // await newTab2.click(captcha);

  // Keeping the browser open for debugging and verifying (remove the timeout or adjust as needed)
  // await page.waitForTimeout(300000); // Adjust the time as needed
  // await new Promise(resolve => { /* never resolves */ });
  // testing
})
