import { test, chromium, BrowserContext, Page } from '@playwright/test';
import { getWalletCredentials } from './credentials';
import { getSeedPhrase } from './seed-phrase';


test('import wallet', async ({ }) => {
  const path = require('path');
  const extensionPath = path.join(__dirname, 'extensions');
  const userDataDir = path.join(__dirname, 'usrdatadir');

  // Launch the browser context with your extension
  const browserContext = await chromium.launchPersistentContext(userDataDir, {
    headless: false, // Necessary for extensions
    args: [
      `--disable-extensions-except=${extensionPath}`,
      `--load-extension=${extensionPath}`,
    ],
  });

  // Attempt to retrieve any existing background pages of the extension
  let backgroundPage = browserContext.backgroundPages()[0];

  // If no background page is found, wait for it to be created
  if (!backgroundPage) {
    backgroundPage = await browserContext.waitForEvent('backgroundpage');
  }

  // Ensure the background page is accessed correctly
  if (backgroundPage) {
    // Example: Get the title of the background page
    const title = await backgroundPage.title();
    console.log(`Title of the background page: ${title}`);

    // Interact with elements on the background page
    const firstButtonSelector = '//*[@id="headlessui-menu-button-1"]';
    await backgroundPage.waitForSelector(firstButtonSelector, { state: 'visible' });
    await backgroundPage.click(firstButtonSelector);
  } else {
    console.log('Background page not found');
  }

  const secondButtonSelector = '#headlessui-menu-item-6';
  await backgroundPage.waitForSelector(secondButtonSelector, { state: 'visible' });
  await backgroundPage.click(secondButtonSelector);

  const thirdButtonSelector = '//*[text()="Import"]';
  await backgroundPage.waitForSelector(thirdButtonSelector, { state: 'visible' });
  await backgroundPage.click(thirdButtonSelector);

  const WalletCredentials = getWalletCredentials('WALLET1');
  const usernameInput = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(2) > div > input';
  const passwordInput = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(2) > div > div:nth-child(2) > input';
  const cfpwInput = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(2) > div > div:nth-child(3) > input';
  await backgroundPage.waitForSelector(usernameInput, { state: 'visible' });
  await backgroundPage.waitForSelector(passwordInput, { state: 'visible' });
  await backgroundPage.waitForSelector(cfpwInput, { state: 'visible' });
  await backgroundPage.fill(usernameInput, WalletCredentials.username);
  await backgroundPage.fill(passwordInput, WalletCredentials.password);
  await backgroundPage.fill(cfpwInput, WalletCredentials.password);

  const agreeToTC = '#termsAndConditions'
  await backgroundPage.waitForSelector(agreeToTC, { state: 'visible' });
  await backgroundPage.click(agreeToTC);

  const continueButton = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(2) > div > button';
  await backgroundPage.waitForSelector(continueButton, { state: 'visible' });
  await backgroundPage.click(continueButton);

  async function clickBlankSpace(backgroundPage) {
    const blankSpace = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div.flex.justify-between.items-start > div.flex-initial.flex.flex-col.mr-2 > span.text-primary.font-medium.text-xl';
    await backgroundPage.waitForSelector(blankSpace, { state: 'visible' });
    await backgroundPage.click(blankSpace);
  }

  const seedPhrase = getSeedPhrase();

  for (let i = 0; i < seedPhrase.length; i++) {
    const ftSeedPhraseSelector = `#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div:nth-child(1) > div:nth-child(2) > div > div > div > div > div:nth-child(${i + 1}) > div > input`;
    await backgroundPage.waitForSelector(ftSeedPhraseSelector, { state: 'visible' });
    await backgroundPage.fill(ftSeedPhraseSelector, seedPhrase[i]);
  }

  const unlockWallet = '#app > div > div > div.flex-grow.overflow-auto > div > div.my-5.flex.justify-center.py-16 > div > div > div > div.mt-6.text-center.flex.justify-center > button';
  await clickBlankSpace(backgroundPage);
  await backgroundPage.waitForSelector(unlockWallet, { state: 'visible' });
  await backgroundPage.click(unlockWallet);

  const divSelector = '//*[@id="lc"]/div[2]/div[1]/div[2]/div/div[1]/div[1]/div/div[2]/div[1]/div/span[1]';
  await backgroundPage.waitForSelector(divSelector, { state: 'visible' });

  // Use the selector to retrieve the element handle
  const elementHandle = await backgroundPage.$(divSelector);
  if (elementHandle) {
    // Retrieve the text content of the element
    const textContent = await elementHandle.textContent();
    if (textContent !== null) {
      // Remove any formatting that might interfere with parseFloat
      const cleanedText = textContent.replace(/,/g, '').trim();
      const floatValue = parseFloat(cleanedText);
      console.log('ADA:', floatValue)
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

  const logOut = '//*[@id="app"]/div/div/div[3]/div/div/div[1]/div/div/div[2]/div[11]/div[2]';
  await backgroundPage.waitForSelector(logOut, { state: 'visible' });
  await backgroundPage.click(logOut);

  const chooseAccount = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[2]/div';
  await backgroundPage.waitForSelector(chooseAccount, { state: 'visible' });
  await backgroundPage.click(chooseAccount);

  const removeAccount = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[2]/div[4]/button';
  await backgroundPage.waitForSelector(removeAccount, { state: 'visible' });
  await backgroundPage.click(removeAccount);

  const confirmRemove = 'button.btn.bg-primary';
  await backgroundPage.waitForSelector(confirmRemove, {state: 'visible'});
  await backgroundPage.click(confirmRemove)

  const addNew = '//*[@id="app"]/div/div/div[3]/div/div[2]/div/div/div[4]';
  await backgroundPage.waitForSelector(addNew, { state: 'visible' });
  await backgroundPage.click(addNew);

});
  
  // const copyAddress = '#receiveAddress > div > div.grow > button';
  // await backgroundPage.waitForSelector(copyAddress, { state: 'visible' });
  // await backgroundPage.click(copyAddress);
  // await backgroundPage.waitForTimeout(500);
  // const copiedText = await backgroundPage.evaluate(() => navigator.clipboard.readText());
  // console.log('Copied Address:', copiedText);

  // const backgroundPage2 = await browser.newPage();
  // await backgroundPage2.goto('https://docs.cardano.org/cardano-testnet/tools/faucet/');

  // async function clickBlankSpace2(backgroundPage) {
  //   const blankSpace2 = '#gatsby-focus-wrapper > div > div > div.css-14y15z9.eh2b2dx0 > div > main > div > div.pageWrap > div.titleWrapper.css-0.ejiqw051 > h1';
  //   await backgroundPage.waitForSelector(blankSpace2, { state: 'visible' });
  //   await backgroundPage.click(blankSpace2);
  // }

  // await clickBlankSpace2(backgroundPage2);
  // await backgroundPage2.evaluate(() => window.scrollBy(0, window.innerHeight+100));
  // await backgroundPage2.waitForTimeout(100);

  // const addressField = ('//*[@id="gatsby-focus-wrapper"]/div/div/div[2]/div/main/div/div[1]/div[2]/div/form/div[3]/div/div/input');
  // await backgroundPage2.waitForSelector(addressField, { state: 'visible' });
  // await backgroundPage2.click(addressField);
  // await backgroundPage2.keyboard.down('Meta'); // Use 'Meta' on Mac
  // await backgroundPage2.keyboard.press('V');
  // await backgroundPage2.keyboard.up('Meta'); // Use 'Meta' on Mac

  // const captcha = '//*[@id="rc-anchor-container"]/div[3]/div[1]/div/div';
  // await backgroundPage2.waitForSelector(captcha, { state: 'visible' });
  // await backgroundPage2.click(captcha);

  // Keeping the browser open for debugging and verifying (remove the timeout or adjust as needed)
  // await page.waitForTimeout(300000); // Adjust the time as needed
  // await new Promise(resolve => { /* never resolves */ });
  // testing

