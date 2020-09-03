#import "WalletPlugin.h"

#import <Foundation/Foundation.h>

#include <memory.h>
#include <stdint.h>

#include "LibWallet.h"

CDVPluginResult*
jormungandr_error_to_plugin_result(ErrorPtr error)
{
    char* error_desc_raw = iohk_jormungandr_wallet_error_to_string(error);
    NSString* error_desc = [NSString stringWithCString:error_desc_raw
                                              encoding:NSUTF8StringEncoding];

    CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                                      messageAsString:error_desc];

    iohk_jormungandr_wallet_delete_string(error_desc_raw);
    iohk_jormungandr_wallet_delete_error(error);

    return pluginResult;
}

@implementation WalletPlugin

- (void)WALLET_RESTORE:(CDVInvokedUrlCommand*)command
{
    NSString* mnemonics = [command.arguments objectAtIndex:0];

    [self.commandDelegate runInBackground:^{
        CDVPluginResult* pluginResult = nil;

        WalletPtr wallet_ptr;
        ErrorPtr result =
            iohk_jormungandr_wallet_recover([mnemonics UTF8String], nil, 0, &wallet_ptr);

        if (result != nil) {
            pluginResult = jormungandr_error_to_plugin_result(result);
        } else {
            NSString* returnValue = [NSString stringWithFormat:@"%ld", (uintptr_t)wallet_ptr];
            pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                             messageAsString:returnValue];
        }

        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
    }];
}

- (void)WALLET_IMPORT_KEYS:(CDVInvokedUrlCommand*)command
{
    NSData* account_key = [command.arguments objectAtIndex:0];
    NSData* utxo_keys = [command.arguments objectAtIndex:1];

    if ([account_key isEqual:[NSNull null]]) {
        CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                                          messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    if ([utxo_keys isEqual:[NSNull null]]) {
        CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                                          messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    if (utxo_keys.length % 64 != 0) {
        CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                                          messageAsString:@"invalid argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    [self.commandDelegate runInBackground:^{
        CDVPluginResult* pluginResult = nil;
        WalletPtr wallet_ptr = nil;

        ErrorPtr result = iohk_jormungandr_wallet_import_keys(account_key.bytes,
            utxo_keys.bytes,
            utxo_keys.length / 64,
            &wallet_ptr);

        if (result != nil) {
            pluginResult = jormungandr_error_to_plugin_result(result);
        } else {
            NSString* returnValue = [NSString stringWithFormat:@"%ld", (uintptr_t)wallet_ptr];
            pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                             messageAsString:returnValue];
        }

        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
    }];
}

- (void)SYMMETRIC_CIPHER_DECRYPT:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSData* password = [command.arguments objectAtIndex:0];
    NSData* ciphertext = [command.arguments objectAtIndex:1];

    if ([password isEqual:[NSNull null]]) {
        CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                                          messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    if ([ciphertext isEqual:[NSNull null]]) {
        CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                                          messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    uint8_t* plaintext = nil;
    uintptr_t plaintext_length;

    ErrorPtr result = iohk_jormungandr_symmetric_cipher_decrypt(password.bytes,
        password.length,
        ciphertext.bytes,
        ciphertext.length,
        &plaintext,
        &plaintext_length);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        NSData* returnValue = [NSData dataWithBytes:plaintext length:plaintext_length];
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                    messageAsArrayBuffer:returnValue];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)WALLET_RETRIEVE_FUNDS:(CDVInvokedUrlCommand*)command
{
    NSString* wallet_ptr_raw = [command.arguments objectAtIndex:0];
    NSData* block0 = [command.arguments objectAtIndex:1];

    if ([block0 isEqual:[NSNull null]]) {
        CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                                          messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];

    [self.commandDelegate runInBackground:^{
        CDVPluginResult* pluginResult = nil;

        SettingsPtr settings_ptr = nil;
        ErrorPtr result = iohk_jormungandr_wallet_retrieve_funds(wallet_ptr,
            block0.bytes,
            block0.length,
            &settings_ptr);

        if (result != nil) {
            pluginResult = jormungandr_error_to_plugin_result(result);
        } else {
            NSString* returnValue = [NSString stringWithFormat:@"%ld", (uintptr_t)settings_ptr];
            pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                             messageAsString:returnValue];
        }

        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
    }];
}

