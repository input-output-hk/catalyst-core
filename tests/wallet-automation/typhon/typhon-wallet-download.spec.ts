import { test, BrowserContext } from '@playwright/test';
import * as fs from 'fs/promises';
import * as path from 'path';

const url = 'https://clients2.google.com/service/update2/crx?response=redirect&os=win&arch=x64&os_arch=x86_64&nacl_arch=x86-64&prod=chromiumcrx&prodchannel=beta&prodversion=79.0.3945.53&lang=ru&acceptformat=crx3&x=id%3Dkfdniefadaanbjodldohaedphafoffoh%26installsource%3Dondemand%26uc';
const downloadPath = path.resolve(__dirname, 'extensions');

test('downloadFile test', async ({ page }) => {
    // Ensure the download directory exists
    await fs.mkdir(downloadPath, { recursive: true });

    // Preemptively set up to handle the download
    const downloadPromise = new Promise(async (resolve) => {
        page.once('download', async (download) => {
            const filePath = path.join(downloadPath, download.suggestedFilename());
            await download.saveAs(filePath);
            console.log(`File has been downloaded to: ${filePath}`);
            resolve(filePath);
        });
    });

    try {
        await page.goto(url, {
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
    } catch (error) {
        console.log('Navigation caused an exception, likely due to immediate download:', 'directDownload');
    }

    // Wait for the download to complete
    const downloadedFilePath = await downloadPromise;

    // Verify the file exists
    try {
        await fs.access(downloadedFilePath); // Corrected the variable name here
        console.log('File verification succeeded, file exists.');
    } catch {
        console.error('File verification failed, file does not exist.');
        throw new Error('Downloaded file does not exist.');
    };

});

