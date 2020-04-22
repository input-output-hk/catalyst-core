
targets = {
    "aarch64-linux-android" : "amd64-v8a",
    "armv7-linux-androideabi" : "armeabi-v7a",
    "i686-linux-android" : "x86",
    "x86_64-linux-android" : "x86_64",
}

if __name__ == "__main__":
    import sys
    from shutil import move, rmtree
    from os import path
    import os
    libname = "libwallet_jni.so"
    for original, target in targets.items():
        if not path.exists(f"./{target}"):
            os.makedirs(f"./{target}")
        move(f"./{original}/{libname}", f"./{target}/{libname}")
        rmtree(f"./{original}")
