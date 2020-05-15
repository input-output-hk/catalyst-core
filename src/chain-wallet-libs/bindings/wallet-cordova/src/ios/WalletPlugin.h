#import <Cordova/CDVPlugin.h>

@interface WalletPlugin : CDVPlugin

- (void)WALLET_RESTORE:(CDVInvokedUrlCommand*)command;
- (void)WALLET_RETRIEVE_FUNDS:(CDVInvokedUrlCommand*)command;
- (void)WALLET_TOTAL_FUNDS:(CDVInvokedUrlCommand*)command;
- (void)WALLET_ID:(CDVInvokedUrlCommand*)command;
- (void)SETTINGS_DELETE:(CDVInvokedUrlCommand*)command;
- (void)WALLET_DELETE:(CDVInvokedUrlCommand*)command;

@end
