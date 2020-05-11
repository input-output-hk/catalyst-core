package com.iohk.jormungandrwallet;

public class Settings {
    static {
        System.loadLibrary("wallet_jni");
    }

    public native static void delete(long settings);
}