# Testing

The tests use [cordova-plugin-test-framework](https://github.com/apache/cordova-plugin-test-framework).

# Android

## TLDR

In the tests directory, run (it should be possible to automatize this later).

`npx run build`

In a new directory for a test app.

```sh
cd $TEST_APP_DIRECTORY 
cordova create hello com.example.hello HelloWorld
cd hello
cordova platform add android 
cordova plugin add cordova-plugin-test-framework
cordova plugin add this-plugin-path
cordova plugin add path-to-wallet-cordova/tests
sed 's/<content src="index.html" \/>/<content src="cdvtests\/index.html" \/>/' config.xml -i
cordova build
cordova run android
```

Where `this-plugin-path` could be the packaged version, or the path to the root directory.

# Electron

Currently the tests don't work for some reason