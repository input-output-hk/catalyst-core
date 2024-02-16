import { test, chromium } from '@playwright/test';
import { getWalletCredentials } from './credentials';
import { getSeedPhrase } from './seed-phrase';
const path = require('path');
// extension ID for Typhon: kfdniefadaanbjodldohaedphafoffoh

test('import wallet', async ({ }) => {
  const extensionPath: string = path.resolve(__dirname, 'extensions/KFDNIEFADAANBJODLDOHAEDPHAFOFFOH_unzipped');
  const userDataDir = path.resolve(__dirname, 'usrdatadir');

  const browser = await chromium.launchPersistentContext(userDataDir, {
    headless: false, // extensions only work in headful mode
    args: [
      `--disable-extensions-except=${extensionPath}`,
      `--load-extension=${extensionPath}`,
    ],
  });

  const page = await browser.newPage();
  await page.waitForTimeout(1000); // adjust the timeout as needed

  const pages = browser.pages();

  const newTab = pages[pages.length - 1];
  await newTab.bringToFront();

  // interact with elements on the background page
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

  // use the selector to retrieve the element handle
  const elementHandle = await newTab.$(divSelector);
  if (elementHandle) {
    // retrieve the text content of the element
    const textContent = await elementHandle.textContent();
    if (textContent !== null) {
      // remove any formatting that might interfere with parseFloat
      const cleanedText = textContent.replace(/,/g, '').trim();
      const floatValue = parseFloat(cleanedText);
      console.log('ADA:', floatValue)
      if (!isNaN(floatValue)) {
        if (floatValue < 500) {
          console.log('not eligible for voting ☹️');
        } else {
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

//   const logOut = '//*[@id="app"]/div/div/div[3]/div/div/div[1]/div/div/div[2]/div[11]/div[2]';
//   await newTab.waitForSelector(logOut, { state: 'visible' });
//   await newTab.click(logOut);

//   const chooseAccount = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[2]/div';
//   await newTab.waitForSelector(chooseAccount, { state: 'visible' });
//   await newTab.click(chooseAccount);

//   const removeAccount = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[2]/div[4]/button';
//   await newTab.waitForSelector(removeAccount, { state: 'visible' });
//   await newTab.click(removeAccount);

//   const confirmRemove = 'button.btn.bg-primary';
//   await newTab.waitForSelector(confirmRemove, { state: 'visible' });
//   await newTab.click(confirmRemove)

//   const addNew = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[4]';
//   await newTab.waitForSelector(addNew, { state: 'visible' });
//   await newTab.click(addNew);
  
  // keeping the browser open for debugging and verifying (remove the timeout or adjust as needed)
  // await page.waitForTimeout(300000); // Adjust the time as needed
  // await new Promise(resolve => { /* never resolves */ });

});