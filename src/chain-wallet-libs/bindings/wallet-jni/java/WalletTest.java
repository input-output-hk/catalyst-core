import com.iohk.jormungandrwallet.Wallet;
import com.iohk.jormungandrwallet.Settings;
import com.iohk.jormungandrwallet.Conversion;
import com.iohk.jormungandrwallet.Fragment;
import com.iohk.jormungandrwallet.Proposal;
import com.iohk.jormungandrwallet.PendingTransactions;
import com.iohk.jormungandrwallet.SymmetricCipher;
import com.iohk.jormungandrwallet.Fragment;
import com.iohk.jormungandrwallet.Time;
import com.iohk.jormungandrwallet.Time.BlockDate;

import java.util.Properties;
import java.util.Enumeration;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;

import org.junit.Test;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertNotEquals;

public class WalletTest {
    private byte[] accountKey() throws IOException {
        final String accountKey = new String(Files.readAllBytes(Paths.get("../../../test-vectors/free_keys/key1.prv")))
                .trim();

        return hexStringToByteArray(accountKey);
    }

    private byte[] utxoKeys() throws IOException {
        final String utxoKey1 = new String(Files.readAllBytes(Paths.get("../../../test-vectors/free_keys/key2.prv")))
                .trim();

        final String utxoKey2 = new String(Files.readAllBytes(Paths.get("../../../test-vectors/free_keys/key3.prv")))
                .trim();

        return hexStringToByteArray(utxoKey1.concat(utxoKey2));
    }

    @Test
    public void extractSettings() throws IOException {
        final long walletPtr = Wallet.importKeys(accountKey(), utxoKeys());

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final Settings.LinearFees fees = Settings.fees(settingsPtr);

        assertEquals(fees.constant, 10);
        assertEquals(fees.coefficient, 2);
        assertEquals(fees.certificate, 100);

        assertEquals(fees.perCertificateFee.certificatePoolRegistration, 0);
        assertEquals(fees.perCertificateFee.certificateStakeDelegation, 0);
        assertEquals(fees.perCertificateFee.certificateOwnerStakeDelegation, 0);

        assertEquals(fees.perVoteCertificateFee.certificateVotePlan, 0);
        assertEquals(fees.perVoteCertificateFee.certificateVoteCast, 0);

        final Settings.Discrimination discrimination = Settings.discrimination(settingsPtr);

        // TODO: actually, why are we using PRODUCTION discrimination in the
        // test vectors genesis?
        assertEquals(Settings.Discrimination.PRODUCTION, discrimination);

        assertArrayEquals(hexStringToByteArray("8f7eb264426d2a81d5df7433e4713a38397deda81813115884b68c853f549dae"),
                Settings.block0Hash(settingsPtr));

        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }

    @Test
    public void buildSettings() throws IOException {
        final byte[] blockId = hexStringToByteArray("182764b45bae25cc466143de8107618b37f0d28fe3daa0a0d39fd0ab5a2061e1");
        final Settings.Discrimination discrimination = Settings.Discrimination.TEST;
        final Settings.LinearFees expectedFees = new Settings.LinearFees(1, 2, 3,
                new Settings.PerCertificateFee(4, 5, 6), new Settings.PerVoteCertificateFee(7, 8));

        final Settings.TimeEra timeEra = new Settings.TimeEra(0, 0, 100);

        Settings.PerCertificateFee test = new Settings.PerCertificateFee(4, 5, 6);

        final long settingsPtr = Settings.build(expectedFees, discrimination, blockId, 10, (short) 15, timeEra, (short) 2);

        final Settings.LinearFees fees = Settings.fees(settingsPtr);

        assertEquals(fees.constant, expectedFees.constant);
        assertEquals(fees.coefficient, expectedFees.coefficient);
        assertEquals(fees.certificate, expectedFees.certificate);

        assertEquals(fees.perCertificateFee.certificatePoolRegistration,
                expectedFees.perCertificateFee.certificatePoolRegistration);

        assertEquals(fees.perCertificateFee.certificateStakeDelegation,
                expectedFees.perCertificateFee.certificateStakeDelegation);

        assertEquals(fees.perCertificateFee.certificateOwnerStakeDelegation,
                expectedFees.perCertificateFee.certificateOwnerStakeDelegation);

        assertEquals(fees.perVoteCertificateFee.certificateVotePlan,
                expectedFees.perVoteCertificateFee.certificateVotePlan);
        assertEquals(fees.perVoteCertificateFee.certificateVoteCast,
                expectedFees.perVoteCertificateFee.certificateVoteCast);

        final Settings.Discrimination foundDiscrimination = Settings.discrimination(settingsPtr);

        assertEquals(discrimination, foundDiscrimination);

        assertArrayEquals(blockId, Settings.block0Hash(settingsPtr));

        Settings.delete(settingsPtr);
    }

