package com.iohk.jormungandrwallet

class Settings {
    private external fun delete(settings: Long)
    companion object {
        init {
            System.loadLibrary("wallet_jni")
        }
    }

    private var settings_prt : Long = 0

    constructor(ptr : Long) {
        this.settings_prt = ptr
    }

    protected fun finalize() {
        delete(this.settings_prt)
    }
}