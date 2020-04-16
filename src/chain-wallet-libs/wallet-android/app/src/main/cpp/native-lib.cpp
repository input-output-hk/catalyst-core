#include <jni.h>
#include <string>

typedef void const *WalletPtr;
typedef void const *SettingsPtr;
typedef uint8_t RecoveringResult;

extern "C" {
    RecoveringResult iohk_jormungandr_wallet_recover(const char *mnemonics,
                                                     const uint8_t *password,
                                                     uintptr_t password_length,
                                                     WalletPtr *wallet_out);
    RecoveringResult iohk_jormungandr_wallet_retrieve_funds(WalletPtr wallet,
                                                            const uint8_t *block0,
                                                            uintptr_t block0_length,
                                                            SettingsPtr *settings_out);
    RecoveringResult iohk_jormungandr_wallet_total_value(WalletPtr wallet,
                                                         uint64_t *total_out);
    void iohk_jormungandr_wallet_delete_settings(SettingsPtr settings);
    void iohk_jormungandr_wallet_delete_wallet(WalletPtr wallet);
}

extern "C"
JNIEXPORT jlong JNICALL
Java_com_iohk_jormungandrwallet_Wallet_recover(JNIEnv *env, jobject thiz, jstring mnemonics) {
    const char* mnemonics_c = env->GetStringUTFChars(mnemonics, 0);

    WalletPtr wallet = NULL;

    RecoveringResult r = iohk_jormungandr_wallet_recover(mnemonics_c, NULL, 0, &wallet);

    env->ReleaseStringUTFChars(mnemonics, mnemonics_c);

    if (r == 0) {
        return (jlong)wallet;
    } else {
        return 0;
    }
}

extern "C"
JNIEXPORT void JNICALL
Java_com_iohk_jormungandrwallet_Wallet_delete(JNIEnv *env, jobject thiz, jlong wallet) {
    WalletPtr ptr = (WalletPtr)wallet;
    if (ptr != NULL) {
        iohk_jormungandr_wallet_delete_wallet(ptr);
    }
}

extern "C"
JNIEXPORT void JNICALL
Java_com_iohk_jormungandrwallet_Settings_delete(JNIEnv *env, jobject thiz, jlong settings) {
    SettingsPtr ptr = (SettingsPtr)settings;

    if (ptr != NULL) {
        iohk_jormungandr_wallet_delete_settings(ptr);
    }
}

extern "C"
JNIEXPORT jint JNICALL
Java_com_iohk_jormungandrwallet_Wallet_totalValue(JNIEnv *env, jobject thiz, jlong wallet) {
    WalletPtr ptr = (WalletPtr)wallet;
    uint64_t value = 0;

    if (ptr != NULL) {
        iohk_jormungandr_wallet_total_value(ptr, &value);
    }

    return value;
}

extern "C"
JNIEXPORT jlong JNICALL
Java_com_iohk_jormungandrwallet_Wallet_initialFunds(JNIEnv *env, jobject thiz, jlong wallet,
                                                    jbyteArray block0) {
    WalletPtr ptr = (WalletPtr)wallet;
    SettingsPtr settings_ptr = NULL;
    uintptr_t len = env->GetArrayLength(block0);
    uint8_t* bytes = new uint8_t[len];
    env->GetByteArrayRegion(block0, 0, len, reinterpret_cast<jbyte*>(bytes));

    if (ptr != NULL) {
        iohk_jormungandr_wallet_retrieve_funds(ptr, bytes, len, &settings_ptr);
    }

    delete[] bytes;

    return (jlong)settings_ptr;
}