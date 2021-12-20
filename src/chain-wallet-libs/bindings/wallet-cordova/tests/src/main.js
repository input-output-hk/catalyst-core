import "core-js/stable";
import "regenerator-runtime/runtime";

const primitives = require('wallet-cordova-plugin.wallet');

const { hexStringToBytes, promisify, uint8ArrayEquals } = require('./src/utils.js');
const keys = require('../../../test-vectors/free_keys/keys.json');
const genesis = require('../../../test-vectors/block0.json');
const BLOCK0 = genesis.hex;
const BLOCK0_ID = genesis.id;
const ENCRYPTED_WALLET = keys.encrypted;
const PASSWORD = new Uint8Array(4);
PASSWORD[0] = keys.password[0];
PASSWORD[1] = keys.password[1];
PASSWORD[2] = keys.password[2];
PASSWORD[3] = keys.password[3];
const VOTE_ENCRYPTION_KEY = 'ristretto255_votepk1nc988wtjlrm5k0z43088p0rrvd5yhvc96k7zh99p6w74gupxggtqyx4792';

// TODO: write settings getter for this
const BLOCK0_DATE = 1586637936;

let promisifyP = f => promisify(primitives, f)
const restoreWallet = promisifyP(primitives.walletRestore);
const importKeys = promisifyP(primitives.walletImportKeys);
const retrieveFunds = promisifyP(primitives.walletRetrieveFunds);
const spendingCounter = promisifyP(primitives.walletSpendingCounter);
const walletId = promisifyP(primitives.walletId);
const totalFunds = promisifyP(primitives.walletTotalFunds);
const convertWallet = promisifyP(primitives.walletConvert);
const setState = promisifyP(primitives.walletSetState);
const deleteWallet = promisifyP(primitives.walletDelete);
const deleteSettings = promisifyP(primitives.settingsDelete);
const deleteConversion = promisifyP(primitives.conversionDelete);
const deleteProposal = promisifyP(primitives.proposalDelete);
const deletePending = promisifyP(primitives.pendingTransactionsDelete);
const conversionGetTransactionAt = promisifyP(primitives.conversionTransactionsGet);
const conversionGetIgnored = promisifyP(primitives.conversionGetIgnored);
const proposalNewPublic = promisifyP(primitives.proposalNewPublic);
const proposalNewPrivate = promisifyP(primitives.proposalNewPrivate);
const walletVote = promisifyP(primitives.walletVote);
const walletSetState = promisifyP(primitives.walletSetState);
const walletConfirmTransaction = promisifyP(primitives.walletConfirmTransaction);
const walletPendingTransactions = promisifyP(primitives.walletPendingTransactions);
const pendingTransactionsGet = promisifyP(primitives.pendingTransactionsGet);
const symmetricCipherDecrypt = promisifyP(primitives.symmetricCipherDecrypt);
const settingsGet = promisifyP(primitives.settingsGet);
const settingsNew = promisifyP(primitives.settingsNew);
const fragmentId = promisifyP(primitives.fragmentId);
const blockDateFromSystemTime = promisifyP(primitives.blockDateFromSystemTime);
const maxExpirationDate = promisifyP(primitives.maxExpirationDate);

async function walletFromFile() {
    const accountKey = hexStringToBytes(keys.account.private_key);
    const utxoKeys = keys.utxo_keys.map(utxo => utxo.private_key).reduce((accum, current) => accum + current);
    return await importKeys(accountKey, hexStringToBytes(utxoKeys));
}

