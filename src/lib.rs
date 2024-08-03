mod network;
mod storage_node;
mod client;
mod erc20;

pub use network::Network;
pub use storage_node::StorageNode;
pub use client::Client;


#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    #[test]
    fn test_network_creation() {
        let network = Network::new();
        assert!(network.storage_nodes().is_empty());
        assert!(network.clients().is_empty());
    }

    #[test]
    fn test_add_storage_node() {
        let mut network = Network::new();
        let peer_id = PeerId::random();
        network.add_storage_node(peer_id, 10); // Add a default price of 10
        assert_eq!(network.storage_nodes().len(), 1);
        assert!(network.storage_nodes().contains_key(&peer_id));
    }

    #[test]
    fn test_add_client() {
        let mut network = Network::new();
        let peer_id = PeerId::random();
        network.add_client(peer_id);
        assert_eq!(network.clients().len(), 1);
        assert!(network.clients().contains_key(&peer_id));
    }

    #[test]
    fn test_file_upload_and_download() {
        let mut network = Network::new();
        let client_id = PeerId::random();
        let storage_node_id = PeerId::random();
        network.add_client(client_id);
        network.add_storage_node(storage_node_id, 10); // Add a default price of 10

        let filename = "test.txt".to_string();
        let data = b"Hello, world!".to_vec();

        // Upload file
        assert!(network.upload_file(&client_id, &storage_node_id, filename.clone(), data.clone()).is_ok());

        // Download file
        let downloaded_data = network.download_file(&client_id, &storage_node_id, &filename).unwrap();
        assert_eq!(data, downloaded_data);

        // Check that the client's balance is deducted
        let client_balance = network.get_balance(&client_id);
        assert!(client_balance < 100000);

        // Remove file
        assert!(network.remove_file(&client_id, &storage_node_id, &filename).is_ok());

        // Try to download removed file
        assert!(network.download_file(&client_id, &storage_node_id, &filename).is_err());
    }

    #[test]
    fn test_file_replication() {
        let mut network = Network::new();
        let client_id = PeerId::random();
        let storage_node_id1 = PeerId::random();
        let storage_node_id2 = PeerId::random();
        network.add_client(client_id);
        network.add_storage_node(storage_node_id1, 10); // Add a default price of 10
        network.add_storage_node(storage_node_id2, 10); // Add a default price of 10

        let filename = "replicated.txt".to_string();
        let data = b"Replicate me!".to_vec();

        // Upload file to first storage node
        assert!(network.upload_file(&client_id, &storage_node_id1, filename.clone(), data.clone()).is_ok());

        // Replicate file to second storage node
        assert!(network.replicate_file(&storage_node_id1, &storage_node_id2, &filename).is_ok());

        // Download file from second storage node
        let downloaded_data = network.download_file(&client_id, &storage_node_id2, &filename).unwrap();
        assert_eq!(data, downloaded_data);
    }
}
