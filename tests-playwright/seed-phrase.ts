import dotenv from 'dotenv';
import path from 'path';

// Construct the path to the .env file
const envPath = path.join(__dirname, '..', 'catalyst-core', 'src', 'typhon-wallet-storage.env');
dotenv.config({ path: envPath });

interface SeedPhrase {

  seedPhrase: string[];
}

// Function to get the seed phrase from environment variables
const getSeedPhrase = (): string[] => {
  const seedPhraseArray: string[] = [];
  for (let i = 1; i <= 15; i++) {
    const word = process.env[`WALLET1_SEED_WORD_${i}`];
    if (!word) {
      throw new Error(`Seed word ${i} is missing`);
    }
    seedPhraseArray.push(word);
  }
  return seedPhraseArray;
};

export { getSeedPhrase };
