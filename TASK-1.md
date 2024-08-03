# Background
You are tasked with creating a very basic implementation of the Pioneer Proof of Storage System - PoSS2.

The network will consist initially of a permissionless and unenforced consensus network based on the PoSS algorithm. This will be implemented in the pioneerFS program.

You will consume the contents of peerbit-examples, to take inspiration from. You are forbidden from and deeply warned away from changing the contents of peerbits-examples.

In the system:

Users running a storage node will be able to:
* Receive deals from clients
* Send and receive UNIT tokens to and from clients and other storage providers

For now, this will take the form of conversations that actors play back and forth. You can "listen in" on the conversations with an intuitive Web User Interface that displays a global map of the network using Tauri
Selecting a node allows you to listen to its conversations that it has outgoing, and to incoming conversations with it.

The actors consist of the following:

* Storage nodes, who are paid PIO to receive files from clients and store them. If paid by a client to replicate their file, will find a number of other storage nodes to replicate the file to.
    * They will then replicate the file to the other SPs - even as far as replicating it to them WHILE the user is uploading the file to it.
    * The client essentially pays for the SP to pay other SPs to replicate their file for them - either
        * depositing the full amount of PIO needed to achieve desired replication, and then allowing the person operating the client to leave the job running and check on it later, leave the network and come back, etc
        * or simply streaming the file out to as many SPs as the client wanted and spending the tokens as needed to achieve that process
* Clients, who pay PIO to store and retrieve and replicate files, and who pay SPs to verify that their files exist out there on multiple other SPs. Clients can set a replication level of 3 for a file and pay for it (this might cost for example $3/TB times 3 copies - so let's call it $9/TB or 9000 PIO - in this testnet PIO has a fixed equivalent price of $0.0001)
* Storage nodes must be able to receive files from both clients and other storage nodes, and must be able to send files to other storage nodes.
* All nodes connect to each other with Peerbit, forming a mesh with the best feasible connectivity

# Task 1: Basics

Implement a really basic system based on the libraries mentioned, focusing mostly on what you were just told as "Background".

You should be able to run the system and see the following:

* Clients uploading files
* Nodes replicating files to each other
* Clients requesting higher replication levels
* Nodes bidding on the ability to store those files and clients setting the maximum price they are willing to pay for replications

With the TUI/WebUI You can view the various nodes as a text list as if they were a list of long running tasks, or view them as a global map with virtual locations around the world.

# Task 2: TUI/WebUI
Ensure there is a TUI but separately also a valid WebUI