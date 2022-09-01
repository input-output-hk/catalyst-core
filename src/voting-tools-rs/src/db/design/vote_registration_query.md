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

## Summary

This query gathers all of the registration transactions in the DB, adds the hash from the transaction they are from to the result set, and then orders them by their `nonce` value.
