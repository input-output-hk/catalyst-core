# Vote Registration DB Query

## Complete Query

```sql
WITH meta_table AS (
        select tx_id, json AS metadata from tx_metadata where key = '61284'
     ),
     sig_table AS (
        select tx_id, json AS signature from tx_metadata where key = '61285'
     )
SELECT tx.hash, tx_id, metadata, signature
FROM meta_table
         INNER JOIN tx ON tx.id = meta_table.tx_id
         INNER JOIN sig_table USING (tx_id)
         INNER JOIN block ON block.id = tx.block_id
WHERE block.slot_no <= ???
ORDER BY metadata -> '4' ASC
```

## No Filter on Block Slot Number

```sql
WITH meta_table AS (select tx_id, json AS metadata from tx_metadata where key = '61284'),
     sig_table AS (select tx_id, json AS signature from tx_metadata where key = '61285')
SELECT tx.hash, tx_id, metadata, signature
FROM meta_table
         INNER JOIN tx ON tx.id = meta_table.tx_id
         INNER JOIN sig_table USING (tx_id)
ORDER BY metadata -> '4' ASC

```

## Explanation of the Query

1. Make a temp table called `meta_table` from the `tx_metadata` table where the `key` field = `61284`,  BUT only get the `tx_id` and `json` fields.

    ```sql
    meta_table AS (
        select tx_id, json AS metadata from tx_metadata where key = '61284'
     ),
    ```

    Sample Results:

    ```csv
    tx_id, metadata
    2936412,"{\"1\": \"0x35a169ab354f1936934e0d665728a68922728f83913a27a77442f01c0b4c06ca\", \"2\": \"0x9b5bd6d6a8142c69586493d367a3a561e26d76037ac3928939fdc374a110bbdf\", \"3\": \"0xe0edc52808d58a420ce2df3da8c7118c858c7aa136fa7b2b1baa3e81ed\", \"4\": 47139419}"
    2958136,"{\"1\": \"0xa45eaeede2fd818d2bd626691b37e9851306511890d9e55e7ff8ad8457b054b5\", \"2\": \"0x73042b25fc3aae6aacf8cbe38659b0e8abbfab7d92647a49c83f8da6191dff55\", \"3\": \"0xe0db8ec58f9fa297093e286f81d37bea7154209064956254d5d4e2108d\", \"4\": 47193845}"
    2958481,"{\"1\": \"0x23d1875ccc4676724362a0ef5e1386c326b324e9175d2271e1ae50bf8835faaf\", \"2\": \"0x4b99f45fdc71e8ef3c941a8691cdf423ab4acded4f90fba9be8d62b0fcee1ca1\", \"3\": \"0xe059725f52ca144d6ad5efac538dbf632391915ea6c0d54ae66fae748f\", \"4\": 47194820}"
    2969691,"{\"1\": \"0xe5cec9fa71e98e7efa28839aed9314ceda38cd3bad4f8cffd861f12f5ff4a666\", \"2\": \"0xef20a5b845f1b8fabacecd3fc72bfc09f38ba487d2c504da8ed531dabfe3d26c\", \"3\": \"0xe0db5e8ece0982acc4883de67c2e3411cc26bd56686a162074998c02bc\", \"4\": 50048562}"
    2969719,"{\"1\": \"0x5b14c711b9e8b0c685bf14cb97894e9656b5a0bb970e5f22c9552e70568b6e16\", \"2\": \"0xef20a5b845f1b8fabacecd3fc72bfc09f38ba487d2c504da8ed531dabfe3d26c\", \"3\": \"0xe0db5e8ece0982acc4883de67c2e3411cc26bd56686a162074998c02bc\", \"4\": 50048687}"
    ```

    See [CIP-15](https://github.com/cardano-foundation/CIPs/tree/master/CIP-0015) for `metadata` format.

2. Make a temp table called `sig_table` from the `tx_metadata` table where the `key` field = `61285`,  BUT only get the `tx_id` and `json` fields.

    ```sql
     sig_table AS (
        select tx_id, json AS signature from tx_metadata where key = '61285'
     )

    ```

    Sample Results:

    ```csv
    tx_id, signature
    2958481,"{\"1\": \"0x5f6a6466089e3e1100591cdec2803332064abe8406f8627ef59d871af6650b34b98e8856882ea58a45c9288502344ce52c682887b0f174fb283a89dc20efff00\"}"
    2969691,"{\"1\": \"0x701a31f652e5870699252eb61a85714060c7e40957483d383d52b8d1835e62f6a4ee4c57a1c182b8618582f6fb0e2aca8ce49eb2b942ba3286304aa5465b0f04\"}"
    2969719,"{\"1\": \"0x94cdbdb1d679141ef0e9fa22ff8f1a087de8a05adcfe2be1291afe0674e09fecf4fc25f729da937d1e5a4e7253e72546f0c33d7ab6b5d08fc32b6b3babc46700\"}"
    3016397,"{\"1\": \"0x384b75ce44ecdf824a78e31dc11eb1419807b961db924c752a569b274269117ac1b42407cc46b277dcc960c33074690a1de69774c8a734d7c55c80179a4c6d0d\"}"
    3016678,"{\"1\": \"0x703cc1008a6bd53021494422da95650e962528e6d7dd0bdb3db6f63c701821517537df369ffd2bd1189e292403e328c2f6c54f0cea3ced1348ec981da1f3f602\"}"
    ```

    See [CIP-15](https://github.com/cardano-foundation/CIPs/tree/master/CIP-0015) for `signature` format.

3. Combine all records from the temp `meta_table` and `sig_table` and the get the `tx.hash` field from the `tx` table, where the `tx_id` is the same.

    ```sql
    SELECT tx.hash, tx_id, metadata, signature
    FROM meta_table
            INNER JOIN tx ON tx.id = meta_table.tx_id
            INNER JOIN sig_table USING (tx_id)
    ```

    Sample Results:

    ```csv
    hash, tx_id, metadata, signature
    0x0041BABE0209D6FF6D0E719ED7484225C0B2916DDB6B76E135ACA8F6F234EDB9,2958481,"{\"1\": \"0x23d1875ccc4676724362a0ef5e1386c326b324e9175d2271e1ae50bf8835faaf\", \"2\": \"0x4b99f45fdc71e8ef3c941a8691cdf423ab4acded4f90fba9be8d62b0fcee1ca1\", \"3\": \"0xe059725f52ca144d6ad5efac538dbf632391915ea6c0d54ae66fae748f\", \"4\": 47194820}","{\"1\": \"0x5f6a6466089e3e1100591cdec2803332064abe8406f8627ef59d871af6650b34b98e8856882ea58a45c9288502344ce52c682887b0f174fb283a89dc20efff00\"}"
    0x0106485F2AC9B94D725F8F827211B43A980AF0F20122CBD6A17CEAB6C76D8B4F,2969691,"{\"1\": \"0xe5cec9fa71e98e7efa28839aed9314ceda38cd3bad4f8cffd861f12f5ff4a666\", \"2\": \"0xef20a5b845f1b8fabacecd3fc72bfc09f38ba487d2c504da8ed531dabfe3d26c\", \"3\": \"0xe0db5e8ece0982acc4883de67c2e3411cc26bd56686a162074998c02bc\", \"4\": 50048562}","{\"1\": \"0x701a31f652e5870699252eb61a85714060c7e40957483d383d52b8d1835e62f6a4ee4c57a1c182b8618582f6fb0e2aca8ce49eb2b942ba3286304aa5465b0f04\"}"
    0xF373397131734F0C5A8E3A6F9B24BCB3005C720144D0A3C074173EE2F0DAD25F,2969719,"{\"1\": \"0x5b14c711b9e8b0c685bf14cb97894e9656b5a0bb970e5f22c9552e70568b6e16\", \"2\": \"0xef20a5b845f1b8fabacecd3fc72bfc09f38ba487d2c504da8ed531dabfe3d26c\", \"3\": \"0xe0db5e8ece0982acc4883de67c2e3411cc26bd56686a162074998c02bc\", \"4\": 50048687}","{\"1\": \"0x94cdbdb1d679141ef0e9fa22ff8f1a087de8a05adcfe2be1291afe0674e09fecf4fc25f729da937d1e5a4e7253e72546f0c33d7ab6b5d08fc32b6b3babc46700\"}"
    0xEE64FBD806CAAB4EFC1FFDD46E56D121B3356D2BEC7FFAA89C126BC005BF97BB,3016397,"{\"1\": \"0xc681f72c2952be387988504a2e4ae0bbe3b13c4aff17a8bc65a926145d8f3760\", \"2\": \"0x8c9d924397918f7f4ecfb22fab2aea45ce28f26d687680800886492da6ad2e53\", \"3\": \"0xe063ea8c5404f9ed9ae80d95b5544857b2011e3f26b63ddc3be1abd42d\", \"4\": 50239868}","{\"1\": \"0x384b75ce44ecdf824a78e31dc11eb1419807b961db924c752a569b274269117ac1b42407cc46b277dcc960c33074690a1de69774c8a734d7c55c80179a4c6d0d\"}"
    0x4E746AA0F0E2D16EF580D957F564E1B02F6AEB596C0D0546CC5F551415C28475,3016678,"{\"1\": \"0x2ebf9eb57b750032ee284c4afb1bd7a57e59a13fea29b5a7370a620e58d0fa23\", \"2\": \"0x8c9d924397918f7f4ecfb22fab2aea45ce28f26d687680800886492da6ad2e53\", \"3\": \"0xe063ea8c5404f9ed9ae80d95b5544857b2011e3f26b63ddc3be1abd42d\", \"4\": 50240410}","{\"1\": \"0x703cc1008a6bd53021494422da95650e962528e6d7dd0bdb3db6f63c701821517537df369ffd2bd1189e292403e328c2f6c54f0cea3ced1348ec981da1f3f602\"}"

4. Optionally, Only include records which occurred BEFORE or EQUAL to a specified `block.slot_no`.

    ```sql
        INNER JOIN block ON block.id = tx.block_id
    WHERE block.slot_no <= ???
    ```

    This only filters the records. The result set is otherwise unchanged. The
    filter is used to lock the maximum point of time for the snapshot purpose,
    so that later transactions do not interfere with the snapshot.

    Filtering this way makes the snapshot process repeatable.

5. Order all the results by the `4` field in the json `metadata` column.  This is what the `metadata -> '4'` means.  The `4` field of the json `metadata` is defined to be an every increasing ***nonce*** value.  So, this makes sure the highest valued ***nonce*** for the same `staking key` (json field 2) will occur last.

    ```sql
    ORDER BY metadata -> '4' ASC;
    ```

    Note:
    * `ASC` on the ORDER BY is redundant but useful for clarity.

6. Post process this data to only have the latest stake key registration.

    ```text
    For each unique stake key:
        Keep only the registration record with:
            The Highest Block Slot Number AND
            Lowest Transaction ID.
    ```

## Summary

This query gathers all of the registration transactions in the DB, adds the hash from the transaction they are from to the result set, and then orders them by their `nonce` value.  Post processing then further reduces the result set to the latest registration transaction for each stake key.

## Optimized Version of the Query

```sql
WITH meta_table AS (select tx_id, json AS metadata from tx_metadata where key = '61284'),
     sig_table AS (select tx_id, json AS signature from tx_metadata where key = '61285')
SELECT DISTINCT ON (metadata->'2') * FROM (
    SELECT * FROM (
        SELECT  tx.hash, tx_id, metadata, signature,
                block.slot_no AS block_slot_no,
                (
                    CASE WHEN (metadata->>'5')~E'^\\d+$' THEN
                        CAST ((metadata->>'5') AS INTEGER)
                    WHEN metadata->>'5' IS NULL THEN
                        0
                    ELSE
                        -1
                    END
                ) AS purpose
        FROM meta_table
                 INNER JOIN tx ON tx.id = meta_table.tx_id
                 INNER JOIN sig_table USING (tx_id)
                 INNER JOIN block ON block.id = tx.block_id
        WHERE metadata ? '1' AND metadata ? '2' AND metadata ? '3' AND metadata ? '4' AND
              signature ? '1' AND
              block.slot_no > 0 AND block.slot_no <= 62510369
        ORDER BY metadata -> '2', metadata -> '4' DESC, tx_id ASC
    ) as all_registrations WHERE purpose = 0
) as purpose_registrations;
```

This query:

1. Can get registrations on any period, defined between two block numbers, inclusively:

   ```sql
   block.slot_no > 0 AND block.slot_no <= 62510369
   ```

   must be parameterized in the query above.
2. It also filters out ONLY the latest registration for each stake key, as per the original iterative logic. (Eliminates the need to do it in code iteratively).

    ```sql
    SELECT DISTINCT ON (metadata->'2') * FROM (
        --- ...
        ORDER BY metadata -> '2', metadata -> '4' DESC, tx_id ASC
    ```

    Is the part of the query which filters that, and does NOT need parameterization.

3. Ensures that the metadata and signature have the minimum number of required
   fields, which pre-filters some badly formed transactions from the results:

    ```sql
    WHERE metadata ? '1' AND metadata ? '2' AND metadata ? '3' AND metadata ? '4' AND
          signature ? '1' AND
    ```

    Is the part of the query which does that, and does not need to be parameterized.

4. Adds a new column called `purpose` which is the voting purpose of the
   registration in alignment with CIP-36. The query is then filtered on the
   `purpose`, so that ONLY registrations for the required purpose are found.
   This properly does not exclude multiple registrations for multiple purposes.

    ```sql
    (
        CASE WHEN (metadata->>'5')~E'^\\d+$' THEN
            CAST ((metadata->>'5') AS INTEGER)
        WHEN metadata->>'5' IS NULL THEN
            0
        ELSE
            -1
        END
    ) AS purpose
    ```

    This SQL creates the purpose column. IF the `5` field is present in the
    metadata, AND its a properly formed integer, it becomes the purpose (as an
    integer). If it is NOT present, it is defaulted to purpose 0 (Cip-15->36
    upgrade). Otherwise the purpose is set to -1 to indicate the registration
    record is invalid and should be ignored.

    ```sql
    WHERE purpose = 0
    ```

    This SQL determines which purpose the registration records are returned for,
    and needs to be parameterized.

This query is now so fast that it should not require to be divided into multiple queries bounded by `block.slot_no` and should be implemented straight until performance of it proves to be an issue.

## Outstanding Questions

1. Is the Vote Snapshot tool supposed to verify the Signature?
2. Does the Haskell node will only allow transactions that are properly signed?

If the answer to 1 is NO and 2 is YES then the query above can be further optimized.

If the answer to 1 is YES, then regardless of the answer to 2, the query will need to return all registrations, and check the signature of each, in order and only pick the FIRST which validates.  The transaction can still return the data in the correct priority order which will greatly simplify that process.

## Notes on processing if registration validation is needed

If the registrations are needed to be validated, such that the LATEST valid registration is the one used, then the following changes will need to be made:

1. Remove `SELECT DISTINCT ON (metadata->'2') * FROM (` from the optimized query.
2. This will result in ALL registrations being returned, BUT they will be already grouped by stake key, and in proper order.
3. Iterate the results, and run `is_valid` on the registration record.
4. If it is valid:
   1. save it as a valid registration
   2. skip all following records (if any) with the same stake key.
   3. Repeat for each new stake key encountered in the unfiltered results.
5. If it is invalid, discard and try the next record.

## CIP-15/CIP-36

* [CIP-15](https://github.com/cardano-foundation/CIPs/tree/master/CIP-0015#registration-metadata-format)
* [CIP-36](https://github.com/cardano-foundation/CIPs/tree/master/CIP-0036#example)

All new registrations should be in CIP-36 form, however many will still be in CIP-15 form.  In order to simplify processing, when the registrations are read from the database, and validated, they should be converted to CIP-36 form by the following method:

* If Field `1` of key `61284` is a single string of the form
  `"0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663"`
  it should be converted to an array of the form
  `[["0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663",1]]`

    This transformations turns the original registration into a single registration with a weight of 1.  Further processing can then assume the field is a proper CIP-36 formed delegation.

* If Field `1` of key `61284` is an array of the form `[["0xa6a3c0447aeb9cc54cf6422ba32b294e5e1c3ef6d782f2acff4a70694c4d1663", 1], ["0x00588e8e1d18cba576a4d35758069fe94e53f638b6faf7c07b8abd2bc5c5cdee", 3]]` it is a proper CIP-31 voting key delegation with the individual keys and weights defined and should be used as is.

* Any other structure of field `1` of key `61284` is an error and the registration should be ignored.

* The optimized query above will properly filter registrations to their purpose,
  and should be parameterized accordingly. The `purpose` field in the result
  data set can be used in the code to prevent the need to repeat this logic, however once the registrations are filtered there may be no further need for the `purpose`.

* IF multiple purposes were supported at the same time, then the query above would need to be executed once for each unique `purpose`.
