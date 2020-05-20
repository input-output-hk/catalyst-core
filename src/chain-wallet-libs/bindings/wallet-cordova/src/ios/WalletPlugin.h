#import <Cordova/CDVPlugin.h>

#ifndef H_WALLET_PLUGIN
#define H_WALLET_PLUGIN

@interface WalletPlugin : CDVPlugin

- (void)WALLET_RESTORE:(CDVInvokedUrlCommand*)command;
- (void)WALLET_RETRIEVE_FUNDS:(CDVInvokedUrlCommand*)command;
- (void)WALLET_TOTAL_FUNDS:(CDVInvokedUrlCommand*)command;
- (void)WALLET_ID:(CDVInvokedUrlCommand*)command;
- (void)WALLET_CONVERT:(CDVInvokedUrlCommand*)command;

- (void)CONVERSION_TRANSACTIONS_SIZE:(CDVInvokedUrlCommand*)command;
- (void)CONVERSION_TRANSACTIONS_GET:(CDVInvokedUrlCommand*)command;

- (void)WALLET_DELETE:(CDVInvokedUrlCommand*)command;
- (void)SETTINGS_DELETE:(CDVInvokedUrlCommand*)command;
- (void)CONVERSION_DELETE:(CDVInvokedUrlCommand*)command;

@end

#endif /* H_WALLET_PLUGIN */
