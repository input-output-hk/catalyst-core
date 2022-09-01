# Snapshot UTXO Query

## Filter on Block Slot Number

```sql
CREATE TEMPORARY TABLE IF NOT EXISTS tx_out_snapshot AS (
    SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out
        INNER JOIN tx ON tx_out.tx_id = tx.id
        INNER JOIN block ON tx.block_id = block.id
        INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
    WHERE block.slot_no <= ???);

CREATE TEMPORARY TABLE IF NOT EXISTS tx_in_snapshot AS (
    SELECT tx_in.* FROM tx_in
          INNER JOIN tx ON tx_in.tx_in_id = tx.id
          INNER JOIN block ON tx.block_id = block.id
    WHERE block.slot_no <= ???);

CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (
    SELECT tx_out_snapshot.* FROM tx_out_snapshot
        LEFT OUTER JOIN tx_in_snapshot ON
            tx_out_snapshot.tx_id = tx_in_snapshot.tx_out_id AND
            tx_out_snapshot.index = tx_in_snapshot.tx_out_index
        WHERE tx_in_snapshot.tx_in_id IS NULL);

CREATE INDEX IF NOT EXISTS utxo_snapshot_stake_credential ON utxo_snapshot(stake_credential);
ANALYZE tx_out_snapshot;
ANALYZE tx_in_snapshot;
ANALYZE utxo_snapshot;
```

Speed:

This is a really long running set of operations:

For example, with a test set of data:

* `tx_out` = 13798161 rows.
* `tx_in` = 9642743 rows.

```text
cexplorer.public> CREATE TEMPORARY TABLE IF NOT EXISTS tx_out_snapshot AS (
                      SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out
                          INNER JOIN tx ON tx_out.tx_id = tx.id
                          INNER JOIN block ON tx.block_id = block.id
                          INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
                      WHERE block.slot_no <= 62510369)
[2022-09-01 16:13:53] 8,755,772 rows affected in 18 s 714 ms
cexplorer.public> CREATE TEMPORARY TABLE IF NOT EXISTS tx_in_snapshot AS (
                      SELECT tx_in.* FROM tx_in
                            INNER JOIN tx ON tx_in.tx_in_id = tx.id
                            INNER JOIN block ON tx.block_id = block.id
                      WHERE block.slot_no <= 62510369)
[2022-09-01 16:14:04] 9,642,743 rows affected in 10 s 915 ms
cexplorer.public> CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (
                      SELECT tx_out_snapshot.* FROM tx_out_snapshot
                          LEFT OUTER JOIN tx_in_snapshot ON
                              tx_out_snapshot.tx_id = tx_in_snapshot.tx_out_id AND
                              tx_out_snapshot.index = tx_in_snapshot.tx_out_index
                          WHERE tx_in_snapshot.tx_in_id IS NULL)
[2022-09-01 16:14:34] 3,039,851 rows affected in 30 s 56 ms
```

## No Filter on Block Slot Number

This is the SET of queries which is getting executed when there is no block slot number supplied to the snapshot.

```sql
CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (
    SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out
        LEFT OUTER JOIN tx_in ON
            tx_out.tx_id = tx_in.tx_out_id AND
            tx_out.index = tx_in.tx_out_index
        INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
   WHERE tx_in.tx_in_id IS NULL);

CREATE INDEX IF NOT EXISTS utxo_snapshot_stake_credential ON utxo_snapshot (stake_credential);

ANALYZE utxo_snapshot;
```

Speed:

This is a really long running set of operations:

For example, with a test set of data:

* `tx_out` = 13798161 rows.
* `tx_in` = 9642743 rows.

```text
cexplorer.public> CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (
                      SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out
                          LEFT OUTER JOIN tx_in ON
                              tx_out.tx_id = tx_in.tx_out_id AND
                              tx_out.index = tx_in.tx_out_index
                          INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
                     WHERE tx_in.tx_in_id IS NULL)
[2022-09-01 15:36:26] 3,039,851 rows affected in 10 s 329 ms
cexplorer.public> CREATE INDEX IF NOT EXISTS utxo_snapshot_stake_credential ON utxo_snapshot (stake_credential)
[2022-09-01 15:36:29] completed in 3 s 109 ms
cexplorer.public> ANALYZE utxo_snapshot
[2022-09-01 15:36:29] completed in 454 ms
```

## Explanation of the Query

