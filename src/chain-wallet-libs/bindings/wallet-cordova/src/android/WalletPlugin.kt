package com.iohk.jormungandr_wallet;

import android.util.Base64
import android.util.Log
import org.apache.cordova.*
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.text.Normalizer
import java.text.Normalizer.Form
import java.util.concurrent.atomic.AtomicInteger

class WalletPlugin
/**
 * Constructor.
 */
    : CordovaPlugin() {
    @ExperimentalUnsignedTypes
    private val wallets: MutableMap<Int, Wallet> = mutableMapOf()
    private var nextWalletId = AtomicInteger()

    @ExperimentalUnsignedTypes
    private val settingsPool: MutableMap<Int, Settings> = mutableMapOf()
    private var nextSettingsId = AtomicInteger()

    @ExperimentalUnsignedTypes
    private val pendingTransactionsPool: MutableMap<Int, List<List<UByte>>> = mutableMapOf()
    private var nextPendingTransactionsId = AtomicInteger()

    @ExperimentalUnsignedTypes
    private val proposalPool: MutableMap<Int, Proposal> = mutableMapOf()
    private var nextProposalId = AtomicInteger()

    /**
     * Sets the context of the Command. This can then be used to do things like get
     * file paths associated with the Activity.
     *
     * @param cordova The context of the main Activity.
     * @param webView The CordovaWebView Cordova is running in.
     */
    override fun initialize(cordova: CordovaInterface?, webView: CordovaWebView?) {
        super.initialize(cordova, webView)
        Log.d(TAG, "Initializing wallet plugin")
    }

    /**
     * Executes the request and returns PluginResult.
     *
     * @param action          The action to execute.
     * @param args            JSONArry of arguments for the plugin.
     * @param callbackContext The callback id used when calling back into
     * JavaScript.
     * @return True if the action was valid, false if not.
     */
    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    override fun execute(
        action: String,
        args: CordovaArgs,
        callbackContext: CallbackContext
    ): Boolean {
        Log.d(TAG, "action: $action")
        when (action) {
            "WALLET_IMPORT_KEYS" -> walletImportKeys(args, callbackContext)
            "SYMMETRIC_CIPHER_DECRYPT" -> symmetricCipherDecrypt(args, callbackContext)
            "SETTINGS_NEW" -> settingsNew(args, callbackContext)
            "SETTINGS_GET" -> settingsGet(args, callbackContext)
            "WALLET_VOTE" -> walletVote(args, callbackContext)
            "WALLET_TOTAL_FUNDS" -> walletTotalFunds(args, callbackContext)
            "WALLET_SPENDING_COUNTER" -> walletSpendingCounters(args, callbackContext)
            "WALLET_ID" -> walletId(args, callbackContext)
            "WALLET_SET_STATE" -> walletSetState(args, callbackContext)
            "PENDING_TRANSACTIONS_SIZE" -> pendingTransactionsSize(args, callbackContext)
            "PENDING_TRANSACTIONS_GET" -> pendingTransactionsGet(args, callbackContext)
            "BLOCK_DATE_FROM_SYSTEM_TIME" -> blockDateFromSystemTime(args, callbackContext)
            "MAX_EXPIRATION_DATE" -> maxExpirationDate(args, callbackContext)
            "PROPOSAL_NEW_PUBLIC" -> proposalNewPublic(args, callbackContext)
            "PROPOSAL_NEW_PRIVATE" -> proposalNewPrivate(args, callbackContext)
            "FRAGMENT_ID" -> fragmentId(args, callbackContext)
            "WALLET_DELETE" -> walletDelete(args, callbackContext)
            "SETTINGS_DELETE" -> settingsDelete(args, callbackContext)
            "PROPOSAL_DELETE" -> proposalDelete(args, callbackContext)
            "PENDING_TRANSACTIONS_DELETE" -> pendingTransactionsDelete(args, callbackContext)
            else -> {
                Log.w(TAG, "not found: $action")
                return false
            }
        }
        return true
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun walletImportKeys(args: CordovaArgs, callbackContext: CallbackContext) {
        val accountKey = args.getArrayBuffer(0).toUByteArray().toList()

        try {
            val walletId = nextWalletId.incrementAndGet()
            wallets[walletId] = Wallet(SecretKeyEd25519Extended(accountKey))
            callbackContext.success(walletId.toString())
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun symmetricCipherDecrypt(args: CordovaArgs, callbackContext: CallbackContext) {
        val password = args.getArrayBuffer(0).toUByteArray()
        val ciphertext = args.getArrayBuffer(1).toUByteArray()

        try {
            val decrypted =
                symmetricCipherDecrypt(password.toList(), ciphertext.toList()).toUByteArray()
                    .toByteArray()
            callbackContext.success(decrypted)
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun settingsNew(args: CordovaArgs, callbackContext: CallbackContext) {
        val block0Hash = args.getArrayBuffer(0).toUByteArray().toList()
        val discriminationInput = args.getInt(1)
        val fees = args[2] as JSONObject
        val block0Date = args.getString(3).toULong()
        val slotDuration = args.getString(4).toUByte()
        val era = args[5] as JSONObject
        val transactionMaxExpiryEpochs = args.getString(6).toUByte()
        try {
            val constant = fees.getString("constant").toULong()
            val coefficient = fees.getString("coefficient").toULong()
            val certificate = fees.getString("certificate").toULong()
            val certificatePoolRegistration =
                fees.getString("certificatePoolRegistration").toULong()
            val certificateStakeDelegation = fees.getString("certificateStakeDelegation").toULong()
            val certificateOwnerStakeDelegation =
                fees.getString("certificateOwnerStakeDelegation").toULong()
            val certificateVotePlan = fees.getString("certificateVotePlan").toULong()
            val certificateVoteCast = fees.getString("certificateVoteCast").toULong()
            val linearFees: LinearFee = LinearFee(
                constant, coefficient, certificate,
                PerCertificateFee(
                    certificatePoolRegistration.toULong(), certificateStakeDelegation.toULong(),
                    certificateOwnerStakeDelegation.toULong()
                ),
                PerVoteCertificateFee(
                    certificateVotePlan.toULong(),
                    certificateVoteCast.toULong()
                )
            )
            val discrimination: Discrimination =
                if (discriminationInput == 0) Discrimination.PRODUCTION else Discrimination.TEST
            val timeEra = TimeEra(
                era.getString("epochStart").toUInt(),
                era.getString("slotStart").toULong(),
                era.getString("slotsPerEpoch").toUInt()
            )
            val settingsInit = SettingsRaw(
                linearFees, discrimination, block0Hash, block0Date, slotDuration,
                timeEra, transactionMaxExpiryEpochs
            )

            val settingsId = nextSettingsId.incrementAndGet()
            settingsPool[settingsId] = Settings(settingsInit)

            callbackContext.success(settingsId.toString())
        } catch (e: java.lang.Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun settingsGet(args: CordovaArgs, callbackContext: CallbackContext) {
        val settingsId = args.getInt(0)
        try {
            val settings = settingsPool[settingsId]

            val settingsRaw = settings?.settingsRaw()

            val fees = settingsRaw?.fees
            val discrimination: Discrimination? = settingsRaw?.discrimination
            val block0Hash = settingsRaw?.block0Hash?.toUByteArray()
            val feesJson = JSONObject().put("constant", fees?.constant.toString())
                .put("coefficient", fees?.coefficient.toString())
                .put("certificate", fees?.certificate.toString())
                .put(
                    "certificatePoolRegistration",
                    fees?.perCertificateFees?.certificatePoolRegistration.toString()
                )
                .put(
                    "certificateStakeDelegation",
                    fees?.perCertificateFees?.certificateStakeDelegation.toString()
                )
                .put(
                    "certificateOwnerStakeDelegation",
                    fees?.perCertificateFees?.certificateOwnerStakeDelegation.toString()
                )
                .put(
                    "certificateVotePlan",
                    fees?.perVoteCertificateFees?.certificateVotePlan.toString()
                )
                .put(
                    "certificateVoteCast",
                    fees?.perVoteCertificateFees?.certificateVoteCast.toString()
                )
            val result: JSONObject = JSONObject().put("fees", feesJson)
                .put(
                    "discrimination",
                    if (discrimination === Discrimination.PRODUCTION) 0 else 1
                )
                .put(
                    "block0Hash", Base64.encodeToString(
                        block0Hash?.asByteArray(),
                        Base64.NO_WRAP
                    )
                )

            callbackContext.success(result)
        } catch (e: java.lang.Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun walletVote(args: CordovaArgs, callbackContext: CallbackContext) {
        val walletId = args.getInt(0)
        val settingsId = args.getInt(1)
        val proposalId = args.getInt(2)
        val choice = args.getString(3).toUByte()
        val expirationDate = args[4] as JSONObject
        val epoch = expirationDate.getString("epoch").toUInt()
        val slot = expirationDate.getString("slot").toUInt()
        val lane = args.getString(5).toUByte()

        val wallet = wallets[walletId]
        val settings = settingsPool[settingsId]
        val proposal = proposalPool[proposalId]

        val validUntil = BlockDate(epoch, slot)

        try {
            val tx = wallet?.vote(settings!!, proposal!!, choice, validUntil, lane)

            callbackContext.success(tx?.toUByteArray()?.toByteArray())
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun walletTotalFunds(args: CordovaArgs, callbackContext: CallbackContext) {
        val walletId = args.getInt(0)
        val wallet = wallets[walletId]
        try {
            callbackContext.success(wallet?.totalValue().toString())
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun walletSpendingCounters(args: CordovaArgs, callbackContext: CallbackContext) {
        val walletId = args.getInt(0)
        val wallet = wallets[walletId]
        try {
            val array = JSONArray()
            for (nonce in wallet?.spendingCounters()!!) {
                array.put(nonce)
            }
            callbackContext.success(array)
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun walletId(args: CordovaArgs, callbackContext: CallbackContext) {
        val walletId = args.getInt(0)
        val wallet = wallets[walletId]

        try {
            val id = wallet?.accountId()?.toUByteArray()?.toByteArray()
            callbackContext.success(id)
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun walletSetState(args: CordovaArgs, callbackContext: CallbackContext) {
        val walletId = args.getInt(0)
        val value = args.getString(1).toULong()
        val rawCounters = args.getJSONArray(2)

        val counters = mutableListOf<UInt>()
        for (i in 0 until rawCounters.length()) {
            counters.add(rawCounters.getString(i).toUInt())
        }

        try {
            wallets[walletId]?.setState(value, counters)
            callbackContext.success()
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun pendingTransactionsSize(args: CordovaArgs, callbackContext: CallbackContext) {
        val pendingTransactionsId = args.getInt(0)
        val pendingTransactions = pendingTransactionsPool[pendingTransactionsId]
        try {
            val size = pendingTransactions?.size
            callbackContext.success(size.toString())
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun pendingTransactionsGet(args: CordovaArgs, callbackContext: CallbackContext) {
        val pendingTransactionsId = args.getInt(0)
        val index = args.getInt(1)
        try {
            val transaction = pendingTransactionsPool[pendingTransactionsId]?.get(index)

            callbackContext.success(transaction?.toUByteArray()?.toByteArray())
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun blockDateFromSystemTime(args: CordovaArgs, callbackContext: CallbackContext) {
        val settingsId = args.getInt(0)
        val unixEpoch = args.getString(1).toULong()
        val settings = settingsPool[settingsId]
        try {
            val blockDate = blockDateFromSystemTime(settings!!, unixEpoch)
            val json =
                JSONObject().put("epoch", blockDate.epoch.toString()).put(
                    "slot",
                    blockDate.slot.toString()
                )
            callbackContext.success(json)
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun maxExpirationDate(args: CordovaArgs, callbackContext: CallbackContext) {
        val settingsId = args.getInt(0)
        val unixEpoch = args.getString(1).toULong()
        val settings = settingsPool[settingsId]
        try {
            val blockDate = maxExpirationDate(settings!!, unixEpoch)
            val json =
                JSONObject().put("epoch", blockDate.epoch.toString()).put(
                    "slot",
                    blockDate.slot.toString()
                )
            callbackContext.success(json)
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun proposalNewPublic(args: CordovaArgs, callbackContext: CallbackContext) {
        val votePlanId = args.getArrayBuffer(0).toUByteArray().toList()
        val index = args.getString(1).toUByte()
        val numChoices = args.getString(2).toUByte()
        try {
            val proposal = Proposal(votePlanId, index, numChoices, PayloadTypeConfig.Public)

            val proposalId = nextProposalId.incrementAndGet()
            proposalPool[proposalId] = proposal
            callbackContext.success(proposalId.toString())
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun proposalNewPrivate(args: CordovaArgs, callbackContext: CallbackContext) {
        val votePlanId = args.getArrayBuffer(0).toUByteArray().toList()
        val index = args.getString(1).toUByte()
        val numChoices = args.getString(2).toUByte()
        val encryptingVoteKey = args.getString(3)
        try {
            val proposal = Proposal(
                votePlanId, index, numChoices, PayloadTypeConfig.Private(
                    encryptingVoteKey
                )
            )

            val proposalId = nextProposalId.incrementAndGet()
            proposalPool[proposalId] = proposal
            callbackContext.success(proposalId.toString())
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun fragmentId(args: CordovaArgs, callbackContext: CallbackContext) {
        val transaction = args.getArrayBuffer(0).toUByteArray().toList()
        try {
            val fragment = Fragment(transaction)
            val id = fragment.id()
            fragment.destroy()
            callbackContext.success(id.toUByteArray().toByteArray())
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun walletDelete(args: CordovaArgs, callbackContext: CallbackContext) {
        val walletId: Int = args.getInt(0)
        try {
            wallets[walletId]?.destroy()
            callbackContext.success()
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun settingsDelete(args: CordovaArgs, callbackContext: CallbackContext) {
        val settingsId: Int = args.getInt(0)
        try {
            settingsPool[settingsId]?.destroy()
            callbackContext.success()
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun proposalDelete(args: CordovaArgs, callbackContext: CallbackContext) {
        val proposalId: Int = args.getInt(0)
        try {
            proposalPool.remove(proposalId)
            callbackContext.success()
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    @ExperimentalUnsignedTypes
    @Throws(JSONException::class)
    private fun pendingTransactionsDelete(args: CordovaArgs, callbackContext: CallbackContext) {
        val pid: Int = args.getInt(0)
        try {
            pendingTransactionsPool.remove(pid)
            callbackContext.success()
        } catch (e: Exception) {
            callbackContext.error(e.message)
        }
    }

    companion object {
        const val TAG = "WALLET"
    }
}

