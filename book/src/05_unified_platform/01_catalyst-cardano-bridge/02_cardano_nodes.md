# Cardano Nodes

The bridge will need at least 1, and preferably more Cardano Nodes to read blocks from.

The Bridge will employ a local consensus model, in place of the absolute trust of a single node.
Part of the configuration of the bridge will need to be:

* the addresses of the available nodes that may be requested for new blocks.
* the number of nodes which must send concurring blocks before a block is accepted.
* the number of blocks to retrieve in advance of the current head.
