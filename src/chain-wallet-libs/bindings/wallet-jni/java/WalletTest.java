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
}
