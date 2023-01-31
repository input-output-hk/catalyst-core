# Phase 1

In this initial implementation phase, we build the core foundational pieces of the service.

## Blockchain Interface

This is a blockchain reader task that takes the following parameters:

1. Starting block #
2. Minimum distance from blockchain head
3. Ending Block# (Optional, defaults to head)
4. A queue to store blocks

Reads as fast as possible from the starting block number and:

* follows the blockchain but gets no closer than the minimum distance.
* stops when it gets to the last block (if specified)

This task should be able to run independently once started, and post blocks to its queue until killed or it reaches its final block.

## Basic consumer

Reads from the block queue and prints out each block number as it arrives.

## Test CLI

Rough CLI which allows the parameters to be specified.

## Result

A very simple chain follower which:

1. does not parse or validate blocks
2. but reliably can read sequential blocks from the Cardano network at its maximum rate.

This can be used to test how fast a full chain sync could take in its simplest form.
