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

interface WalletCredentials {
  username: string;
  password: string; // Added password field
}

// Function to get wallet credentials
const getWalletCredentials = (walletID: string): WalletCredentials => {
  const username = process.env[`${walletID}_USERNAME`];
  const password = process.env[`${walletID}_PASSWORD`]; // Retrieve password from env
  console.log(`Username: ${username}, Password: ${password}`); // Debugging line

  if (!username || !password) {
    throw new Error(`Credentials for ${walletID} not found`);
  }

  return { username, password };  // Include password in the return value
};

export { getWalletCredentials };
