import dotenv from 'dotenv';
import path from 'path';

// Construct the path to the .env file
const envPath = path.join(__dirname, '..', 'catalyst-core', 'src', 'typhon-wallet-storage.env');
dotenv.config({ path: envPath });

interface WalletCredentials {
  username: string;
  password: string; // Added password field
  seedPhrase: string[];
}

// Function to get the seed phrase from environment variables
const getSeedPhrase = (): string[] => {
  const seedPhraseArray: string[] = [];
  for (let i = 1; i <= 15; i++) {
    const word = process.env[`SEED_WORD_${i}`];
    if (!word) {
      throw new Error(`Seed word ${i} is missing`);
    }
    seedPhraseArray.push(word);
  }
  return seedPhraseArray;
};

// Function to get wallet credentials
const getWalletCredentials = (walletId: string): WalletCredentials => {
  const username = process.env[`${walletId}_USERNAME`];
  const password = process.env[`${walletId}_PASSWORD`]; // Retrieve password from env
  const seedPhrase = getSeedPhrase();
  
  if (!username || !password || seedPhrase.length === 0) {
    throw new Error(`Credentials for ${walletId} not found`);
  }
  
  return { username, password, seedPhrase }; // Include password in the return value
};

export { getWalletCredentials };
