// copypasted ArrayBuffer to Hex string function
const byteToHex = [];

for (let n = 0; n <= 0xff; ++n) {
    const hexOctet = ('0' + n.toString(16)).slice(-2);
    byteToHex.push(hexOctet);
}

function hex(arrayBuffer) {
    const buff = new Uint8Array(arrayBuffer);
    const hexOctets = [];

    for (let i = 0; i < buff.length; ++i) { hexOctets.push(byteToHex[buff[i]]); }

    return hexOctets.join('');
}

function hexStringToBytes(string) {
    const bytes = [];
    for (let c = 0; c < string.length; c += 2) { bytes.push(parseInt(string.substr(c, 2), 16)); }
    return Uint8Array.from(bytes);
}

/**
 * helper to convert the cordova-callback-style to promises, to make tests simpler
 * @param {function} f
 * @returns {function}
 */
function promisify(thisArg, f) {
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

            f.apply(thisArg, args);
        });
    };
    return newFunction;
}

function uint8ArrayEquals(a, b) {
    const length = a.length === b.length;
    let elements = true;

    for (let i = 0; i < a.length; i++) {
        elements = elements && a[i] === b[i];
    }

    return length && elements;
}

module.exports = {
    hex, hexStringToBytes, promisify, uint8ArrayEquals
}