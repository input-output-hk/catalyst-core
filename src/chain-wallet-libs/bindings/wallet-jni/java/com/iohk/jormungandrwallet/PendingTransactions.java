package com.iohk.jormungandrwallet;

public class PendingTransactions {
    static {
        System.loadLibrary("wallet_jni");
    }

    public native static int len(long self);

    public native static byte[] get(long self, int index);

    public native static void delete(long self);
}