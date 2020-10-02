import "core-js/stable";
import "regenerator-runtime/runtime";

const primitives = require('wallet-cordova-plugin.wallet');
const BLOCK0 = '00520000000003c1000000000000000000000000da1006dbf989038c7df6048fb59ae3359e67184ba193e734db61718126a56948000000000000000000000000000000000000000000000000000000000000000000a60000000e0088000000005e922c7000410100c200010398000000000000000a000000000000000200000000000000640104000000b401411404040000a8c00208000000000000006402440001900001840000006405810104c800005af3107a40000521020000000000000064000000000000000d0000000000000013000000010000000302e0e57ceb3b2832f07e2ef051e772b62a837f7a486c35e38f51bf556bd3abcd8eca016f00010500000000000f4240004c82d818584283581c0992e6e3970dd01055ba919cff5b670a6813f41c588eb701231e3cf0a101581e581c4bff51e6e1bcf245c7bcb610415fad427c2d8b87faca8452215970f6001a660a147700000000000186a0004c82d818584283581c3657ed91ad2f25ad3ebc4faec404779f8dafafc03fa181743c76aa61a101581e581cd7c99cfa13e81ca55d026fe0395124646e39b188c475fb276525975d001ab75977f20000000000002710002b82d818582183581cadff678b11b127aef0c296e88bfb4769c905284716c23e5d63278787a0001a63f679c70000000000000001004c82d818584283581c4baebf60011d051b02143a3417514fed6f25c8c03d2253025aa2ed5fa101581e581c4bff51e6e1bcf245c7bcb5104c7ca9ed201e1b1a6c6dfbe93eadeece001a318972700000000000000064002b82d818582183581cadff678b11b127aef0c296e88bfb4769c905284716c23e5d63278787a0001a63f679c7014e00010500000000000f4240002b82d818582183581c783fd3008d0d8fb4532885481360cb6e97dc7801c8843f300ed69a56a0001a7d83a21d0000000000002710002b82d818582183581cadff678b11b127aef0c296e88bfb4769c905284716c23e5d63278787a0001a63f679c70000000000000001002b82d818582183581c783fd3008d0d8fb4532885481360cb6e97dc7801c8843f300ed69a56a0001a7d83a21d0000000000000064004c82d818584283581cffd85f20cf3f289fd091e0b033285ecad725496bc57035a504b84a10a101581e581c4bff51e6e1bcf245c7bcb4105299a598c50eabacdd0f72815c016da7001a57f9068f00000000000003f2004c82d818584283581c847329097386f263121520fc9c364047b436298b37c9148a15efddb4a101581e581cd7c99cfa13e81ce17f4221e0aed54c08625a0a8c687d9748f462a6b2001af866b8b9005600020002032fc94e416c9cb9b3f7ea999846acd31fe466c092e1c0d8b6634074def8f9e1e800000000000003e8033005131602e42423f16621bbe6100bfa7a1a69aee5fd6118dc588fe6f2a8af7c0000000000002710';

function hexStringToBytes (string) {
    const bytes = [];
    for (let c = 0; c < string.length; c += 2) { bytes.push(parseInt(string.substr(c, 2), 16)); }
    return Uint8Array.from(bytes);
}

const WALLET_VALUE = 1000000 + 10000 + 10000 + 1 + 100;
const YOROI_WALLET = 'neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone';

const ENCRYPTED_WALLET = '017b938f189c7d1d9e4c75b02710a9c9a6b287b6ca55d624001828cba8aeb3a9d4c2a86261016693c7e05fb281f012fb2d7af44484da09c4d7b2dea6585965a4cc208d2b2fb1aa5ba6338520b3aa9c4f908fdd62816ebe01f496f8b4fc0344892fe245db072d054c3dedff926320589231298e216506c1f6858c5dba915c959a98ba0d0e3995aef91d4216b5172dedf2736b451d452916b81532eb7f8487e9f88a2de4f9261d0a0ddf11698796ad8b6894908024ebc4be9bba985ef9c0f2f71afce0b37520c66938313f6bf81b3fc24f5c93d216cd2528dabc716b8093359fda84db4e58d876d215713f2db000';
const ACCOUNT = 'c86596c2d1208885db1fe3658406aa0f7cc7b8e13c362fe46a6db277fc5064583e487588c98a6c36e2e7445c0add36f83f171cb5ccfd815509d19cd38ecb0af3';
const KEY1 = '301559ccb2d4cc7e9e54a6f55a80960bb691b7b1409549ef8695a92ea41a6f4f405231a806c2e7b9d0db30e15ee0cc1f261c1b9f9615636b48bd89fe7be6ea1f';
const KEY2 = 'a8b6bdf080c74fbc31337ede4b6692c2ebed7e34af6d98b7bbcd478cf07b0d5ed93f7e9d4aa3afde321ae1abb61b8344c243c9d0b407cbf8917db6df2c653dea';
const PASSWORD = new Uint8Array(4);
PASSWORD[0] = 1;
PASSWORD[1] = 2;
PASSWORD[2] = 3;
PASSWORD[3] = 4;

