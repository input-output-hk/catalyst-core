# Suggested optimized queries

WIP NOT COMPLETE

The query is complex and requires a lot of suboptimal joins and table scans.

In the following queries, we use the `block.slot_no` to divide up the problem space.
This would allow us to run multiple queries in parallel, which postgres can not do if this is a single query.
We should have a parameter which defines how much we break up the query.
We can then execute the set of queries asynchronously, and return data from the combined async results, as if it was 1 query.

This is the proposed solution:

Basic Query 1 (Get all records from TX Out):

    ```sql
        SELECT * FROM tx_out
                INNER JOIN tx ON tx_out.tx_id = tx.id
                INNER JOIN block ON tx.block_id = block.id
            WHERE block.slot_no <= 62510369 AND block.slot_no > 31255184;
    ```

    Parallelize using non overlapping bands of `block.slot_no`
    If the max block.slot_no is not specified, it should be queried, which would be very quick.

Basic Query 2 (Get all stake address hashes):
    ```sql
        SELECT id, hash_raw FROM stake_address
        WHERE id <= (max) and id >= min;

    ```

    This query can also be parallelized, but we would need to know what the MAX
    `id` is, which is itself a very quick query.

Basic Query 3 (Get all records from tx_in, but only selected data.)

    ```sql
        SELECT tx_out_id, tx_out_index FROM tx_in
                INNER JOIN tx ON tx_in.tx_in_id = tx.id
                INNER JOIN block ON tx.block_id = block.id
            WHERE block.slot_no <= 62510369 AND block.slot_no > 31255184;
    ```

    Parallelize using non overlapping bands of `block.slot_no`
    If the max block.slot_no is not specified, it should be queried, which would be very quick.

Basic Query 4:

    ```sql

    ```
