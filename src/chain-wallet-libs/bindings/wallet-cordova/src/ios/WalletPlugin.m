#import "WalletPlugin.h"
#include <bits/stdint-uintn.h>

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

- (void)WALLET_SPENDING_COUNTER:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;
    NSString* wallet_ptr_raw = [command.arguments objectAtIndex:0];

    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];
    uint32_t value;
    ErrorPtr result = iohk_jormungandr_wallet_spending_counter(wallet_ptr, &value);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        NSString* returnValue = [NSString stringWithFormat:@"%u", value];
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                         messageAsString:returnValue];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
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
    NSDictionary* expirationDate = [command.arguments objectAtIndex:4];

    uint32_t epoch = (uint32_t)[expirationDate[@"epoch"] longLongValue];
    uint32_t slot = (uint32_t)[expirationDate[@"slot"] longLongValue];

    BlockDate date = { epoch, slot };

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
        date,
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
    NSDictionary* expirationDate = [command.arguments objectAtIndex:2];

    uint32_t epoch = (uint32_t)[expirationDate[@"epoch"] longLongValue];
    uint32_t slot = (uint32_t)[expirationDate[@"slot"] longLongValue];
    BlockDate date = { epoch, slot };

    WalletPtr wallet_ptr = (WalletPtr)[wallet_ptr_raw longLongValue];
    SettingsPtr settings_ptr = (SettingsPtr)[settings_ptr_raw longLongValue];

    ConversionPtr conversion_ptr = nil;
    ErrorPtr result =
        iohk_jormungandr_wallet_convert(wallet_ptr, settings_ptr, date, &conversion_ptr);

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

