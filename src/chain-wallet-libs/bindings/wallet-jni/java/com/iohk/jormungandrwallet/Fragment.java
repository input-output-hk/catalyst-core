package com.iohk.jormungandrwallet;

public class Fragment {
    static {
        System.loadLibrary("wallet_jni");
    }

    public native static long fromBytes(byte[] buffer);

    public native static byte[] id(long fragmentPtr);

    public native static void delete (long fragmentPtr);
}