1. For all block slot numbers less than (x), Create a copy of the `tx_out` table and add the linked `stake_address.hash_raw` and call it `stake_credential`.  Save this table as a temporary table of the name `tx_out_snapshot`

    ```sql
    CREATE TEMPORARY TABLE IF NOT EXISTS tx_out_snapshot AS (
        SELECT tx_out.*, stake_address.hash_raw AS stake_credential FROM tx_out
            INNER JOIN tx ON tx_out.tx_id = tx.id
            INNER JOIN block ON tx.block_id = block.id
            INNER JOIN stake_address ON stake_address.id = tx_out.stake_address_id
        WHERE block.slot_no <= 62510369);
    ```

    Sample Results:

    ```csv
    id, tx_id, index, address, address_raw, address_has_script, payment_cred, stake_address_id, value, data_hash, inline_datum_id, reference_script_id, stake_credential
    44666,20117,0,addr_test1qzjlptv5m75xhqsuaed8t0ekzmnwc0fykhgd66g9eyp7na5vr8svw5cndlfh777apttwyjm042x54vxtrnutyppekc7qj09qmd,0x00A5F0AD94DFA86B821CEE5A75BF3616E6EC3D24B5D0DD6905C903E9F68C19E0C753136FD37F7BDD0AD6E24B6FAA8D4AB0CB1CF8B20439B63C,false,0xA5F0AD94DFA86B821CEE5A75BF3616E6EC3D24B5D0DD6905C903E9F6,1,874836239,,,,0xE08C19E0C753136FD37F7BDD0AD6E24B6FAA8D4AB0CB1CF8B20439B63C
    44667,20118,0,addr_test1qqr585tvlc7ylnqvz8pyqwauzrdu0mxag3m7q56grgmgu7sxu2hyfhlkwuxupa9d5085eunq2qywy7hvmvej456flknswgndm3,0x000743D16CFE3C4FCC0C11C2403BBC10DBC7ECDD4477E053481A368E7A06E2AE44DFF6770DC0F4ADA3CF4CF2605008E27AECDB332AD349FDA7,false,0x0743D16CFE3C4FCC0C11C2403BBC10DBC7ECDD4477E053481A368E7A,2,45398559217,,,,0xE006E2AE44DFF6770DC0F4ADA3CF4CF2605008E27AECDB332AD349FDA7
    44668,20118,1,addr_test1qqr585tvlc7ylnqvz8pyqwauzrdu0mxag3m7q56grgmgu7sxu2hyfhlkwuxupa9d5085eunq2qywy7hvmvej456flknswgndm3,0x000743D16CFE3C4FCC0C11C2403BBC10DBC7ECDD4477E053481A368E7A06E2AE44DFF6770DC0F4ADA3CF4CF2605008E27AECDB332AD349FDA7,false,0x0743D16CFE3C4FCC0C11C2403BBC10DBC7ECDD4477E053481A368E7A,2,46649319060,,,,0xE006E2AE44DFF6770DC0F4ADA3CF4CF2605008E27AECDB332AD349FDA7
    44669,20118,2,addr_test1qqr585tvlc7ylnqvz8pyqwauzrdu0mxag3m7q56grgmgu7sxu2hyfhlkwuxupa9d5085eunq2qywy7hvmvej456flknswgndm3,0x000743D16CFE3C4FCC0C11C2403BBC10DBC7ECDD4477E053481A368E7A06E2AE44DFF6770DC0F4ADA3CF4CF2605008E27AECDB332AD349FDA7,false,0x0743D16CFE3C4FCC0C11C2403BBC10DBC7ECDD4477E053481A368E7A,2,43998107229,,,,0xE006E2AE44DFF6770DC0F4ADA3CF4CF2605008E27AECDB332AD349FDA7
    44670,20118,3,addr_test1qqr585tvlc7ylnqvz8pyqwauzrdu0mxag3m7q56grgmgu7sxu2hyfhlkwuxupa9d5085eunq2qywy7hvmvej456flknswgndm3,0x000743D16CFE3C4FCC0C11C2403BBC10DBC7ECDD4477E053481A368E7A06E2AE44DFF6770DC0F4ADA3CF4CF2605008E27AECDB332AD349FDA7,false,0x0743D16CFE3C4FCC0C11C2403BBC10DBC7ECDD4477E053481A368E7A,2,43998982940,,,,0xE006E2AE44DFF6770DC0F4ADA3CF4CF2605008E27AECDB332AD349FDA7

    ```

2. For all block slot numbers less than (x), Create a copy of the `tx_in` table.  Save this table as a temporary table of the name `tx_in_snapshot`

    ```sql
    CREATE TEMPORARY TABLE IF NOT EXISTS tx_in_snapshot AS (
        SELECT tx_in.* FROM tx_in
            INNER JOIN tx ON tx_in.tx_in_id = tx.id
            INNER JOIN block ON tx.block_id = block.id
        WHERE block.slot_no <= 62510369);
    ```

    Sample Results:

    ```csv
    id, tx_in_id, tx_out_id, tx_out_index, redeemer_id
    3887548,2356951,2356779,1,
    3886318,2356115,2128684,1,
    3886317,2356115,2352563,0,
    3886316,2356115,2112843,0,
    3885933,2355905,2348428,1,
    ```

