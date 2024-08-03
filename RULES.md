# Basic rules
* Everything should be async/event driven.
* Everything should be possible to run on multiple nodes.
* Everything should be possible to run on a single node.

## Dependencies
The project should use the following libaries, tools, techniques and protocols

* HTTP/3
* Rust
* Ratatui
* rust-libp2p

## The network
The network will consist initially of a permissionless and unenforced consensus network based on the PoSS algorithm. This will be implemented in the pioneerFS program.

Users running a storage node will be able to:
* Receive deals from clients
* Send and receive PIONEER tokens to and from clients and other storage providers, using a small test program running on an ERC20 testnet
