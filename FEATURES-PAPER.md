The network will comprise of N number of individuals, running storage nodes. Each storage node has a minimum of 1000 PIONEER tokens (an ERC20 running on Metis) stored and staked.

A client comes to the network and wants to store data. They know they want to store their data reliably for 540 days, and that they want to be able to retrieve that data 100 times per minute. They want, therefore, a service level agreement of at least one successful retrieval per minute, reasoning that if there is at least one, the other 99 should usually succeed. They pay upfront for the expected bandwidth this represents.

They initiate a write to Node Atlas, which begins storing the data on its local storage. The client also pays for Node Barcelona to store the data and starts storing data to it too. (In future, replication directly between nodes would make a good enhancement)

For each round of retrieval proof the client wants to perform, a handshake takes place.

RANDAO is used at a particular height to select a random scalar. All three parties - the client, Barcelona and Atlas - will agree upon the scalar, or rather on the block height from which it is extracted. Or, put another way, they will all abide by a particular height determined by a smart contract.

Each participant does an ECMUL, multiplying their public key by the random scalar selected by the RANDAO process. This gives them their personal private key for this round of proofs.

The participant is then required to do an ECMUL operation against the actual chunk of data - multiplying the data by their personal private key - and record the hash of the result. They sign the hash, which can be calculated by anybody with the scalar + pubkey - with their node private key, which only they can do. 

In doing so, they are able to prove that their keypair was involved in calculating the values and attesting to them. The stake of PIONEER tokens the node is required to hold discourages them from sharing their private key with another party to have them perform this attestation for them.

The process can be repeated to prove multiple chunks at once, depending on what settings the client selected for their SLA.

A smart contract enforces that each participant who claims to be storing data in a retrievable way is doing so, by demanding they come up with a signed attestation for each proof window as they come.

If the client has demanded 1440 proofs per day, the SPs will submit a recursive proof once per hour which records the results of each of their commitments + any audits they have performed.
The Audit Process
Serving a CID with Range Requests
When a client makes a range request, they're asking for a specific portion of the file. This request is mapped to the corresponding blocks within the IPFS network.

Selecting a Target
We (the auditor) take the list of potential peers, which is composed of all nodes with an active stake of PIONEER who are "in the contract". We order them by their public key/peer ID. We then take the hash of (ECMUL [SP's private key x RANDAO value]) and perform a modulo operation of that hash against the number of peers to end up with a target node. The smart contract will enforce that we perform this math exactly as required.

Request
We (the auditor) request the data from our target, with the range we're looking for determined by the RANDAO value.

Storage Provider (SP) Submission
Merkle Inclusion Proof: As the SP serves the requested data, they also provide a Merkle inclusion proof. This proof demonstrates that the specific blocks of data being served are indeed part of the larger file represented by the CID.

Cryptographic Transformation: Alongside the Merkle inclusion proof, the SP performs the cryptographic folding scheme, which includes:

Elliptic Curve Multiplication (ECMUL) using a RANDAO-generated scalar.
Keyed hash operations involving the SP's private key and the RANDAO value.

These operations are performed on the actual data blocks within the range request (not a hash which could be passed around separately).

Verification by a Separate Node (Auditor)
Checking the Merkle Inclusion Proof: The verifier (a separate node acting as an auditor) first checks the Merkle inclusion proof provided by the SP. This step validates that the data blocks served are indeed part of the file corresponding to the CID.

Validating the Cryptographic Proof: After confirming the Merkle inclusion proof, the verifier checks the cryptographic proofs (ECMUL results and keyed hashes) to ensure they have been correctly computed and are tied to the SP's identity and the RANDAO value.

Submitting an Audit Result: Once the Merkle inclusion proof and the cryptographic proofs are validated, the verifier submits an audit result. This result indicates whether the SP has correctly and reliably served the requested data, adhering to the integrity and authenticity standards of the network.


### In another contributor's words
We use a randomness beacon (RANDAO) on Ethereum to determine
What range of data we read
What nodes an auditor node is allowed to audit
What transformation we apply to data when proving we hold it right now.

Client stores data on SP A, and commits funds to SP A associated with serving the file. As part of this the CID of the file is recorded. Once SP A completes the transfer, it does a proof to Client using the latest RANDAO value and elliptic curve multiplication (ECMUL) using its keys and the data. The client validates this proof and then begins the contract.

Any SP in the network may now, at random chance each block (We take the list of potential peers, which is composed of all nodes with an active stake of PIONEER who are "in the contract". We order them by their public key/peer ID. We then take the hash of (ECMUL [SP's private key x RANDAO value]) and perform a modulo operation of that hash against the number of peers to end up with a target node. The smart contract will enforce that we performed this math exactly as required.), have the opportunity to become an auditor and interrogate SP A. If this happens, SP A must produce a proof that shows that it has the data requested by the Auditing SP, and the auditing SP is tasked with downloading the relevant data blocks from the SP (disguised as a regular retrieval), validating the associated proof and either signing off on it or rejecting it.

The proof will be a folding scheme. This will consist first of an inclusion proof which the SP comes up with to prove that the data it's reading back from disk consists of valid blocks within the IPFS CID that it is being asked to serve. Then, an ECMUL is performed using the scalar selected by RANDAO - against the original blocks of data, as well as keyed hash operations involving the SP's private key and the RANDAO value.

These operations are performed on the actual data blocks within the range request (not a hash which could be passed around separately), to assert that the actual data is being stored and not just hashes thereof.

The auditing SP submits its proofs at the top of the hour along with the SP being checked, and their results are compared by the contract after a delay.

For each successful audit the auditing SP submits (ie: indicating a correct piece of data), the auditing SP is rewarded. Audits that result in slashing for an SP (indicating the SP is misbehaving) slash the target SP and also reward the auditing SP.

At the end of each Epoch (4/8/12/24 hours, TBD) the contract is finalised and sends the finalised funds to whoever is appropriate. In the case of perfect behaviour by the client and SP, half of the funds are sent back to the client and half to the SP who they worked with (with portions of that latter half going towards auditors). In the case of imperfect behaviour, slashing may occur which reduces the profit of either the client or the SP or both. In rare cases of extreme misbehaviour, stakes may be eroded and funds lost to burning.

In the event of an SP failing numerous checks in a row (or some overall percentage of checks), the job will be cancelled and the remaining funds returned to the client. In the event of the client failing to lock a job (by attesting that the SPs they sent data to, have successfully stored the data), the same will occur.