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

To run the program and interact with it:

1. Start the program:
   ```
   cargo run
   ```

2. The program will start a simulation of the decentralized storage network. You'll see output in the console showing the creation of storage nodes and clients.

3. Interact with the network:
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

4. The program will provide feedback for each action, showing success messages or errors.

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
