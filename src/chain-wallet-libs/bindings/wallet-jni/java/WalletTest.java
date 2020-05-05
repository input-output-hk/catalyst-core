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
        long walletPtr = Wallet.recover(
                "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone");

        byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        int totalValue = Wallet.totalValue(walletPtr);

        assertEquals(1000000 + 10000 + 10000 + 1 + 100, totalValue);

        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }

    @Test
    public void convertWallet() throws IOException {
        long walletPtr = Wallet.recover(
                "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone");

        byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        long conversionPtr = Wallet.convert(walletPtr, settingsPtr);

        int transactionsSize = Conversion.transactionsSize(conversionPtr);

        // System.out.println(Integer.toString(transactionsSize));

        byte[] transaction = Conversion.transactionsGet(conversionPtr, 0);

        // System.out.println(Integer.toString(transaction.length));
        // TODO: try to assert something
        assertEquals(true, true);

        Conversion.delete(conversionPtr);
        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }
}