/**
 * helper to convert the cordova-callback-style to promises, to make tests simpler
 * @param {function} f
 * @returns {function}
 */
function promisify (f) {
    const newFunction = function () {
        const args = Array.prototype.slice.call(arguments);
        return new Promise(function (resolve, reject) {
            const success = function () {
                resolve(arguments[0]);
            };

            const error = function () {
                reject(arguments[0]);
            };

            args.push(success);
            args.push(error);

            f.apply(primitives, args);
        });
    };
    return newFunction;
}

const restoreWallet = promisify(primitives.walletRestore);
const importKeys = promisify(primitives.walletImportKeys);
const retrieveFunds = promisify(primitives.walletRetrieveFunds);
const totalFunds = promisify(primitives.walletTotalFunds);
const convertWallet = promisify(primitives.walletConvert);
const setState = promisify(primitives.walletSetState);
const deleteWallet = promisify(primitives.walletDelete);
const deleteSettings = promisify(primitives.settingsDelete);
const deleteConversion = promisify(primitives.conversionDelete);
const deleteProposal = promisify(primitives.proposalDelete);
const deletePending = promisify(primitives.pendingTransactionsDelete);
const conversionGetTransactionAt = promisify(primitives.conversionTransactionsGet);
const conversionGetIgnored = promisify(primitives.conversionGetIgnored);
const proposalNew = promisify(primitives.proposalNew);
const walletVote = promisify(primitives.walletVote);
const walletSetState = promisify(primitives.walletSetState);
const walletConfirmTransaction = promisify(primitives.walletConfirmTransaction);
const walletPendingTransactions = promisify(primitives.walletPendingTransactions);
const pendingTransactionsGet = promisify(primitives.pendingTransactionsGet);
const symmetricCipherDecrypt = promisify(primitives.symmetricCipherDecrypt);

function conversionGetTransactions (conversion) {
    return new Promise(function (resolve, reject) {
        primitives.conversionTransactionsSize(conversion, function (size) {
            const transactions = [];
            for (let i = 0; i < size; ++i) {
                transactions.push(conversionGetTransactionAt(conversion, i));
            }
            Promise.all(transactions).then(
                function (array) {
                    resolve(array);
                }
            ).catch(function (err) {
                reject(err);
            });
        }, function (err) {
            reject(err);
        });
    }
    );
}

function pendingTransactionsGetAll (pendingTransactions) {
    return new Promise(function (resolve, reject) {
        primitives.pendingTransactionsSize(pendingTransactions, function (size) {
            const transactions = [];
            for (let i = 0; i < size; ++i) {
                transactions.push(pendingTransactionsGet(pendingTransactions, i));
            }
            Promise.all(transactions).then(
                function (array) {
                    resolve(array);
                }
            ).catch(function (err) {
                reject(err);
            });
        }, function (err) {
            reject(err);
        });
    }
    );
}

function getPendingTransactions (walletPtr) {
    return walletPendingTransactions(walletPtr).then(
        function (pendingPtr) {
            return pendingTransactionsGetAll(pendingPtr).then(pending => {
                return deletePending(pendingPtr).then(() => pending);
            });
        });
}

