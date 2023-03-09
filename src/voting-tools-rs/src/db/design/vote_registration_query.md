# Vote Registration DB Query

## Complete Query

```sql
SELECT meta_table.id as reg_id,
       sig_table.id as sig_id,
       meta_table.tx_id as reg_tx_id,
       sig_table.tx_id as sig_tx_id,
       "block"."slot_no",
       meta_table.key as reg_key,
       meta_table.json as reg_json,
       meta_table.bytes as reg_bytes,
       sig_table.key as sig_key,
       sig_table.json as sig_json,
       sig_table.bytes as sig_bytes
FROM ((("tx_metadata" AS "meta_table" INNER JOIN "tx"
        ON ("tx"."id" = "meta_table"."tx_id")) INNER JOIN "tx_metadata" AS "sig_table"
       ON ("sig_table"."tx_id" = "meta_table"."tx_id")) INNER JOIN "block" ON ("block"."id" = "tx"."block_id"))
WHERE (((("meta_table"."key" = 61284) AND ("sig_table"."key" = 61285))) AND
       (("block"."slot_no" >= ???) AND ("block"."slot_no" <= ???)))
ORDER BY meta_table.tx_id DESC;
```

### Explanation of the Query

1. Join all the tables we need to get data from

    ```sql
    FROM (
        (
            ("tx_metadata" AS "meta_table" INNER JOIN "tx" ON ("tx"."id" = "meta_table"."tx_id"))
            INNER JOIN "tx_metadata" AS "sig_table" ON ("sig_table"."tx_id" = "meta_table"."tx_id")
        )
        INNER JOIN "block" ON ("block"."id" = "tx"."block_id")
    )
    ```

    Joins the `tx_metadata` table twice to itself, because we need two records to appear as one.
    Join with the `block` table, so we can get data about the block directly.

2. Filter out all metadata we are not interested in.

    ```sql
    WHERE (((("meta_table"."key" = 61284) AND ("sig_table"."key" = 61285))) AND
        (("block"."slot_no" >= ???) AND ("block"."slot_no" <= ???)))
    ```

    Looks for Metadata Keys 61284 and 61285, because valid transaction needs both keys.
    Limit the search to a particular slot range, which aligns the snapshot.

3. Get ALL data present which might be useful

    ```sql
    SELECT meta_table.id as reg_id,
       sig_table.id as sig_id,
       meta_table.tx_id as reg_tx_id,
       sig_table.tx_id as sig_tx_id,
       "block"."slot_no",
       meta_table.key as reg_key,
       meta_table.json as reg_json,
       meta_table.bytes as reg_bytes,
       sig_table.key as sig_key,
       sig_table.json as sig_json,
       sig_table.bytes as sig_bytes
    ```

    We need to use both the raw binary and json values of a transaction.
    Other fields help us to identify the exact transaction on the blockchain.

4. Order the results, so that the newest entries occur first.

    ```sql
    ORDER BY meta_table.tx_id DESC;
    ```

### Query Summary

This query gathers all possible registration transactions with MINIMAL Filtering to exclude bad transactions.
This is because bad transactions are difficult to properly detect with SQL, AND we want to be able to report ALL
transactions which might have been OK but are, in fact, bad for some reason.

## Results Processing

### Validate Signatures

1. Take the RAW data for every registration as reported by the query above.
2. Create TWO lists or maps:
   1. `valid_sig` = All registrations found by the query where the Signature check passes.
      1. Signature check is to be done against the raw binary data of the registration, not reconstructed from the json.
   2. `invalid_sig` = All the registrations which failed signature checks.
      1. These records need to be emitted in the errors JSON report.

There are over 170,000 registrations.
This number will only grow.
Therefore signature validation must be done in parallel with a parallel iterator.

### Validate the Registration Records

Having validated the signatures, the collection of `valid_sig` transactions are checked to see if they are proper according to either CIP-15 or CIP-36.

This validation would also include a check that the `purpose` is correct.

Invalid registrations which fail this check are added to a `invalid_registration` list, and reported in the errors JSON file.
The reason the transaction failed validation should be included, not just that it was invalid.

### Find the latest Registration Record for each stake address key

The registration transaction list now ONLY includes valid registrations.
It is iterated again, and the most recent registration for each `public stake address` is extracted.

The "obsolete" transactions should be added to the `invalid_transactions` list.

The latest transaction is defined as the transaction with the largest `nonce` field.
Any transaction without a `nonce` field should be rejected as invalid, with that reason, when the registration is validated.

### Voting Power

The fully validated and latest list of registrations is then fed to the voting power calculation logic.

## CIP-15/CIP-36

* [CIP-15](https://github.com/cardano-foundation/CIPs/tree/master/CIP-0015#registration-metadata-format)
* [CIP-36](https://github.com/cardano-foundation/CIPs/tree/master/CIP-0036#example)

All new registrations should be in CIP-36 form, however many will still be in CIP-15 form.

## Final Notes

The processing of these transactions should occur in this sequence.
We should not be combining this processing in a single iteration, but doing discreet validation steps.
Parallel iterators should be used at all stages because each record can be validated in parallel.
