package com.iohk.jormungandrwallet;

public class Wallet {
    static {
        System.loadLibrary("wallet_jni");
    }

    public native static long recover(String mnemonics);

    public native static void delete(long wallet);

    public native static int totalValue(long wallet);

    public native static long initialFunds(long wallet, byte[] block0);

    public native static long convert(long wallet, long settings);

    public native static byte[] id(long wallet);

    public native static void setState(long wallet, long value, long counter);

    public native static byte[] voteCast(long wallet, long settings, long proposal, int choice);

    public native static void confirmTransaction(long wallet, byte[] fragmentId);

    public native static long pendingTransactions(long wallet);

    public native static byte[] transferDecrypt(byte[] password, byte[] ciphertext);

    public native static long importKeys(byte[] password, byte[] ciphertext);
}