exports.defineAutoTests = function () {
    describe('primitive mappings', function () {
        it('clobber should exist', function () {
            expect(window.wallet).toBeDefined();
        });

        it('should recover wallet', function (done) {
            let walletPtr;
            let settingsPtr;

            const accountKey = new Uint8Array([
                200, 101, 150, 194, 209, 32, 136, 133, 219, 31, 227, 101, 132, 6, 170, 15, 124, 199,
                184, 225, 60, 54, 47, 228, 106, 109, 178, 119, 252, 80, 100, 88, 62, 72, 117, 136, 201,
                138, 108, 54, 226, 231, 68, 92, 10, 221, 54, 248, 63, 23, 28, 181, 204, 253, 129, 85,
                9, 209, 156, 211, 142, 203, 10, 243
            ]);
            const utxoKeys = new Uint8Array([
                48, 21, 89, 204, 178, 212, 204, 126, 158, 84, 166, 245, 90, 128, 150, 11, 182, 145,
                183, 177, 64, 149, 73, 239, 134, 149, 169, 46, 164, 26, 111, 79, 64, 82, 49, 168, 6,
                194, 231, 185, 208, 219, 48, 225, 94, 224, 204, 31, 38, 28, 27, 159, 150, 21, 99, 107,
                72, 189, 137, 254, 123, 230, 234, 31,
                168, 182, 189, 240, 128, 199, 79, 188, 49, 51, 126, 222, 75, 102, 146, 194, 235, 237,
                126, 52, 175, 109, 152, 183, 187, 205, 71, 140, 240, 123, 13, 94, 217, 63, 126, 157,
                74, 163, 175, 222, 50, 26, 225, 171, 182, 27, 131, 68, 194, 67, 201, 208, 180, 7, 203,
                248, 145, 125, 182, 223, 44, 101, 61, 234
            ]);

            importKeys(accountKey, utxoKeys)
                .then(function (wallet) {
                    expect(wallet !== 0).toBe(true);
                    walletPtr = wallet;
                    return retrieveFunds(wallet, hexStringToBytes(BLOCK0));
                })
                .then(function (settings) {
                    expect(settings !== 0).toBe(true);
                    settingsPtr = settings;
                    return totalFunds(walletPtr);
                })
                .then(function (funds) {
                    expect(parseInt(funds)).toBe(10000 + 1000);
                    return deleteSettings(settingsPtr);
                })
                .then(function () {
                    return deleteWallet(walletPtr);
                })
                .catch(function (err) {
                    done.fail('could not restore wallet' + err);
                })
                .then(function () {
                    done();
                });
        });

        it('should import keys', function (done) {
            let walletPtr;
            let settingsPtr;
            restoreWallet(YOROI_WALLET)
                .then(function (wallet) {
                    expect(wallet !== 0).toBe(true);
                    walletPtr = wallet;
                    return retrieveFunds(wallet, hexStringToBytes(BLOCK0));
                })
                .then(function (settings) {
                    expect(settings !== 0).toBe(true);
                    settingsPtr = settings;
                    return totalFunds(walletPtr);
                })
                .then(function (funds) {
                    expect(parseInt(funds)).toBe(WALLET_VALUE);
                    return deleteSettings(settingsPtr);
                })
                .then(function () {
                    return deleteWallet(walletPtr);
                })
                .catch(function (err) {
                    done.fail('could not restore wallet' + err);
                })
                .then(function () {
                    done();
                });
        });

        // there is nothing we can assert here, I think
        it('should be able to set state', function (done) {
            restoreWallet(YOROI_WALLET)
                .then(function (wallet) {
                    const value = 1000;
                    const counter = 2;
                    setState(wallet, value, counter);
                })
                .catch(function (err) {
                    done.fail('could not set wallet state' + err);
                })
                .then(function () {
                    done();
                });
        });

        it('should fail with invalid mnemonics', function (done) {
            restoreWallet('invalidmnemonics')
                .then(
                    function () {
                        done.fail('Invalid mnemonics should fail');
                    })
                .catch(function () {
                    done();
                });
        });

        it('should fail with invalid block', function (done) {
            restoreWallet(YOROI_WALLET)
                .then(function (wallet) {
                    return retrieveFunds(wallet, [0, 0, 0, 0]);
                })
                .then(function () {
                    done.fail('Invalid block should fail');
                })
                .catch(function () {
                    done();
                });
        });

        it('get conversion transaction', function (done) {
            let walletPtr;
            let settingsPtr;
            let conversionPtr;
            restoreWallet(YOROI_WALLET)
                .then(function (wallet) {
                    expect(wallet !== 0).toBe(true);
                    walletPtr = wallet;
                    return retrieveFunds(wallet, hexStringToBytes(BLOCK0));
                })
                .then(function (settings) {
                    settingsPtr = settings;
                    return convertWallet(walletPtr, settingsPtr);
                })
                .then(function (conversion) {
                    conversionPtr = conversion;
                    return conversionGetTransactions(conversion);
                })
                .then(function (transactions) {
                    expect(transactions.length).toBe(1);
                    return conversionGetIgnored(conversionPtr);
                })
                .then(function (ignored_value) {
                    expect(Number(ignored_value.value)).toBe(1);
                    expect(ignored_value.ignored).toBe(1);
                })
                .then(function () {
                    return getPendingTransactions(walletPtr);
                })
                .then(function (pending) {
                    expect(pending.length).toBe(1);
                    return walletConfirmTransaction(walletPtr, new Uint8Array(pending[0])).then(
                        function () { return getPendingTransactions(walletPtr); }
                    );
                })
                .then(function (pending) {
                    expect(pending.length).toBe(0);
                })
                .then(function () {
                    return deleteSettings(settingsPtr);
                })
                .then(function () {
                    return deleteWallet(walletPtr);
                })
                .then(function () {
                    return deleteConversion(conversionPtr);
                })
                .catch(function (err) {
                    done.fail('could not restore wallet' + err);
                })
                .then(function () {
                    done();
                });
        });

        it('can cast vote', function (done) {
            let walletPtr;
            let settingsPtr;
            let proposalPtr;

            const array = new Array(32);
            for (let index = 0; index < array.length; index++) {
                array[index] = index;
            }

            const votePlanId = new Uint8Array(array);
            const payloadType = primitives.PayloadType.PUBLIC;
            const index = 0;
            const numChoices = 3;

            proposalNew(votePlanId, payloadType, index, numChoices)
                .catch(function (err) {
                    done.fail('could not create proposal ' + err);
                })
                .then(function (proposal) {
                    proposalPtr = proposal;
                    return restoreWallet(YOROI_WALLET);
                })
                .then(function (wallet) {
                    walletPtr = wallet;
                    return retrieveFunds(wallet, hexStringToBytes(BLOCK0));
                })
                .then(function (settings) {
                    settingsPtr = settings;
                    return walletSetState(walletPtr, 1000000, 1);
                })
                .then(function () {
                    const tx = walletVote(walletPtr, settingsPtr, proposalPtr, 0);
                    return tx.then(function (tx) {
                        console.log(tx);
                    });
                })
                .then(function () {
                    return deleteSettings(settingsPtr);
                })
                .then(function () {
                    return deleteWallet(walletPtr);
                })
                .then(function () {
                    return deleteProposal(proposalPtr);
                })
                .catch(function (err) {
                    done.fail('could not restore wallet' + err);
                })
                .then(function () {
                    done();
                });
        });

        it('decrypts keys correctly', function (done) {
            console.log(hexStringToBytes(ENCRYPTED_WALLET));
            symmetricCipherDecrypt(PASSWORD, hexStringToBytes(ENCRYPTED_WALLET))
                .then(function (decryptedKeys) {
                    console.log(new Uint8Array(decryptedKeys));
                    const account = decryptedKeys.slice(0 * 64, 1 * 64);
                    const key1 = decryptedKeys.slice(1 * 64, 2 * 64);
                    const key2 = decryptedKeys.slice(2 * 64, 3 * 64);

                    if (!uint8ArrayEquals(hexStringToBytes(ACCOUNT), new Uint8Array(account))) {
                        done.fail('wrong expected account');
                    }
                    if (!uint8ArrayEquals(hexStringToBytes(KEY1), new Uint8Array(key1))) {
                        done.fail('wrong expected key1');
                    }
                    if (!uint8ArrayEquals(hexStringToBytes(KEY2), new Uint8Array(key2))) {
                        done.fail('wrong expected key2');
                    }
                })
                .catch(function (err) {
                    done.fail('could not restore wallet' + err);
                })
                .then(function () {
                    done();
                });
        });

        it('decrypt QR', function (done) {
            const encrypted = '01f4c73628793ad51150f3885c706474fb35f33d85c5e3d183773f8138f5d4294807ff71fd9de1fb1a31656520eb72ebefff19f563d51d4b70c1fed789ef73ca70e9870eff4516b2a2550978ede3062e3cf8dbb40408e6f977f7f7a3b92756902b37ca172dc8b2cb09456ee891';
            const expected = 'c86596c2d1208885db1fe3658406aa0f7cc7b8e13c362fe46a6db277fc5064583e487588c98a6c36e2e7445c0add36f83f171cb5ccfd815509d19cd38ecb0af3';
            const password = new Uint8Array(4);
            password[0] = 1;
            password[1] = 2;
            password[2] = 3;
            password[3] = 4;

            symmetricCipherDecrypt(password, hexStringToBytes(encrypted))
                .then(function (decrypted) {
                    if (!uint8ArrayEquals(hexStringToBytes(expected), new Uint8Array(decrypted))) {
                        done.fail('decryption failed');
                    }
                })
                .catch(function (err) {
                    done.fail('could not restore wallet' + err);
                })
                .then(function () {
                    done();
                });
        });
    });
};