const tests = [
    ['should recover wallet', async function () {
        const walletPtr = await walletFromFile();
        expect(walletPtr !== 0).toBe(true);
        const settingsPtr = await retrieveFunds(walletPtr, hexStringToBytes(BLOCK0));

        expect(settingsPtr !== 0).toBe(true);
        const funds = await totalFunds(walletPtr);
        expect(parseInt(funds)).toBe(21000);

        const accountId = await walletId(walletPtr);

        uint8ArrayEquals(accountId, hexStringToBytes(keys.account.account_id));

        await deleteSettings(settingsPtr);
        await deleteWallet(walletPtr);
    }],
    // there is nothing we can assert here, I think
    ['should be able to set state', async function () {
        const wallet = await walletFromFile();
        const value = 1000;
        const counter = 2;
        await setState(wallet, value, counter);
    }],
    ['should fail with invalid mnemonics', async function () {
        try {
            await restoreWallet('invalidmnemonics');
            throw Error('Invalid mnemonics should fail');
        } catch (e) {
            return;
        }
    }],
    ['should fail with invalid block', async function () {
        const accountKey = hexStringToBytes(keys.account.private_key);
        const utxoKeys = Uint8Array.from([]);

        const wallet = await importKeys(accountKey, utxoKeys);
        try {
            await retrieveFunds(wallet, [0, 0, 0, 0]);
        }
        catch (e) {
            return;
        }
        throw Error('Invalid block should fail');
    }],
    ['get conversion transaction', async function () {
        const wallet = await walletFromFile();

        expect(wallet !== 0).toBe(true);

        const settings = await retrieveFunds(wallet, hexStringToBytes(BLOCK0));
        const conversion = await convertWallet(wallet, settings, await maxExpirationDate(settings, BLOCK0_DATE + 600));
        const transactions = await conversionGetTransactions(conversion);
        expect(transactions.length).toBe(1);

        const ignoredValue = await conversionGetIgnored(conversion);
        expect(ignoredValue.ignored).toBe('0');

        const pendingBefore = await getPendingTransactions(wallet);
        expect(pendingBefore.length).toBe(1);
        await walletConfirmTransaction(wallet, new Uint8Array(pendingBefore[0]))

        const pendingAfter = await getPendingTransactions(wallet);
        expect(pendingAfter.length).toBe(0);

        await deleteSettings(settings);
        await deleteConversion(conversion);
        await deleteWallet(wallet);
    }],
    ['can cast vote', async function () {
        const array = new Array(32);
        for (let index = 0; index < array.length; index++) {
            array[index] = index;
        }

        const votePlanId = new Uint8Array(array);
        const index = 0;
        const numChoices = 3;

        const proposalPtr = await proposalNewPublic(votePlanId, index, numChoices);
        const walletPtr = await walletFromFile();

        const settingsPtr = await retrieveFunds(walletPtr, hexStringToBytes(BLOCK0));
        await walletSetState(walletPtr, 1000000, 0);

        expect(await spendingCounter(walletPtr)).toBe(0);

        await walletVote(walletPtr, settingsPtr, proposalPtr, 0, await maxExpirationDate(settingsPtr, BLOCK0_DATE + 600));

        expect(await spendingCounter(walletPtr)).toBe(1);

        const pending = await getPendingTransactions(walletPtr);
        expect(pending.length).toBe(1);

        await deleteSettings(settingsPtr);
        await deleteWallet(walletPtr);
        await deleteProposal(proposalPtr);
    }],
    ['can cast private vote', async function () {
        const array = new Array(32);
        for (let index = 0; index < array.length; index++) {
            array[index] = index;
        }

        const votePlanId = new Uint8Array(array);
        const index = 0;
        const numChoices = 3;

        const proposalPtr = await proposalNewPrivate(votePlanId, index, numChoices, VOTE_ENCRYPTION_KEY);
        const walletPtr = await walletFromFile();
        const settingsPtr = await retrieveFunds(walletPtr, hexStringToBytes(BLOCK0));
        await walletSetState(walletPtr, 1000000, 1);
        await walletVote(walletPtr, settingsPtr, proposalPtr, 0, await maxExpirationDate(settingsPtr, BLOCK0_DATE + 600));

        await deleteSettings(settingsPtr);
        await deleteWallet(walletPtr);
        await deleteProposal(proposalPtr);
    }],
    ['decrypts keys correctly', async function () {
        const decryptedKeys = await symmetricCipherDecrypt(PASSWORD, hexStringToBytes(ENCRYPTED_WALLET));
        const account = decryptedKeys.slice(0 * 64, 1 * 64);
        const key1 = decryptedKeys.slice(1 * 64, 2 * 64);
        const key2 = decryptedKeys.slice(2 * 64, 3 * 64);

        if (!uint8ArrayEquals(hexStringToBytes(keys.account.private_key), new Uint8Array(account))) {
            throw Error('wrong expected account');
        }
        if (!uint8ArrayEquals(hexStringToBytes(keys.utxo_keys[0].private_key), new Uint8Array(key1))) {
            throw Error('wrong expected key1');
        }
        if (!uint8ArrayEquals(hexStringToBytes(keys.utxo_keys[1].private_key), new Uint8Array(key2))) {
            throw Error('wrong expected key2');
        }
    }],
    ['decrypt QR', async function () {
        const encrypted = '01f4c73628793ad51150f3885c706474fb35f33d85c5e3d183773f8138f5d4294807ff71fd9de1fb1a31656520eb72ebefff19f563d51d4b70c1fed789ef73ca70e9870eff4516b2a2550978ede3062e3cf8dbb40408e6f977f7f7a3b92756902b37ca172dc8b2cb09456ee891';
        const expected = 'c86596c2d1208885db1fe3658406aa0f7cc7b8e13c362fe46a6db277fc5064583e487588c98a6c36e2e7445c0add36f83f171cb5ccfd815509d19cd38ecb0af3';
        const password = new Uint8Array(4);
        password[0] = 1;
        password[1] = 2;
        password[2] = 3;
        password[3] = 4;

        const decrypted = await symmetricCipherDecrypt(password, hexStringToBytes(encrypted));
        if (!uint8ArrayEquals(hexStringToBytes(expected), new Uint8Array(decrypted))) {
            throw Error('decryption failed');
        }
    }],
    ['extract settings', async function () {
        const walletPtr = await walletFromFile();

        expect(walletPtr !== 0).toBe(true);
        const settingsPtr = await retrieveFunds(walletPtr, hexStringToBytes(BLOCK0));

        expect(settingsPtr !== 0).toBe(true);

        const settings = await settingsGet(settingsPtr);

        expect(uint8ArrayEquals(settings.block0Hash, hexStringToBytes(BLOCK0_ID))).toBe(true);
        expect(settings.discrimination).toBe(primitives.Discrimination.PRODUCTION);

        expect(settings.fees.constant).toBe("10");
        expect(settings.fees.coefficient).toBe("2");
        expect(settings.fees.certificate).toBe("100");

        expect(settings.fees.certificatePoolRegistration).toBe("0");
        expect(settings.fees.certificateStakeDelegation).toBe("0");
        expect(settings.fees.certificateOwnerStakeDelegation).toBe("0");

        expect(settings.fees.certificateVotePlan).toBe("0");
        expect(settings.fees.certificateVoteCast).toBe("0");

        await deleteSettings(settingsPtr);
        await deleteWallet(walletPtr);
    }],
    ['new settings', async function () {
        const settingsExpected = {
            block0Hash: hexStringToBytes(BLOCK0_ID),
            discrimination: primitives.Discrimination.TEST,
            fees: {
                constant: "1",
                coefficient: "2",
                certificate: "3",
                certificatePoolRegistration: "4",
                certificateStakeDelegation: "5",
                certificateOwnerStakeDelegation: "6",
                certificateVotePlan: "7",
                certificateVoteCast: "8",
            }
        };

        const block0Date = "110";
        const slotDuration = "10";
        const era = {epochStart: "0", slotStart: "0", slotsPerEpoch: "100"};
        const transactionMaxExpiryEpochs = "2";

        const settingsPtr = await settingsNew(settingsExpected.block0Hash,
            settingsExpected.discrimination, settingsExpected.fees, block0Date,
            slotDuration, era, transactionMaxExpiryEpochs
        );

        expect(settingsPtr !== 0).toBe(true);

        const settings = await settingsGet(settingsPtr);

        expect(uint8ArrayEquals(settings.block0Hash, settingsExpected.block0Hash)).toBe(true);
        expect(settings.discrimination).toBe(settingsExpected.discrimination);

        expect(settings.fees.constant).toBe(settingsExpected.fees.constant);
        expect(settings.fees.coefficient).toBe(settingsExpected.fees.coefficient);
        expect(settings.fees.certificate).toBe(settingsExpected.fees.certificate);

        expect(settings.fees.certificatePoolRegistration).toBe(settingsExpected.fees.certificatePoolRegistration);
        expect(settings.fees.certificateStakeDelegation).toBe(settingsExpected.fees.certificateStakeDelegation);
        expect(settings.fees.certificateOwnerStakeDelegation).toBe(settingsExpected.fees.certificateOwnerStakeDelegation);

        expect(settings.fees.certificateVotePlan).toBe(settingsExpected.fees.certificateVotePlan);
        expect(settings.fees.certificateVoteCast).toBe(settingsExpected.fees.certificateVoteCast);

        await deleteSettings(settingsPtr);
    }],
    ['get vote fragment id', async function () {
        const array = new Array(32);
        for (let index = 0; index < array.length; index++) {
            array[index] = index;
        }

        const votePlanId = new Uint8Array(array);
        const index = 0;
        const numChoices = 3;

        const proposalPtr = await proposalNewPublic(votePlanId, index, numChoices);
        const walletPtr = await walletFromFile();

        const settingsPtr = await retrieveFunds(walletPtr, hexStringToBytes(BLOCK0));
        await walletSetState(walletPtr, 1000000, 0);

        const tx1 = await walletVote(walletPtr, settingsPtr, proposalPtr, 1, await maxExpirationDate(settingsPtr, BLOCK0_DATE + 600));
        const tx2 = await walletVote(walletPtr, settingsPtr, proposalPtr, 2, await maxExpirationDate(settingsPtr, BLOCK0_DATE + 600));

        const id1 = await fragmentId(new Uint8Array(tx1));
        const id2 = await fragmentId(new Uint8Array(tx2));

        const pendingTransactions = await getPendingTransactions(walletPtr);

        expect(uint8ArrayEquals(id1, pendingTransactions[0])).toBe(true);
        expect(uint8ArrayEquals(id2, pendingTransactions[0])).toBe(true);

        await deleteSettings(settingsPtr);
        await deleteWallet(walletPtr);
        await deleteProposal(proposalPtr);
    }],
    ['systemtime to date', async function () {
        const walletPtr = await walletFromFile();
        const settingsPtr = await retrieveFunds(walletPtr, hexStringToBytes(BLOCK0));

        let first = await blockDateFromSystemTime(settingsPtr, BLOCK0_DATE);
        expect(first.epoch).toBe("0");
        expect(first.slot).toBe("0");

        // TODO: implement getters for this.
        let slots_per_epoch = 180;
        let slot_duration = 20;

        let second = await blockDateFromSystemTime(settingsPtr, BLOCK0_DATE + slot_duration + 1);
        expect(second.epoch).toBe("0");
        expect(second.slot).toBe("1");

        let third = await blockDateFromSystemTime(settingsPtr, BLOCK0_DATE + slot_duration * slots_per_epoch);
        expect(third.epoch).toBe("1");
        expect(third.slot).toBe("0");
    }],
]

exports.defineAutoTests = function () {
    describe('primitive mappings', function () {
        it('clobber should exist', function () {
            expect(window.wallet).toBeDefined();
        });

        // For some reason, I can't get the Jasmine version that is used in the
        // testing framework to work with async functions directly, so instead I
        // define them as async functions (which return promises), and manually
        // wrap them with the asynchronous `done` function.
        tests.forEach(([name, f]) =>
            it(name, function (done) {
                f().then(done).catch(done.fail)
            }));
    });
};

exports.defineManualTests = require('./src/manual_tests.js');

function conversionGetTransactions(conversion) {
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

function pendingTransactionsGetAll(pendingTransactions) {
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

function getPendingTransactions(walletPtr) {
    return walletPendingTransactions(walletPtr).then(
        function (pendingPtr) {
            return pendingTransactionsGetAll(pendingPtr).then(pending => {
                return deletePending(pendingPtr).then(() => pending);
            });
        });
}
