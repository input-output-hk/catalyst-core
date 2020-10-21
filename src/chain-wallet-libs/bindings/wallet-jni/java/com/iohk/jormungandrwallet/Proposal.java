package com.iohk.jormungandrwallet;

public class Proposal {
    static {
        System.loadLibrary("wallet_jni");
    }
    public static final int ID_SIZE = 32;

    public native static long withPublicPayload(byte [] votePlanId, int index, int numChoices);

    public native static long withPrivatePayload(byte [] votePlanId, int index, int numChoices, byte [] encryptionKey);

    public native static void delete(long proposalPtr);
}