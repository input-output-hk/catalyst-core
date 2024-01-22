import dotenv from 'dotenv';

// Construct the path to the .env file
const envPath = '/Users/alicechaiyakul/typhon-wallet-registration/catalyst-core/src/typhon-wallet-storage.env';
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
