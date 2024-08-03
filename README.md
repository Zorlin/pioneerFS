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
   cargo run
   ```

3. The testnet will initialize and you'll see a terminal user interface (TUI) showing the network status and available commands.

### Interacting with the Network

Once the testnet is running:

1. Use the TUI to interact with the network. Available commands include:
   - `add_storage_node <price_per_gb>`: Add a new storage node with the specified price per GB
   - `add_client`: Add a new client to the network
   - `upload_file <client_id> <filename> <file_content>`: Upload a file to the network
   - `download_file <client_id> <filename>`: Download a file from the network
   - `remove_file <client_id> <filename>`: Remove a file from the network
   - `list_files <client_id>`: List files stored by a client
   - `get_balance <peer_id>`: Check the balance of a client or storage node
   - `list_storage_offers`: View available storage offers in the marketplace
   - `accept_storage_offer <client_id> <offer_index> <file_size>`: Accept a storage offer

2. Enter commands in the input field at the bottom of the TUI.

3. The program will provide feedback for each action in the message area of the TUI.

### Stopping the Testnet

To stop the testnet, press `q` or `Ctrl+C` in the terminal where it's running.

### Smart Contracts

The project uses built-in Rust implementations to simulate smart contract functionality. There's no need for external smart contract deployment.

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
