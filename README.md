# Decentralized Storage Network

This project implements a decentralized storage network using libp2p in Rust. It simulates a network of storage nodes and clients, allowing file storage, retrieval, and management.

## Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

## Getting Started

1. Clone the repository:
   ```
   git clone https://github.com/your-username/decentralized-storage-network.git
   cd decentralized-storage-network
   ```

2. Build the project:
   ```
   cargo build
   ```

3. Run the tests:
   ```
   cargo test
   ```

## Usage

### Starting the Testnet

To start the testnet and interact with it:

1. Ensure you have the necessary dependencies installed:
   ```
   cargo install --path .
   ```

2. Start the testnet:
   ```
   cargo run --bin testnet
   ```

3. The testnet will initialize and you'll see output in the console showing the creation of storage nodes and clients.

### Deploying Smart Contracts

Before interacting with the network, you need to deploy the necessary smart contracts:

1. Install the Solidity compiler if you haven't already:
   ```
   npm install -g solc
   ```

2. Compile the smart contracts:
   ```
   solc --bin --abi contracts/*.sol -o build
   ```

3. Deploy the contracts to the testnet:
   ```
   cargo run --bin deploy_contracts
   ```

   This will deploy the contracts and output their addresses. Make note of these addresses as you'll need them for interacting with the network.

### Interacting with the Network

Once the testnet is running and contracts are deployed:

1. Interact with the network:
   - To store a file: Enter `store <client_id> <storage_node_id> <filename> <file_content>`
   - To retrieve a file: Enter `retrieve <client_id> <storage_node_id> <filename>`
   - To remove a file: Enter `remove <client_id> <storage_node_id> <filename>`
   - To check a client's balance: Enter `balance client <client_id>`
   - To check a storage node's balance: Enter `balance node <storage_node_id>`
   - To exit the program: Enter `exit`

   Example:
   ```
   store 1 2 myfile.txt This is the content of my file
   retrieve 1 2 myfile.txt
   balance client 1
   ```

2. The program will provide feedback for each action, showing success messages or errors.

### Stopping the Testnet

To stop the testnet, simply press `Ctrl+C` in the terminal where it's running.

## Project Structure

- `src/client.rs`: Defines the `Client` struct and its methods.
- `src/storage_node.rs`: Defines the `StorageNode` struct and its methods.
- `src/network.rs`: Implements the `Network` struct, managing interactions between clients and storage nodes.
- `src/lib.rs`: Main library file that ties everything together.

## Features

- Create and manage storage nodes and clients
- Store and retrieve files in the network
- Manage client and storage node balances
- Simulate network operations

## Troubleshooting

- If you encounter any "peer not found" errors, make sure you're using valid client and storage node IDs.
- If a file operation fails, check that the client has sufficient balance and the storage node has available space.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
