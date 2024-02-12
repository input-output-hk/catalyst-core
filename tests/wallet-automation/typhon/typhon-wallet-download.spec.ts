import { test, expect } from '@playwright/test';
import * as fs from 'fs/promises';
import * as path from 'path';

const url = 'https://clients2.google.com/service/update2/crx?response=redirect&os=win&arch=x64&os_arch=x86_64&nacl_arch=x86-64&prod=chromiumcrx&prodchannel=beta&prodversion=79.0.3945.53&lang=ru&acceptformat=crx3&x=id%3Dkfdniefadaanbjodldohaedphafoffoh%26installsource%3Dondemand%26uc';
const downloadPath = path.resolve(__dirname, 'extensions'); // Ensure this directory exists or add logic to create it

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
    }
});


// import { test, expect } from '@playwright/test';
// import * as fs from 'fs';
// import * as path from 'path';

// // Specifies the use of Firefox for all tests in this file
// test.use({ browserName: 'firefox' });

// const url = 'https://clients2.google.com/service/update2/crx?response=redirect&os=win&arch=x64&os_arch=x86_64&nacl_arch=x86-64&prod=chromiumcrx&prodchannel=beta&prodversion=79.0.3945.53&lang=ru&acceptformat=crx3&x=id%3Dkfdniefadaanbjodldohaedphafoffoh%26installsource%3Dondemand%26uc'; // Update to your target URL
// const downloadPath = './downloads'; // Make sure this directory exists or add logic to create it

// test('downloadFile', async ({ browser }) => {
//     // Setup: Ensure the download directory exists
//     if (!fs.existsSync(downloadPath)) {
//         fs.mkdirSync(downloadPath, { recursive: true });
//     }

//     // Launch a new browser context and page within Firefox
//     const context = await browser.newContext();
//     const page = await context.newPage();

//     // Navigate to the URL
//     await page.goto(url);

//     // Listen for the download event
//     page.on('download', async (download) => {
//         // Use the suggested filename or specify your own
//         const fileName = download.suggestedFilename();
//         // Save the download to the specified path
//         await download.saveAs(path.join(downloadPath, fileName));
//         console.log(`File downloaded to ${path.join(downloadPath, fileName)}`);
        
//         // Verify the file exists (as an example of an assertion you might make)
//         expect(fs.existsSync(path.join(downloadPath, fileName))).toBeTruthy();
//     });

//     // Replace the following line with the actual action that triggers the download
//     // This could be navigating to a URL that starts the download or clicking a download button
//     // Example: await page.click('selector-for-download-link');

//     // Optionally, wait for a specific condition after triggering the download
//     // This is where you would add any additional assertions or cleanup

//     // Close the context
//     await context.close();
// });
