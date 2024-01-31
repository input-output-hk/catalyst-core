import fs from 'fs';
import path from 'path';

// Construct the path to the .txt file
const txtPath = '../../wallet-automation/typhon/typhon-wallet-storage.txt';

// Read the contents of the .txt file
const txtContent = fs.readFileSync(path.resolve(txtPath), 'utf8');

// Parse the contents and set them to process.env
txtContent.split('\n').forEach(line => {
  const [key, value] = line.split('=');
  if (key && value) {
    process.env[key.trim()] = value.trim();
  }
});

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
