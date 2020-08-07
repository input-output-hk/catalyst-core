import com.iohk.jormungandrwallet.Wallet;
import com.iohk.jormungandrwallet.Settings;
import com.iohk.jormungandrwallet.Conversion;
import com.iohk.jormungandrwallet.Proposal;
import com.iohk.jormungandrwallet.PendingTransactions;

import java.util.Properties;
import java.util.Enumeration;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;

import org.junit.Test;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotEquals;

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

        Conversion.ignored(conversionPtr, new Conversion.IgnoredCallback() {
            @Override
            public void call(long value, long ignored) {
                assertEquals(1, value);
                assertEquals(1, ignored);
            }
        });

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

    @Test
    public void voteCast() throws IOException {
        final long walletPtr = Wallet.recover(
                "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone");

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final byte[] id = new byte[Proposal.ID_SIZE];
        final long proposalPtr = Proposal.withPublicPayload(id, 0, 3);

        Wallet.setState(walletPtr, 10000000, 0);
        try {
            final byte[] transaction = Wallet.voteCast(walletPtr, settingsPtr, proposalPtr, 1);
        } catch (final Exception e) {
            Proposal.delete(proposalPtr);
            Settings.delete(settingsPtr);
            Wallet.delete(walletPtr);
            System.out.println(e.getMessage());
            throw e;
        }
    }

    @Test
    public void confirmConversionTransaction() throws IOException {
        final long walletPtr = Wallet.recover(
                "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone");

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final long conversionPtr = Wallet.convert(walletPtr, settingsPtr);

        final int transactionsSize = Conversion.transactionsSize(conversionPtr);

        final long before = Wallet.pendingTransactions(walletPtr);

        final int sizeBefore = PendingTransactions.len(before);

        assertEquals(sizeBefore, transactionsSize);

        final byte[] fragmentId = PendingTransactions.get(before, 0);

        PendingTransactions.delete(before);

        Wallet.confirmTransaction(walletPtr, fragmentId);

        final long after = Wallet.pendingTransactions(walletPtr);

        final int sizeAfter = PendingTransactions.len(after);

        assertEquals(sizeAfter, 0);

        PendingTransactions.delete(after);
        Conversion.delete(conversionPtr);
        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }

    @Test
    public void confirmVoteCast() throws IOException {
        final long walletPtr = Wallet.recover(
                "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone");

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final byte[] id = new byte[Proposal.ID_SIZE];
        final long proposalPtr = Proposal.withPublicPayload(id, 0, 3);

        Wallet.setState(walletPtr, 10000000, 0);
        try {
            final byte[] transaction = Wallet.voteCast(walletPtr, settingsPtr, proposalPtr, 1);

            final long before = Wallet.pendingTransactions(walletPtr);

            final int sizeBefore = PendingTransactions.len(before);

            assertEquals(sizeBefore, 1);

            final byte[] fragmentId = PendingTransactions.get(before, 0);

            PendingTransactions.delete(before);

            Wallet.confirmTransaction(walletPtr, fragmentId);

            final long after = Wallet.pendingTransactions(walletPtr);

            final int sizeAfter = PendingTransactions.len(after);

            assertEquals(sizeAfter, 0);

            PendingTransactions.delete(after);
        } catch (final Exception e) {
            Proposal.delete(proposalPtr);
            Settings.delete(settingsPtr);
            Wallet.delete(walletPtr);
            System.out.println(e.getMessage());
            throw e;
        }
    }

    @Test
    public void testDecrypt() throws IOException {
        final String hex = "0142a4ff1c711c3ce066bb216923abd961c76fc912d5983319ce8a9f4f7e6bdf0a530f63eb3dfc3d57257222450387c62919c55c2bcdd3e3f58e81e846a6cb7f988a88c71327c46282632e915363b1757d733417b261e1381b2e8f2fafe241f23dcbf37b03cd589d2362109c227718b12e46d51cf79b409876012b96518a7b0920916f08b159e7fe699234d417ed4c9b9d144f91652616409033f0a30ad46df8168308e7345744b89d44c32ae81444c0ad3ce5b47df79999404ed67f9a40eb3204e48c37b3c7c7e4adf798ebee15566b10b4a52f1488db12f111036d9f7fc2bee8441c0155d0071d839763d4f8";
        final byte[] bytes = hexStringToByteArray(hex);

        final byte[] password = { 1, 2, 3, 4 };

        final byte[] plaintext = Wallet.transferDecrypt(password, bytes);

        final byte[] expected = new byte[64 * 3];

        for (int j = 0; j < 3; j++) {
            for (int i = j * 64; i < (j + 1) * 64; i++) {
                expected[i] = (byte) (j + 1);
            }
        }

        for (int i = 0; i < expected.length; i++) {
           assertEquals(plaintext[i], expected[i]);
        }

        assertEquals(plaintext.length, expected.length);
    }

    public static byte[] hexStringToByteArray(String s) {
        int len = s.length();
        byte[] data = new byte[len / 2];
        for (int i = 0; i < len; i += 2) {
            data[i / 2] = (byte) ((Character.digit(s.charAt(i), 16) << 4) + Character.digit(s.charAt(i + 1), 16));
        }
        return data;
    }
}
