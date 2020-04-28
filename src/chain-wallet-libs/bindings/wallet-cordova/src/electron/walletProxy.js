const MOCKED_WALLET_PTR = 1000;
const MOCKED_SETTINGS_PTR = 2000;

const BLOCK0 = 'AFIAAAAAA2kAAAAAAAAAAAAAAAD9i29cnYJNuv/jwQQ120w1IjI6I4/L0XRXVp6J3W381AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKYAAAAOAIgAAAAAXpIscABBAQDCAAEDmAAAAAAAAAAKAAAAAAAAAAIAAAAAAAAAZAEEAAAAtAFBFAQEAACowAIIAAAAAAAAAGQCRAABkAABhAAAAGQFgQEEyAAAWvMQekAABSECAAAAAAAAAGQAAAAAAAAADQAAAAAAAAATAAAAAQAAAAMC4OV86zsoMvB+LvBR53K2KoN/ekhsNeOPUb9Va9OrzY7KAW8AAQUAAAAAAA9CQABMgtgYWEKDWBwJkubjlw3QEFW6kZz/W2cKaBP0HFiOtwEjHjzwoQFYHlgcS/9R5uG88kXHvLYQQV+tQnwti4f6yoRSIVlw9gAaZgoUdwAAAAAAAYagAEyC2BhYQoNYHDZX7ZGtLyWtPrxPrsQEd5+Nr6/AP6GBdDx2qmGhAVgeWBzXyZz6E+gcpV0Cb+A5USRkbjmxiMR1+ydlJZddABq3WXfyAAAAAAAAJxAAK4LYGFghg1gcrf9nixGxJ67wwpboi/tHackFKEcWwj5dYyeHh6AAGmP2eccAAAAAAAAAAQBMgtgYWEKDWBxLrr9gAR0FGwIUOjQXUU/tbyXIwD0iUwJaou1foQFYHlgcS/9R5uG88kXHvLUQTHyp7SAeGxpsbfvpPq3uzgAaMYlycAAAAAAAAABkACuC2BhYIYNYHK3/Z4sRsSeu8MKW6Iv7R2nJBShHFsI+XWMnh4egABpj9nnHAU4AAQUAAAAAAA9CQAArgtgYWCGDWBx4P9MAjQ2PtFMohUgTYMtul9x4AciEPzAO1ppWoAAafYOiHQAAAAAAACcQACuC2BhYIYNYHK3/Z4sRsSeu8MKW6Iv7R2nJBShHFsI+XWMnh4egABpj9nnHAAAAAAAAAAEAK4LYGFghg1gceD/TAI0Nj7RTKIVIE2DLbpfceAHIhD8wDtaaVqAAGn2Doh0AAAAAAAAAZABMgtgYWEKDWBz/2F8gzz8on9CR4LAzKF7K1yVJa8VwNaUEuEoQoQFYHlgcS/9R5uG88kXHvLQQUpmlmMUOq6zdD3KBXAFtpwAaV/kGjwAAAAAAAAPyAEyC2BhYQoNYHIRzKQlzhvJjEhUg/Jw2QEe0NimLN8kUihXv3bShAVgeWBzXyZz6E+gc4X9CIeCu1UwIYloKjGh9l0j0YqayABr4Zri5';
const WALLET_VALUE = 1000000 + 10000 + 10000 + 1 + 100;
const YOROI_WALLET = 'neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone';

function walletRestore(successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'string') {
        if (opts[0] === YOROI_WALLET) {
            successCallback(MOCKED_WALLET_PTR);
        }
        else {
            errorCallback('invalid mnemonics');
        }
    }
    else {
        errorCallback('no mnemonics');
    }
}

function walletRetrieveFunds(successCallback, errorCallback, opts) {
    if (opts && typeof (opts[1]) === 'string') {
        if (opts[1] === BLOCK0) {
            successCallback(MOCKED_SETTINGS_PTR);
        }
        else {
            errorCallback('invalid block');
        }
    }
    else {
        errorCallback('no block');
    }
}

function walletTotalFunds(successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'number') {
        if (opts[0] === MOCKED_WALLET_PTR) {
            successCallback(WALLET_VALUE);
        }
        else {
            successCallback(0);
        }
    }
    else {
        errorCallback('no pointer');
    }

}

function walletDelete(successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'number') {
        successCallback();
    }
    else {
        errorCallback();
    }
}

function settingsDelete(successCallback, errorCallback, opts) {
    if (opts && typeof (opts[0]) === 'number') {
        successCallback();
    }
    else {
        errorCallback();
    }
}

bindings = {
    WALLET_RESTORE: walletRestore,
    WALLET_RETRIEVE_FUNDS: walletRetrieveFunds,
    WALLET_TOTAL_FUNDS: walletTotalFunds,
    WALLET_DELETE: walletDelete,
    SETTINGS_DELETE: settingsDelete,
};

require('cordova/exec/proxy').add('WalletPlugin', bindings);