function uint8ArrayEquals (a, b) {
    const length = a.length === b.length;
    let elements = true;

    for (let i = 0; i < a.length; i++) {
        elements = elements && a[i] === b[i];
    }

    return length && elements;
}

// TODO: untangle this nesting hell. I still don't know if I can use promises/async here
function restoreManualInputWallet (mnemonics, hexBlock, callBack) {
    window.wallet.walletRestore(mnemonics, wallet => {
        window.wallet.walletRetrieveFunds(wallet, hexStringToBytes(hexBlock), settings => {
            window.wallet.walletTotalFunds(wallet, retrievedFunds => {
                window.wallet.settingsDelete(settings, () => {
                    window.wallet.walletDelete(wallet, () => {
                        callBack(undefined, retrievedFunds);
                    }, err => { callBack(new Error(`couldn't delete wallet ${err}`)); });
                }, err => { callBack(new Error(`couldn't delete settings ${err}`)); });
            }, err => { callBack(new Error(`couldn't get total funds ${err}`)); });
        }, err => {
            callBack(new Error(`could not retrieve funds ${err}`));
        });
    }, err => {
        callBack(new Error(`could not create wallet ${err}`));
    });
}

function getAccountId (mnemonics, callBack) {
    primitives.walletRestore(mnemonics, wallet => {
        primitives.walletId(wallet, function (id) {
            callBack(undefined, hex(id));
        }, function (err) {
            callBack(new Error(`could not get account id ${err}`));
        });
    }, err => {
        callBack(new Error(`could not create wallet ${err}`));
    });
}

