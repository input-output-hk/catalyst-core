import { test, chromium } from '@playwright/test';
import { getAdaliteSeedPhrase } from './seed-phrase';

test('import wallet', async ({ }) => {
    const browser = await chromium.launch({
        headless: false // Set to false to see the browser UI
      });
      const page = await browser.newPage();
      await page.goto('https://adalite.io');

      const continueToAdalite = '//*[@id="root"]/div/span/div[2]/div[2]/div/section/div[2]/button';
      await page.waitForSelector(continueToAdalite, { state: 'visible' });
      await page.click(continueToAdalite);

      const mnemonic = '//*[@id="root"]/div/span/div/div[2]/main/div[2]/div/div[1]/h3';
      await page.waitForSelector(mnemonic, { state: 'visible' });
      await page.click(mnemonic)

      // await page.pause();

      const AdaliteSeedPhrase = getAdaliteSeedPhrase('WALLET2');
      const seedPhraseInput = '//*[@id="mnemonic-submitted"]';
      await page.waitForSelector(seedPhraseInput, { state: 'visible' });
      await page.fill(seedPhraseInput, AdaliteSeedPhrase.seedPhrase);
});