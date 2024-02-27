import fs from 'fs';
import path from 'path';

const txtContent = fs.readFileSync(path.resolve(__dirname,'adalite-wallet-storage.txt'), 'utf8');;

// parse the contents and set them to process.env
txtContent.split('\n').forEach(line => {
  const [key, value] = line.split('=');
  if (key && value) {
    process.env[key.trim()] = value.trim();
  }
});

interface AdaliteSeedPhrase {
  seedPhrase: string;
}
const getAdaliteSeedPhrase = (walletID: string): AdaliteSeedPhrase => {
  const seedPhrase = process.env[`${walletID}_SEED_PHRASE`];
  console.log(`seedPhrase: , ${seedPhrase}`);

  if (!seedPhrase ) {
    throw new Error(`Credentials for ${walletID} not found`);
  }

  return { seedPhrase };
};
export { getAdaliteSeedPhrase };