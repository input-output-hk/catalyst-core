package com.iohk.jormungandrwallet;

public class Wallet {

    // Used to load the 'native-lib' library on application startup.
    static {
        System.loadLibrary("native-lib");
    }

    private long ptr;

    public Wallet(String mnemonics) {
        ptr = recover(mnemonics);
    }

    public Settings initialFunds(byte[] block0) {
        long settings = initialFunds(ptr, block0);

        return new Settings(settings);
    }

    public int value() {
        return totalValue(ptr);
    }

    protected void finalize() {
        delete(ptr);
    }

    private native long recover(String mnemonics);
    private native void delete(long wallet);
    private native int totalValue(long wallet);
    private native long initialFunds(long wallet, byte[] block0);
}