exports.defineManualTests = function (contentEl, createActionButton) {
    var logMessage = function (message, color) {
        var log = document.getElementById('info');
        var logLine = document.createElement('div');
        if (color) {
            logLine.style.color = color;
        }
        logLine.innerHTML = message;
        log.appendChild(logLine);
    };

    var clearLog = function () {
        var log = document.getElementById('info');
        log.innerHTML = '';
    };

    const form =
        '<div> <label> mnemonics </label> <textarea id="mnemonics" rows="1"></textarea> </div>' +
        '<div> <label> block(hex) </label> <textarea id="block" rows="1"></textarea> </div>' +
        '<div id="get_funds"> </div>' +
        '<div id="account"> </div>';

    contentEl.innerHTML = '<div id="info"></div>' + form;

    createActionButton(
        'get funds',
        function () {
            clearLog();
            const mnemonics = document.getElementById('mnemonics').value;
            const block = document.getElementById('block').value;
            restoreManualInputWallet(mnemonics, block, (error, value) => {
                if (error) {
                    logMessage(`Error: ${error}`, null);
                } else {
                    logMessage(`Funds: ${value}`, null);
                }
            });
        },
        'get_funds'
    );

    createActionButton(
        'get account id',
        function () {
            clearLog();
            const mnemonics = document.getElementById('mnemonics').value;
            getAccountId(mnemonics, (error, value) => {
                if (error) {
                    logMessage(`Error: ${error}`, null);
                } else {
                    logMessage(`account id: ${value}`, null);
                }
            });
        },
        'account'
    );
};

// copypasted ArrayBuffer to Hex string function
const byteToHex = [];

for (let n = 0; n <= 0xff; ++n) {
    const hexOctet = ('0' + n.toString(16)).slice(-2);
    byteToHex.push(hexOctet);
}

function hex (arrayBuffer) {
    const buff = new Uint8Array(arrayBuffer);
    const hexOctets = [];

    for (let i = 0; i < buff.length; ++i) { hexOctets.push(byteToHex[buff[i]]); }

    return hexOctets.join('');
}
