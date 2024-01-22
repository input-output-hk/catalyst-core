import dotenv from 'dotenv';
import path from 'path';

// Construct the path to the .env file
const envPath = path.join(__dirname, '..', 'catalyst-core', 'src', 'typhon-wallet-storage.env');
dotenv.config({ path: envPath });

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