- (void)WALLET_TOTAL_FUNDS:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;
    NSString* wallet_ptr_raw = [command.arguments objectAtIndex:0];

    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];
    uint64_t value;
    ErrorPtr result = iohk_jormungandr_wallet_total_value(wallet_ptr, &value);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        NSString* returnValue = [NSString stringWithFormat:@"%lld", value];
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                         messageAsString:returnValue];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)WALLET_ID:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;
    NSString* wallet_ptr_raw = [command.arguments objectAtIndex:0];

    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];
    uint8_t id_ptr[32];
    ErrorPtr result = iohk_jormungandr_wallet_id(wallet_ptr, id_ptr);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        NSData* returnValue = [NSData dataWithBytes:id_ptr length:32];
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                    messageAsArrayBuffer:returnValue];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)WALLET_SET_STATE:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSString* wallet_ptr_raw = [command.arguments objectAtIndex:0];
    NSString* value_raw = [command.arguments objectAtIndex:1];
    NSString* counter_raw = [command.arguments objectAtIndex:2];

    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];
    uint64_t value = (uint64_t)[value_raw longLongValue];
    uint32_t counter = (uint32_t)[counter_raw longLongValue];

    ErrorPtr result = iohk_jormungandr_wallet_set_state(wallet_ptr, value, counter);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)WALLET_VOTE:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSString* wallet_ptr_raw = [command.arguments objectAtIndex:0];
    NSString* settings_ptr_raw = [command.arguments objectAtIndex:1];
    NSString* proposal_ptr_raw = [command.arguments objectAtIndex:2];
    NSString* choice_raw = [command.arguments objectAtIndex:3];

    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];
    SettingsPtr settings_ptr = (SettingsPtr)[settings_ptr_raw longLongValue];
    ProposalPtr proposal_ptr = (ProposalPtr)[proposal_ptr_raw longLongValue];
    uint8_t choice = (uint8_t)[choice_raw intValue];

    uint8_t* transaction_out = nil;
    uintptr_t len_out;

    ErrorPtr result = iohk_jormungandr_wallet_vote_cast(wallet_ptr,
        settings_ptr,
        proposal_ptr,
        choice,
        &transaction_out,
        &len_out);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        NSData* returnValue = [NSData dataWithBytes:transaction_out length:len_out];
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                    messageAsArrayBuffer:returnValue];

        iohk_jormungandr_wallet_delete_buffer(transaction_out, len_out);
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)WALLET_CONVERT:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;
    NSString* wallet_ptr_raw = [command.arguments objectAtIndex:0];
    NSString* settings_ptr_raw = [command.arguments objectAtIndex:1];

    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];
    SettingsPtr settings_ptr = (SettingsPtr)[settings_ptr_raw longLongValue];

    ConversionPtr conversion_ptr = nil;
    ErrorPtr result = iohk_jormungandr_wallet_convert(wallet_ptr, settings_ptr, &conversion_ptr);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        NSString* returnValue = [NSString stringWithFormat:@"%ld", (uintptr_t)conversion_ptr];
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                         messageAsString:returnValue];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)CONVERSION_TRANSACTIONS_SIZE:(CDVInvokedUrlCommand*)command
{
    NSString* conversion_ptr_raw = [command.arguments objectAtIndex:0];
    ConversionPtr conversion_ptr = (ConversionPtr)[conversion_ptr_raw longLongValue];
    uintptr_t value = iohk_jormungandr_wallet_convert_transactions_size(conversion_ptr);
    NSString* returnValue = [NSString stringWithFormat:@"%ld", (uintptr_t)value];
    CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                                      messageAsString:returnValue];
    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)CONVERSION_TRANSACTIONS_GET:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSString* conversion_ptr_raw = [command.arguments objectAtIndex:0];
    NSString* index_raw = [command.arguments objectAtIndex:1];

    ConversionPtr conversion_ptr = (ConversionPtr)[conversion_ptr_raw longLongValue];
    uintptr_t index = (uintptr_t)[index_raw longLongValue];

    uint8_t* transaction_out_ptr = nil;
    uintptr_t transaction_size;

    ErrorPtr result = iohk_jormungandr_wallet_convert_transactions_get(conversion_ptr,
        index,
        &transaction_out_ptr,
        &transaction_size);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        NSData* returnValue = [NSData dataWithBytes:transaction_out_ptr length:transaction_size];
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                    messageAsArrayBuffer:returnValue];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)CONVERSION_IGNORED:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSString* conversion_ptr_raw = [command.arguments objectAtIndex:0];
    ConversionPtr conversion_ptr = (ConversionPtr)[conversion_ptr_raw longLongValue];

    uint64_t value_out;
    uintptr_t ignored_out;

    ErrorPtr result =
        iohk_jormungandr_wallet_convert_ignored(conversion_ptr, &value_out, &ignored_out);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        NSDictionary* returnValue = @{
            @"value" : [NSNumber numberWithUnsignedLongLong:value_out],
            @"ignored" : [NSNumber numberWithUnsignedLong:ignored_out]
        };
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                     messageAsDictionary:returnValue];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)PROPOSAL_NEW:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSData* vote_plan_id = [command.arguments objectAtIndex:0];
    NSString* payload_type_raw = [command.arguments objectAtIndex:1];
    NSString* index_raw = [command.arguments objectAtIndex:2];
    NSString* num_choices_raw = [command.arguments objectAtIndex:3];

    if ([vote_plan_id isEqual:[NSNull null]]) {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                         messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    int32_t payload_type_num = [payload_type_raw intValue];
    PayloadType payload_type;
    switch (payload_type_num) {
        case 1:
            payload_type = PayloadType_Public;
            break;
        default:
            pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                             messageAsString:@"invalid payload type"];
            [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
            return;
    }

    uint8_t index = (uint8_t)[index_raw intValue];
    uint8_t num_choices = (uint8_t)[num_choices_raw intValue];

    ProposalPtr proposal_out_ptr = nil;
    ErrorPtr result = iohk_jormungandr_wallet_vote_proposal(vote_plan_id.bytes,
        payload_type,
        index,
        num_choices,
        &proposal_out_ptr);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        pluginResult = [CDVPluginResult
            resultWithStatus:CDVCommandStatus_OK
             messageAsString:[NSString stringWithFormat:@"%ld", (uintptr_t)proposal_out_ptr]];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)WALLET_DELETE:(CDVInvokedUrlCommand*)command
{
    NSString* wallet_ptr_raw = [command.arguments objectAtIndex:0];
    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];
    iohk_jormungandr_wallet_delete_wallet(wallet_ptr);
    CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK];
    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)SETTINGS_DELETE:(CDVInvokedUrlCommand*)command
{
    NSString* settings_ptr_raw = [command.arguments objectAtIndex:0];
    SettingsPtr settings_ptr = (SettingsPtr)[settings_ptr_raw longLongValue];
    iohk_jormungandr_wallet_delete_settings(settings_ptr);
    CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK];
    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)CONVERSION_DELETE:(CDVInvokedUrlCommand*)command
{
    NSString* conversion_ptr_raw = [command.arguments objectAtIndex:0];
    ConversionPtr conversion_ptr = (ConversionPtr)[conversion_ptr_raw longLongValue];
    iohk_jormungandr_wallet_delete_conversion(conversion_ptr);
    CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK];
    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)PROPOSAL_DELETE:(CDVInvokedUrlCommand*)command
{
    NSString* proposal_ptr_raw = [command.arguments objectAtIndex:0];
    ProposalPtr proposal_ptr = (ProposalPtr)[proposal_ptr_raw longLongValue];
    iohk_jormungandr_wallet_delete_proposal(proposal_ptr);
    CDVPluginResult* pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK];
    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

@end