    @Test
    public void importKeys() throws IOException {
        final long walletPtr = Wallet.importKeys(accountKey(), utxoKeys());

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final int totalValue = Wallet.totalValue(walletPtr);

        assertEquals(10000 + 10000 + 1000, totalValue);

        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }

    @Test
    public void importOnlyAccountKey() throws IOException {
        final byte[] emptyArray = {};
        final long walletPtr = Wallet.importKeys(accountKey(), emptyArray);

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final int totalValue = Wallet.totalValue(walletPtr);

        assertEquals(10000, totalValue);

        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }

    @Test
    public void convertWallet() throws IOException {
        final long walletPtr = Wallet.importKeys(accountKey(), utxoKeys());

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final long conversionPtr = Wallet.convert(walletPtr, settingsPtr, Time.ttlFromDate(settingsPtr, 0));

        final int transactionsSize = Conversion.transactionsSize(conversionPtr);

        assertEquals(1, transactionsSize);

        final byte[] transaction = Conversion.transactionsGet(conversionPtr, 0);

        Conversion.delete(conversionPtr);
        Settings.delete(settingsPtr);
        Wallet.delete(walletPtr);
    }

    @Test(expected = IndexOutOfBoundsException.class)
    public void negativeIndexConversionTransaction() throws IOException {
        final long walletPtr = Wallet.importKeys(accountKey(), utxoKeys());

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final long conversionPtr = Wallet.convert(walletPtr, settingsPtr, Time.ttlFromDate(settingsPtr, 0));

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
        final byte[] accountKey = { -56, 101, -106, -62, -47, 32, -120, -123, -37, 31, -29, 101, -124, 6, -86, 15, 124,
                -57, -72, -31, 60, 54, 47, -28, 106, 109, -78, 119, -4, 80, 100, 88, 62, 72, 117, -120, -55, -118, 108,
                54, -30, -25, 68, 92, 10, -35, 54, -8, 63, 23, 28, -75, -52, -3, -127, 85, 9, -47, -100, -45, -114, -53,
                10, -13, };

        final byte[] utxoKeys = { 48, 21, 89, -52, -78, -44, -52, 126, -98, 84, -90, -11, 90, -128, -106, 11, -74, -111,
                -73, -79, 64, -107, 73, -17, -122, -107, -87, 46, -92, 26, 111, 79, 64, 82, 49, -88, 6, -62, -25, -71,
                -48, -37, 48, -31, 94, -32, -52, 31, 38, 28, 27, -97, -106, 21, 99, 107, 72, -67, -119, -2, 123, -26,
                -22, 31, -88, -74, -67, -16, -128, -57, 79, -68, 49, 51, 126, -34, 75, 102, -110, -62, -21, -19, 126,
                52, -81, 109, -104, -73, -69, -51, 71, -116, -16, 123, 13, 94, -39, 63, 126, -99, 74, -93, -81, -34, 50,
                26, -31, -85, -74, 27, -125, 68, -62, 67, -55, -48, -76, 7, -53, -8, -111, 125, -74, -33, 44, 101, 61,
                -22, };

        final long walletPtr = Wallet.importKeys(accountKey, utxoKeys);

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final byte[] id = new byte[Proposal.ID_SIZE];
        final long proposalPtr = Proposal.withPublicPayload(id, 0, 3);

        Wallet.setState(walletPtr, 10000000, 0);

        assertEquals(Wallet.spendingCounter(walletPtr), 0);

        try {
            final byte[] transaction = Wallet.voteCast(walletPtr, settingsPtr, proposalPtr, 1, Time.ttlFromDate(settingsPtr, 0));

            assertEquals(Wallet.spendingCounter(walletPtr), 1);
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
        final byte[] accountKey = { -56, 101, -106, -62, -47, 32, -120, -123, -37, 31, -29, 101, -124, 6, -86, 15, 124,
                -57, -72, -31, 60, 54, 47, -28, 106, 109, -78, 119, -4, 80, 100, 88, 62, 72, 117, -120, -55, -118, 108,
                54, -30, -25, 68, 92, 10, -35, 54, -8, 63, 23, 28, -75, -52, -3, -127, 85, 9, -47, -100, -45, -114, -53,
                10, -13, };

        final byte[] utxoKeys = { 48, 21, 89, -52, -78, -44, -52, 126, -98, 84, -90, -11, 90, -128, -106, 11, -74, -111,
                -73, -79, 64, -107, 73, -17, -122, -107, -87, 46, -92, 26, 111, 79, 64, 82, 49, -88, 6, -62, -25, -71,
                -48, -37, 48, -31, 94, -32, -52, 31, 38, 28, 27, -97, -106, 21, 99, 107, 72, -67, -119, -2, 123, -26,
                -22, 31, -88, -74, -67, -16, -128, -57, 79, -68, 49, 51, 126, -34, 75, 102, -110, -62, -21, -19, 126,
                52, -81, 109, -104, -73, -69, -51, 71, -116, -16, 123, 13, 94, -39, 63, 126, -99, 74, -93, -81, -34, 50,
                26, -31, -85, -74, 27, -125, 68, -62, 67, -55, -48, -76, 7, -53, -8, -111, 125, -74, -33, 44, 101, 61,
                -22, };

        final long walletPtr = Wallet.importKeys(accountKey, utxoKeys);

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final long conversionPtr = Wallet.convert(walletPtr, settingsPtr, Time.ttlFromDate(settingsPtr, 0));

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
        final long walletPtr = Wallet.importKeys(accountKey(), utxoKeys());

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));
        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final byte[] id = new byte[Proposal.ID_SIZE];
        final long proposalPtr = Proposal.withPublicPayload(id, 0, 3);

