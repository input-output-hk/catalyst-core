package com.iohk.jormungandrwallet;

public class Settings {
    // Used to load the 'native-lib' library on application startup.
    static {
        System.loadLibrary("native-lib");
    }

    private long ptr;

    public Settings(long ptr) {
        ptr = ptr;
    }

    protected void finalize() {
        delete(ptr);
    }

    private native void delete(long settings);

}
