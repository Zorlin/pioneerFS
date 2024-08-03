Implement an end-to-end storage test suite which spawns 10 nodes, uploads data to one and ensures it's replicated on 3, 5, 7, 9 and 10 nodes (ie: five different tests)

Implement a retrieval test that uses PoSS-style retrieval - see FEATURES-PAPER.md for details! - to retrieve a file from the network.

Implement a split retrieval test that pulls pieces of that file from all nodes that have it.