        Wallet.setState(walletPtr, 10000000, 0);

        try {
            final byte[] transaction = Wallet.voteCast(walletPtr, settingsPtr, proposalPtr, 1, Time.ttlFromDate(settingsPtr, 0));

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
    public void fragmentId() throws IOException {
        final long walletPtr = Wallet.importKeys(accountKey(), utxoKeys());

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final byte[] id = new byte[Proposal.ID_SIZE];
        final long proposalPtr = Proposal.withPublicPayload(id, 0, 3);

        Wallet.setState(walletPtr, 10000000, 0);

        try {
            final byte[] transaction = Wallet.voteCast(walletPtr, settingsPtr, proposalPtr, 1, Time.ttlFromDate(settingsPtr, 0));

            final long fragment = Fragment.fromBytes(transaction);
            final byte[] fragmentId = Fragment.id(fragment);
            Fragment.delete(fragment);

            final long pending = Wallet.pendingTransactions(walletPtr);

            final int sizeBefore = PendingTransactions.len(pending);

            final byte[] expectedFragmentId = PendingTransactions.get(pending, 0);

            for (int i = 0; i < fragmentId.length; i++) {
                assertEquals(fragmentId[i], expectedFragmentId[i]);
            }

            PendingTransactions.delete(pending);
        } catch (final Exception e) {
            Proposal.delete(proposalPtr);
            Settings.delete(settingsPtr);
            Wallet.delete(walletPtr);
            System.out.println(e.getMessage());
            throw e;
        }
    }

    @Test
    public void privateVoteCast() throws IOException {
        final long walletPtr = Wallet.importKeys(accountKey(), utxoKeys());

        final byte[] block0 = Files.readAllBytes(Paths.get("../../../test-vectors/block0"));

        final String encryptingKey = "votepk1nc988wtjlrm5k0z43088p0rrvd5yhvc96k7zh99p6w74gupxggtqwym0vm";

        final long settingsPtr = Wallet.initialFunds(walletPtr, block0);

        final byte[] id = new byte[Proposal.ID_SIZE];
        final long proposalPtr = Proposal.withPrivatePayload(id, 0, 3, encryptingKey);

        Wallet.setState(walletPtr, 10000000, 0);
        try {
            final byte[] transaction = Wallet.voteCast(walletPtr, settingsPtr, proposalPtr, 1, Time.ttlFromDate(settingsPtr, 0));

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
        final String hex = "017b938f189c7d1d9e4c75b02710a9c9a6b287b6ca55d624001828cba8aeb3a9d4c2a86261016693c7e05fb281f012fb2d7af44484da09c4d7b2dea6585965a4cc208d2b2fb1aa5ba6338520b3aa9c4f908fdd62816ebe01f496f8b4fc0344892fe245db072d054c3dedff926320589231298e216506c1f6858c5dba915c959a98ba0d0e3995aef91d4216b5172dedf2736b451d452916b81532eb7f8487e9f88a2de4f9261d0a0ddf11698796ad8b6894908024ebc4be9bba985ef9c0f2f71afce0b37520c66938313f6bf81b3fc24f5c93d216cd2528dabc716b8093359fda84db4e58d876d215713f2db000";

        final byte[] encrypted = hexStringToByteArray(hex);

        final byte[] password = { 1, 2, 3, 4 };

        final byte[] decrypted = SymmetricCipher.decrypt(password, encrypted);

        final byte[] account = { -56, 101, -106, -62, -47, 32, -120, -123, -37, 31, -29, 101, -124, 6, -86, 15, 124,
                -57, -72, -31, 60, 54, 47, -28, 106, 109, -78, 119, -4, 80, 100, 88, 62, 72, 117, -120, -55, -118, 108,
                54, -30, -25, 68, 92, 10, -35, 54, -8, 63, 23, 28, -75, -52, -3, -127, 85, 9, -47, -100, -45, -114, -53,
                10, -13, };

        final byte[] key1 = { 48, 21, 89, -52, -78, -44, -52, 126, -98, 84, -90, -11, 90, -128, -106, 11, -74, -111,
                -73, -79, 64, -107, 73, -17, -122, -107, -87, 46, -92, 26, 111, 79, 64, 82, 49, -88, 6, -62, -25, -71,
                -48, -37, 48, -31, 94, -32, -52, 31, 38, 28, 27, -97, -106, 21, 99, 107, 72, -67, -119, -2, 123, -26,
                -22, 31, };

        final byte[] key2 = { -88, -74, -67, -16, -128, -57, 79, -68, 49, 51, 126, -34, 75, 102, -110, -62, -21, -19,
                126, 52, -81, 109, -104, -73, -69, -51, 71, -116, -16, 123, 13, 94, -39, 63, 126, -99, 74, -93, -81,
                -34, 50, 26, -31, -85, -74, 27, -125, 68, -62, 67, -55, -48, -76, 7, -53, -8, -111, 125, -74, -33, 44,
                101, 61, -22, };

        for (int i = 0; i < 64; i++) {
            assertEquals(decrypted[0 * 64 + i], account[i]);
        }

        for (int i = 0; i < 64; i++) {
            assertEquals(decrypted[1 * 64 + i], key1[i]);
        }

        for (int i = 0; i < 64; i++) {
            assertEquals(decrypted[2 * 64 + i], key2[i]);
        }
    }

    @Test(expected = Exception.class)
    public void testDecryptWrongPassword() throws IOException {
        final String hex = "017b938f189c7d1d9e4c75b02710a9c9a6b287b6ca55d624001828cba8aeb3a9d4c2a86261016693c7e05fb281f012fb2d7af44484da09c4d7b2dea6585965a4cc208d2b2fb1aa5ba6338520b3aa9c4f908fdd62816ebe01f496f8b4fc0344892fe245db072d054c3dedff926320589231298e216506c1f6858c5dba915c959a98ba0d0e3995aef91d4216b5172dedf2736b451d452916b81532eb7f8487e9f88a2de4f9261d0a0ddf11698796ad8b6894908024ebc4be9bba985ef9c0f2f71afce0b37520c66938313f6bf81b3fc24f5c93d216cd2528dabc716b8093359fda84db4e58d876d215713f2db000";

        final byte[] encrypted = hexStringToByteArray(hex);
        final byte[] password = { 127, 127, 127, 127 };
        SymmetricCipher.decrypt(password, encrypted);
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
