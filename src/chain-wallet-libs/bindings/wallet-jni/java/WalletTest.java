import com.iohk.jormungandrwallet.Wallet;
import com.iohk.jormungandrwallet.Settings;
import com.iohk.jormungandrwallet.Conversion;

import java.util.Properties;
import java.util.Enumeration;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;

import org.junit.Test;
import static org.junit.Assert.assertEquals;

public class WalletTest {
    @Test
    public void recoverWallet() throws IOException {
        final long walletPtr = Wallet.recover(
                "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone");

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final int totalValue = Wallet.totalValue(walletPtr);

        assertEquals(1000000 + 10000 + 10000 + 1 + 100, totalValue);

        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }

    @Test
    public void convertWallet() throws IOException {
        final long walletPtr = Wallet.recover(
                "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone");

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final long conversionPtr = Wallet.convert(walletPtr, settingsPtr);

        final int transactionsSize = Conversion.transactionsSize(conversionPtr);

        assertEquals(1, transactionsSize);

        final byte[] transaction = Conversion.transactionsGet(conversionPtr, 0);

        assertEquals(723, transaction.length);

        Conversion.delete(conversionPtr);
        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }

    @Test(expected = IndexOutOfBoundsException.class)
    public void negativeIndexConversionTransaction() throws IOException {
        final long walletPtr = Wallet.recover(
                "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone");

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final long conversionPtr = Wallet.convert(walletPtr, settingsPtr);

        final int transactionsSize = Conversion.transactionsSize(conversionPtr);

        try {
            final byte[] transaction = Conversion.transactionsGet(conversionPtr, -1);
        } catch (final Exception e) {
            Conversion.delete(conversionPtr);
            Settings.delete(settingsPtr);
            Wallet.delete(walletPtr);
            throw e;
        }
    }
}
