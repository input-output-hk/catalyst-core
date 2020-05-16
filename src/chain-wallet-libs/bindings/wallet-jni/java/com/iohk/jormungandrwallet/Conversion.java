package com.iohk.jormungandrwallet;

public class Conversion {
    static {
        System.loadLibrary("wallet_jni");
    }

    public native static void delete(long conversion);

    public native static int transactionsSize(long conversion);

    public native static byte[] transactionsGet(long conversion, int index);

    public native static void ignored(long conversion, IgnoredCallback callback);

    public interface IgnoredCallback {
        void call(long value, long ignored);
    }
}