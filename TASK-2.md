# Implement a basic Geth based testnet
Implement a Geth testnet that makes an ERC20 contract that allows for the following:

Token name: PIONEER
* Purchase credits with which to pay SPs, both for storage and retrieval
* Same credit can be used to pay for different tasks:
    * download: Retrieve a file from one (sequential) or many (BitTorrent style) SPs
    * upload: Store a file to an SP
    * replication: Replicate a file from one SP to another
    * chain replication: Replicate a file to many SPs via one SP
    * replication testing: Paying for an audit retrieval bond (which is used to ensure that a set number of retrievals occur of your dataset per day, which are used to catch, slash and remove cheating SPs to prevent nodes from pretending to store data)
