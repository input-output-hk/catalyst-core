#import <Cordova/CDVPlugin.h>

#ifndef H_WALLET_PLUGIN
#define H_WALLET_PLUGIN

@interface WalletPlugin : CDVPlugin

- (void)WALLET_IMPORT_KEYS:(CDVInvokedUrlCommand*)command;
- (void)SYMMETRIC_CIPHER_DECRYPT:(CDVInvokedUrlCommand*)command;
- (void)WALLET_SPENDING_COUNTER:(CDVInvokedUrlCommand*)command;
- (void)WALLET_TOTAL_FUNDS:(CDVInvokedUrlCommand*)command;
- (void)WALLET_ID:(CDVInvokedUrlCommand*)command;
- (void)WALLET_SET_STATE:(CDVInvokedUrlCommand*)command;
- (void)WALLET_VOTE:(CDVInvokedUrlCommand*)command;

- (void)PROPOSAL_NEW_PUBLIC:(CDVInvokedUrlCommand*)command;
- (void)PROPOSAL_NEW_PRIVATE:(CDVInvokedUrlCommand*)command;

- (void)SETTINGS_NEW:(CDVInvokedUrlCommand*)command;
- (void)SETTINGS_GET:(CDVInvokedUrlCommand*)command;

- (void)FRAGMENT_ID:(CDVInvokedUrlCommand*)command;

- (void)BLOCK_DATE_FROM_SYSTEM_TIME:(CDVInvokedUrlCommand*)command;
- (void)MAX_EXPIRATION_DATE:(CDVInvokedUrlCommand*)command;

- (void)WALLET_DELETE:(CDVInvokedUrlCommand*)command;
- (void)SETTINGS_DELETE:(CDVInvokedUrlCommand*)command;
- (void)PROPOSAL_DELETE:(CDVInvokedUrlCommand*)command;

@end

#endif /* H_WALLET_PLUGIN */
