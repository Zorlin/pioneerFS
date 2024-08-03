pub mod network;
pub mod storage_node;
pub mod client;
pub mod erc20;

pub use network::{Network, DebugLevel};
pub use storage_node::StorageNode;
pub use client::Client;



#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    #[test]
    fn test_network_creation() -> Result<(), String> {
        let network = Network::new()?;
        assert!(!network.storage_nodes().is_empty(), "Network should initialize with default storage nodes");
        assert!(!network.clients().is_empty(), "Network should initialize with default clients");
        Ok(())
    }

    #[test]
    fn test_add_storage_node() {
        let mut network = Network::new()?;
        let initial_count = network.storage_nodes().len();
        let peer_id = PeerId::random();
        network.add_storage_node(peer_id, 10); // Add a default price of 10
        assert_eq!(network.storage_nodes().len(), initial_count + 1);
        assert!(network.storage_nodes().contains_key(&peer_id));
    }

    #[test]
    fn test_add_client() {
        let mut network = Network::new()?;
        let initial_count = network.clients().len();
        let peer_id = PeerId::random();
        network.add_client(peer_id);
        assert_eq!(network.clients().len(), initial_count + 1);
        assert!(network.clients().contains_key(&peer_id));
    }

    #[test]
    fn test_file_upload_and_download() {
        let mut network = Network::new()?;
        let client_id = PeerId::random();
        network.add_client(client_id);
        
        // Add multiple storage nodes to ensure enough are available
        for _ in 0..5 {
            let storage_node_id = PeerId::random();
            network.add_storage_node(storage_node_id, 10); // Add a default price of 10
        }

        let filename = "test.txt".to_string();
        let data = b"Hello, world!".to_vec();

        // Check initial balance
        let initial_balance = network.get_balance(&client_id);
        assert_eq!(initial_balance, 1_000_000, "Initial balance should be 1,000,000");

        // Upload file
        network.upload_file(&client_id, filename.clone(), data.clone(), 3) // Using default replication factor of 3
            .unwrap_or_else(|e| panic!("Failed to upload file: {}", e));

        // Download file
        let downloaded_data = network.download_file(&client_id, &filename).unwrap();
        assert_eq!(data, downloaded_data);

        // Check that the client's balance is deducted
        let client_balance = network.get_balance(&client_id);
        assert!(client_balance < 1_000_000, "Balance should be less than initial balance");
        let expected_deduction = 3 * 10; // 3 replications * 10 tokens per GB (rounded up)
        assert_eq!(initial_balance - client_balance, expected_deduction, "Balance should be deducted by the correct amount");

        // Remove file
        assert!(network.remove_file(&client_id, &filename).is_ok());

        // Try to download removed file
        assert!(network.download_file(&client_id, &filename).is_err());
    }

    #[test]
    fn test_file_replication() {
        let mut network = Network::new()?;
        let client_id = PeerId::random();
        network.add_client(client_id);
        
        // Add multiple storage nodes to ensure enough are available for replication
        for _ in 0..5 {
            let storage_node_id = PeerId::random();
            network.add_storage_node(storage_node_id, 10); // Add a default price of 10
        }

        let filename = "replicated.txt".to_string();
        let data = b"Replicate me!".to_vec();

        // Check initial balance
        let initial_balance = network.get_balance(&client_id);
        assert_eq!(initial_balance, 1_000_000, "Initial balance should be 1,000,000");

        // Upload file
        network.upload_file(&client_id, filename.clone(), data.clone(), 3)
            .unwrap_or_else(|e| panic!("Failed to upload file: {}", e));

        // Replicate file
        assert!(network.replicate_file(&client_id, &filename, 2).is_ok());

        // Download file
        let downloaded_data = network.download_file(&client_id, &filename).unwrap();
        assert_eq!(data, downloaded_data);

        // Check that the client's balance is deducted
        let client_balance = network.get_balance(&client_id);
        assert!(client_balance < 1_000_000, "Balance should be less than initial balance");
        assert!(client_balance >= 999_970, "Balance should not be deducted by more than 30 tokens");
    }
}
