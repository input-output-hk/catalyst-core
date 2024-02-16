import { test, BrowserContext } from '@playwright/test';
import * as fs from 'fs/promises';
import * as path from 'path';


const typhonId = 'KFDNIEFADAANBJODLDOHAEDPHAFOFFOH';
const url = `https://clients2.google.com/service/update2/crx?response=redirect&os=win&arch=x64&os_arch=x86_64&nacl_arch=x86-64&prod=chromiumcrx&prodchannel=beta&prodversion=79.0.3945.53&lang=ru&acceptformat=crx3&x=id%3D${typhonId}%26installsource%3Dondemand%26uc`;
const downloadPath = path.resolve(__dirname, 'extensions');


test('downloadFile test', async ({ page }) => {
    await fs.mkdir(downloadPath, { recursive: true });

    const downloadPromise = new Promise(async (resolve) => {
        page.once('download', async (download) => {
            const filePath = path.join(downloadPath, download.suggestedFilename());
            await download.saveAs(filePath);
            console.log(`file has been downloaded to: ${filePath}`);
            resolve(filePath);
        });
    });

    try {
        await page.goto(url, {
            waitUntil: 'domcontentloaded',
            timeout: 10000
        });
    } catch (error) {
        console.log('navigation caused an exception, likely due to immediate download:', 'directDownload');
    }

    // wait for the download to complete
    const downloadedFilePath = await downloadPromise;
        
    // verify the file exists
    try {
        await fs.access(downloadedFilePath as string); // Type assertion to string
        console.log('file verification succeeded, file exists.');
    } catch {
        console.error('file verification failed, file does not exist.');
        throw new Error('downloaded file does not exist.');
    };

});

