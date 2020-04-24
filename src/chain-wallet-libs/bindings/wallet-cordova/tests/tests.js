/*
 * ADD LICENSE?
 */

const BLOCK0 = '0052000000000369000000000000000000000000fd8b6f5c9d824dbaffe3c10435db4c3522323a238fcbd17457569e89dd6dfcd4000000000000000000000000000000000000000000000000000000000000000000a60000000e0088000000005e922c7000410100c200010398000000000000000a000000000000000200000000000000640104000000b401411404040000a8c00208000000000000006402440001900001840000006405810104c800005af3107a40000521020000000000000064000000000000000d0000000000000013000000010000000302e0e57ceb3b2832f07e2ef051e772b62a837f7a486c35e38f51bf556bd3abcd8eca016f00010500000000000f4240004c82d818584283581c0992e6e3970dd01055ba919cff5b670a6813f41c588eb701231e3cf0a101581e581c4bff51e6e1bcf245c7bcb610415fad427c2d8b87faca8452215970f6001a660a147700000000000186a0004c82d818584283581c3657ed91ad2f25ad3ebc4faec404779f8dafafc03fa181743c76aa61a101581e581cd7c99cfa13e81ca55d026fe0395124646e39b188c475fb276525975d001ab75977f20000000000002710002b82d818582183581cadff678b11b127aef0c296e88bfb4769c905284716c23e5d63278787a0001a63f679c70000000000000001004c82d818584283581c4baebf60011d051b02143a3417514fed6f25c8c03d2253025aa2ed5fa101581e581c4bff51e6e1bcf245c7bcb5104c7ca9ed201e1b1a6c6dfbe93eadeece001a318972700000000000000064002b82d818582183581cadff678b11b127aef0c296e88bfb4769c905284716c23e5d63278787a0001a63f679c7014e00010500000000000f4240002b82d818582183581c783fd3008d0d8fb4532885481360cb6e97dc7801c8843f300ed69a56a0001a7d83a21d0000000000002710002b82d818582183581cadff678b11b127aef0c296e88bfb4769c905284716c23e5d63278787a0001a63f679c70000000000000001002b82d818582183581c783fd3008d0d8fb4532885481360cb6e97dc7801c8843f300ed69a56a0001a7d83a21d0000000000000064004c82d818584283581cffd85f20cf3f289fd091e0b033285ecad725496bc57035a504b84a10a101581e581c4bff51e6e1bcf245c7bcb4105299a598c50eabacdd0f72815c016da7001a57f9068f00000000000003f2004c82d818584283581c847329097386f263121520fc9c364047b436298b37c9148a15efddb4a101581e581cd7c99cfa13e81ce17f4221e0aed54c08625a0a8c687d9748f462a6b2001af866b8b9';

function hexStringToBytes(string) {
    const bytes = [];
    for (let c = 0; c < string.length; c += 2)
        bytes.push(parseInt(string.substr(c, 2), 16));
    return Uint8Array.from(bytes);
}


const WALLET_VALUE = 1000000 + 10000 + 10000 + 1 + 100;
const YOROI_WALLET = 'neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone';

exports.defineAutoTests = function () {
    describe('primitive mappings', function () {
        it('should exist', function () {
            expect(window.wallet).toBeDefined();
        });

        // TODO: untangle this nesting hell. I still don't know if I can use promises/async here
        it('should recover wallet', function (done) {
            window.wallet.walletRestore(YOROI_WALLET, wallet => {
                expect(wallet != 0).toBe(true);
                window.wallet.walletRetrieveFunds(wallet, hexStringToBytes(BLOCK0), settings => {
                    expect(settings != 0).toBe(true);
                    window.wallet.walletTotalFunds(wallet, retrievedFunds => {
                        expect(parseInt(retrievedFunds)).toBe(WALLET_VALUE);
                        window.wallet.settingsDelete(settings, () => {
                            window.wallet.walletDelete(wallet, () => {
                                done()
                            }, err => { done.fail(new Error(`couldn't delete wallet ${err}`)) })
                        }, err => { done.fail(new Error(`couldn't delete settings ${err}`)) });
                    }, err => { done.fail(new Error(`couldn't get total funds ${err}`)) });
                }, err => done.fail(new Error(`could not retrieve funds ${err}`)))
            }, err => {
                done.fail(new Error(`could not create wallet ${err}`));
            });
        });
    });
};

// TODO: untangle this nesting hell. I still don't know if I can use promises/async here
function restoreWallet(mnemonics, hexBlock, callBack) {
    window.wallet.walletRestore(mnemonics, wallet => {
        console.log(wallet);
        window.wallet.walletRetrieveFunds(wallet, hexStringToBytes(hexBlock), settings => {
            console.log(settings);
            window.wallet.walletTotalFunds(wallet, retrievedFunds => {
                window.wallet.settingsDelete(settings, () => {
                    window.wallet.walletDelete(wallet, () => {
                        callBack(undefined, retrievedFunds);
                    }, err => { callBack(new Error(`couldn't delete wallet ${err}`)) })
                }, err => { callBack(new Error(`couldn't delete settings ${err}`)) });
            }, err => { callBack(new Error(`couldn't get total funds ${err}`)) });
        }, err => {
            callBack(new Error(`could not retrieve funds ${err}`))
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
        '<div id="get_funds"> </div>';

    contentEl.innerHTML = '<div id="info"></div>' + form;

    createActionButton(
        'get funds',
        function () {
            clearLog();
            const mnemonics = document.getElementById('mnemonics').value;
            const block = document.getElementById('block').value;
            restoreWallet(mnemonics, block, (error, value) => {
                if (error) {
                    logMessage(`Error: ${error}`, null, '\t');
                }
                else {
                    logMessage(`Funds: ${value}`, null, '\t');
                }
            });
        },
        'get_funds'
    );
};