- (void)PROPOSAL_NEW_PUBLIC:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSData* vote_plan_id = [command.arguments objectAtIndex:0];
    NSString* index_raw = [command.arguments objectAtIndex:1];
    NSString* num_choices_raw = [command.arguments objectAtIndex:2];

    if ([vote_plan_id isEqual:[NSNull null]]) {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                         messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    uint8_t index = (uint8_t)[index_raw intValue];
    uint8_t num_choices = (uint8_t)[num_choices_raw intValue];

    ProposalPtr proposal_out_ptr = nil;
    ErrorPtr result = iohk_jormungandr_vote_proposal_new_public(vote_plan_id.bytes,
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

- (void)PROPOSAL_NEW_PRIVATE:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSData* vote_plan_id = [command.arguments objectAtIndex:0];
    NSString* index_raw = [command.arguments objectAtIndex:1];
    NSString* num_choices_raw = [command.arguments objectAtIndex:2];
    NSString* encrypting_key = [command.arguments objectAtIndex:3];

    if ([vote_plan_id isEqual:[NSNull null]]) {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                         messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    uint8_t index = (uint8_t)[index_raw intValue];
    uint8_t num_choices = (uint8_t)[num_choices_raw intValue];

    ProposalPtr proposal_out_ptr = nil;
    ErrorPtr result = iohk_jormungandr_vote_proposal_new_private(vote_plan_id.bytes,
        index,
        num_choices,
        [encrypting_key UTF8String],
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

- (void)SETTINGS_NEW:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSData* block0_hash = [command.arguments objectAtIndex:0];
    NSString* discrimination_raw = [command.arguments objectAtIndex:1];
    NSDictionary* fees = [command.arguments objectAtIndex:2];
    NSString* block0_date_raw = [command.arguments objectAtIndex:3];
    uint64_t block0_date = (uint64_t)[block0_date_raw longLongValue];
    NSString* slot_duration_raw = [command.arguments objectAtIndex:4];
    uint8_t slot_duration = (uint8_t)[slot_duration_raw longLongValue];
    NSDictionary* era = [command.arguments objectAtIndex:5];
    NSString* max_expiry_epochs_raw = [command.arguments objectAtIndex:6];
    uint8_t max_expiry_epochs = (uint8_t)[slot_duration_raw longLongValue];

    if ([block0_hash isEqual:[NSNull null]] || [fees isEqual:[NSNull null]]) {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                         messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    Discrimination discrimination = (uint8_t)[discrimination_raw intValue] == 0
                                        ? Discrimination_Production
                                        : Discrimination_Test;

    uint64_t constant = (uint64_t)[fees[@"constant"] longLongValue];
    uint64_t coefficient = (uint64_t)[fees[@"coefficient"] longLongValue];
    uint64_t certificate = (uint64_t)[fees[@"certificate"] longLongValue];

    uint64_t certificate_pool_registration =
        (uint64_t)[fees[@"certificatePoolRegistration"] longLongValue];
    uint64_t certificate_stake_delegation =
        (uint64_t)[fees[@"certificateStakeDelegation"] longLongValue];
    uint64_t certificate_owner_stake_delegation =
        (uint64_t)[fees[@"certificateOwnerStakeDelegation"] longLongValue];
    PerCertificateFee per_certificate_fees = { certificate_pool_registration,
        certificate_stake_delegation,
        certificate_owner_stake_delegation };

    uint64_t certificate_vote_plan = (uint64_t)[fees[@"certificateVotePlan"] longLongValue];
    uint64_t certificate_vote_cast = (uint64_t)[fees[@"certificateVoteCast"] longLongValue];
    PerVoteCertificateFee per_vote_certificate_fees = { certificate_vote_plan,
        certificate_vote_cast };

    LinearFee linear_fees = { constant,
        coefficient,
        certificate,
        per_certificate_fees,
        per_vote_certificate_fees };

    uint32_t epoch_start = (uint32_t)[era[@"epochStart"] longLongValue];
    uint64_t slot_start = (uint64_t)[era[@"slotStart"] longLongValue];
    uint32_t slots_per_epoch = (uint32_t)[era[@"slotsPerEpoch"] longLongValue];

    TimeEra time_era = {
        epoch_start,
        slot_start,
        slots_per_epoch,
    };

    SettingsInit settings_init = {
        linear_fees,
        discrimination,
        block0_hash.bytes,
        block0_date,
        slot_duration,
        time_era,
        max_expiry_epochs,
    };

    SettingsPtr settings_out_ptr = nil;
    ErrorPtr result = iohk_jormungandr_wallet_settings_new(settings_init, &settings_out_ptr);

    if (result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result);
    } else {
        pluginResult = [CDVPluginResult
            resultWithStatus:CDVCommandStatus_OK
             messageAsString:[NSString stringWithFormat:@"%ld", (uintptr_t)settings_out_ptr]];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)SETTINGS_GET:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSString* settings_ptr_raw = [command.arguments objectAtIndex:0];
    SettingsPtr settings_ptr = (uintptr_t)[settings_ptr_raw longLongValue];

    if (settings_ptr == nil) {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                         messageAsString:@"invalid settings pointer"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    Discrimination discrimination;
    uint8_t block0_hash[32];
    LinearFee linear_fees;

    ErrorPtr discrimination_result =
        iohk_jormungandr_wallet_settings_discrimination(settings_ptr, &discrimination);
    ErrorPtr block0_hash_result =
        iohk_jormungandr_wallet_settings_block0_hash(settings_ptr, &block0_hash);
    ErrorPtr linear_fees_result = iohk_jormungandr_wallet_settings_fees(settings_ptr, &linear_fees);

    if (discrimination_result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(discrimination_result);
    } else if (block0_hash_result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(block0_hash_result);
    } else if (linear_fees_result != nil) {
        pluginResult = jormungandr_error_to_plugin_result(linear_fees_result);
    } else {
        NSDictionary* result = @{
            @"discrimination" : [NSNumber numberWithUnsignedInt:(uint8_t)discrimination],
            @"block0Hash" : [[NSData dataWithBytes:block0_hash
                                            length:32] base64EncodedStringWithOptions:0],
            @"fees" : @{
                @"constant" : [NSString stringWithFormat:@"%ld", linear_fees.constant],
                @"coefficient" : [NSString stringWithFormat:@"%ld", linear_fees.coefficient],
                @"certificate" : [NSString stringWithFormat:@"%ld", linear_fees.certificate],
                @"certificatePoolRegistration" :
                    [NSString stringWithFormat:@"%ld",
                              linear_fees.per_certificate_fees.certificate_pool_registration],
                @"certificateStakeDelegation" :
                    [NSString stringWithFormat:@"%ld",
                              linear_fees.per_certificate_fees.certificate_stake_delegation],
                @"certificateOwnerStakeDelegation" :
                    [NSString stringWithFormat:@"%ld",
                              linear_fees.per_certificate_fees.certificate_owner_stake_delegation],
                @"certificateVotePlan" :
                    [NSString stringWithFormat:@"%ld",
                              linear_fees.per_vote_certificate_fees.certificate_vote_plan],
                @"certificateVoteCast" :
                    [NSString stringWithFormat:@"%ld",
                              linear_fees.per_vote_certificate_fees.certificate_vote_cast],
            },
        };
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                     messageAsDictionary:result];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)FRAGMENT_ID:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSData* fragment_raw = [command.arguments objectAtIndex:0];

    if ([fragment_raw isEqual:[NSNull null]]) {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                         messageAsString:@"missing argument"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    FragmentPtr fragment_ptr = nil;
    ErrorPtr result_fragment_from_raw =
        iohk_jormungandr_fragment_from_raw(fragment_raw.bytes, fragment_raw.length, &fragment_ptr);

    if (result_fragment_from_raw != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result_fragment_from_raw);
    } else {
        uint8_t fragment_id[32];
        ErrorPtr result_fragment_id = iohk_jormungandr_fragment_id(fragment_ptr, fragment_id);

        if (result_fragment_id != nil) {
            pluginResult = jormungandr_error_to_plugin_result(result_fragment_id);
        } else {
            NSData* returnValue = [NSData dataWithBytes:fragment_id length:32];
            pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                        messageAsArrayBuffer:returnValue];
        }

        iohk_jormungandr_delete_fragment(fragment_ptr);
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)BLOCK_DATE_FROM_SYSTEM_TIME:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSString* settings_ptr_raw = [command.arguments objectAtIndex:0];
    SettingsPtr settings_ptr = (uintptr_t)[settings_ptr_raw longLongValue];

    NSString* seconds_raw = [command.arguments objectAtIndex:1];

    uint64_t date = (uint64_t)[seconds_raw longLongValue];

    if (settings_ptr == nil) {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                         messageAsString:@"invalid settings pointer"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    BlockDate block_date_out;

    ErrorPtr result_c_call =
        iohk_jormungandr_block_date_from_system_time(settings_ptr, date, &block_date_out);

    if (result_c_call != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result_c_call);
    } else {
        NSDictionary* result = @{
            @"epoch" : [NSNumber numberWithUnsignedInt:block_date_out.epoch],
            @"slot" : [NSNumber numberWithUnsignedInt:block_date_out.slot],
        };

        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                     messageAsDictionary:result];
    }

    [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
}

- (void)MAX_EXPIRATION_DATE:(CDVInvokedUrlCommand*)command
{
    CDVPluginResult* pluginResult = nil;

    NSString* settings_ptr_raw = [command.arguments objectAtIndex:0];
    SettingsPtr settings_ptr = (uintptr_t)[settings_ptr_raw longLongValue];

    NSString* seconds_raw = [command.arguments objectAtIndex:1];

    uint64_t date = (uint64_t)[seconds_raw longLongValue];

    if (settings_ptr == nil) {
        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_ERROR
                                         messageAsString:@"invalid settings pointer"];
        [self.commandDelegate sendPluginResult:pluginResult callbackId:command.callbackId];
        return;
    }

    BlockDate block_date_out;

    ErrorPtr result_c_call =
        iohk_jormungandr_max_expiration_date(settings_ptr, date, &block_date_out);

    if (result_c_call != nil) {
        pluginResult = jormungandr_error_to_plugin_result(result_c_call);
    } else {
        NSDictionary* result = @{
            @"epoch" : [NSNumber numberWithUnsignedInt:block_date_out.epoch],
            @"slot" : [NSNumber numberWithUnsignedInt:block_date_out.slot],
        };

        pluginResult = [CDVPluginResult resultWithStatus:CDVCommandStatus_OK
                                     messageAsDictionary:result];
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