3. Remove every record from the `tx_out_snapshot` table that has its `tx_id` listed as `tx_out_id` in the `tx_in_snapshot` table, and call the result the `utxo_snapshot` table.

    ```sql
    CREATE TEMPORARY TABLE IF NOT EXISTS utxo_snapshot AS (
        SELECT tx_out_snapshot.* FROM tx_out_snapshot
            LEFT OUTER JOIN tx_in_snapshot ON
                tx_out_snapshot.tx_id = tx_in_snapshot.tx_out_id AND
                tx_out_snapshot.index = tx_in_snapshot.tx_out_index
            WHERE tx_in_snapshot.tx_in_id IS NULL);
    ```

    Sample Results:

    ```csv
    id, tx_id, index, address, address_raw, address_has_script, payment_cred, stake_address_id, value, data_hash, inline_datum_id, reference_script_id, stake_credential
    44740,20148,0,addr_test1qz3guncpwkpv72ngvf8ymeze7n7jzyqepyjw9me3x9lqt6f0f2j4rqts4hk0jnxkdk87f44qddszukge6yzdrcr7kphq85zxz5,0x00A28E4F017582CF2A68624E4DE459F4FD2110190924E2EF31317E05E92F4AA5518170ADECF94CD66D8FE4D6A06B602E5919D104D1E07EB06E,false,0xA28E4F017582CF2A68624E4DE459F4FD2110190924E2EF31317E05E9,29,998831727,,,,0xE02F4AA5518170ADECF94CD66D8FE4D6A06B602E5919D104D1E07EB06E
    44744,20150,0,addr_test1qpgmrwlz43haws8xyt9jhn35e56yx0velyhx4cwykc5cmj60d8l47qertzu7g3mp85jhtxkxhy3t76ndl68cugxe220qfp30as,0x0051B1BBE2AC6FD740E622CB2BCE34CD34433D99F92E6AE1C4B6298DCB4F69FF5F032358B9E447613D25759AC6B922BF6A6DFE8F8E20D9529E,false,0x51B1BBE2AC6FD740E622CB2BCE34CD34433D99F92E6AE1C4B6298DCB,30,997825743,,,,0xE04F69FF5F032358B9E447613D25759AC6B922BF6A6DFE8F8E20D9529E
    44745,20151,0,addr_test1qra5me0r42s269yakeuecwtt0fa939t8zjr8hj3pdc3qw72azvwvckse0k42rmpt8q4d40hmdqguz5us2tjkhsgjr4xsj0c7xf,0x00FB4DE5E3AAA0AD149DB6799C396B7A7A58956714867BCA216E2207795D131CCC5A197DAAA1EC2B382ADABEFB6811C1539052E56BC1121D4D,false,0xFB4DE5E3AAA0AD149DB6799C396B7A7A58956714867BCA216E220779,32,1000000000,,,,0xE05D131CCC5A197DAAA1EC2B382ADABEFB6811C1539052E56BC1121D4D
    44755,20156,0,addr_test1qq2dmxr35ven98fw3wqkdvt0vhh7f0370lagm44536avnysls87c4lgn2quz22unl9h0kh4hqhauhn63c0aq02kkjl9sypw8ak,0x0014DD9871A333329D2E8B8166B16F65EFE4BE3E7FFA8DD6B48EBAC9921F81FD8AFD135038252B93F96EFB5EB705FBCBCF51C3FA07AAD697CB,false,0x14DD9871A333329D2E8B8166B16F65EFE4BE3E7FFA8DD6B48EBAC992,33,1000000000,,,,0xE01F81FD8AFD135038252B93F96EFB5EB705FBCBCF51C3FA07AAD697CB
    44779,20167,0,addr_test1qr2v7t870htvj9fuu55ea8ejk5lfaxxkv8yzg574egvyg8jalqp3rkt552mqj5cazf3cqef9w674zvh90r6d24xeylzspdnn7m,0x00D4CF2CFE7DD6C9153CE5299E9F32B53E9E98D661C82453D5CA18441E5DF80311D974A2B609531D126380652576BD5132E578F4D554D927C5,false,0xD4CF2CFE7DD6C9153CE5299E9F32B53E9E98D661C82453D5CA18441E,38,997825743,,,,0xE05DF80311D974A2B609531D126380652576BD5132E578F4D554D927C5

    ```

4. Create some indexes, and try and optimize future queries on these tables.

    ```sql
    CREATE INDEX IF NOT EXISTS utxo_snapshot_stake_credential ON utxo_snapshot(stake_credential);
    ANALYZE tx_out_snapshot;
    ANALYZE tx_in_snapshot;
    ANALYZE utxo_snapshot;
    ```

    This is a localized attempt at query optimization and has nothing to do with the vote power calculation itself.

## Summary

This query creates a temp table called `utxo_snapshot`  which is a copy of the `tx_out` table entries which do not have matching entries in the `tx_in` table, and adds the matching stake address from the original transaction to each row.

The simplified query does exactly the same thing, but it is simpler only because it does not consider the block slot number and produces its result from ALL data in the `tx_out` and `tx_in` tables.
