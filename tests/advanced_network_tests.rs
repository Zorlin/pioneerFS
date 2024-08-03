use pioneerFS::network::Network;
use pioneerFS::client::Client;
use pioneerFS::storage_node::StorageNode;
use libp2p::PeerId;

#[cfg(test)]
mod advanced_network_tests {
    use super::*;

    fn setup() -> Network {
        let mut network = Network::new();
        // Add some clients and storage nodes
        network.add_client(PeerId::random());
        network.add_client(PeerId::random());
        network.add_storage_node(PeerId::random(), 10); // 10 tokens per GB
        network.add_storage_node(PeerId::random(), 15); // 15 tokens per GB
        network
    }

    #[test]
    fn test_file_replication() {
        let mut network = setup();
        let client_id = network.list_clients()[0];
        let filename = "test_file.txt";
        let data = vec![1, 2, 3, 4, 5]; // 5 bytes of data

        // Upload file with replication factor 2
        network.upload_file(&client_id, filename.to_string(), data.clone(), 2).unwrap();

        // Check if file is stored on two different storage nodes
        let client = network.clients().get(&client_id).unwrap();
        let storage_nodes = client.get_file_locations(filename).unwrap();
        assert_eq!(storage_nodes.len(), 2, "File should be replicated on two storage nodes");

        // Verify file content on both storage nodes
        for &node_id in storage_nodes {
            let node = network.storage_nodes().get(&node_id).unwrap();
            assert_eq!(node.get_file(filename).unwrap(), &data, "File content mismatch on storage node");
        }
    }

    #[test]
    fn test_storage_node_failure() {
        let mut network = setup();
        let client_id = network.list_clients()[0];
        let filename = "important_file.txt";
        let data = vec![10, 20, 30, 40, 50]; // 5 bytes of data

        // Upload file with replication factor 2
        network.upload_file(&client_id, filename.to_string(), data.clone(), 2).unwrap();

        // Simulate failure of one storage node
        let client = network.clients().get(&client_id).unwrap();
        let storage_nodes = client.get_file_locations(filename).unwrap();
        let failed_node_id = storage_nodes[0];
        network.storage_nodes.remove(&failed_node_id);

        // Attempt to download the file
        let downloaded_data = network.download_file(&client_id, filename).unwrap();
        assert_eq!(downloaded_data, data, "Downloaded data should match original data even after node failure");
    }

    #[test]
    fn test_dynamic_replication() {
        let mut network = setup();
        let client_id = network.list_clients()[0];
        let filename = "growing_file.txt";
        let initial_data = vec![1, 2, 3]; // 3 bytes of data

        // Upload file with initial replication factor 1
        network.upload_file(&client_id, filename.to_string(), initial_data.clone(), 1).unwrap();

        // Check initial replication
        let client = network.clients().get(&client_id).unwrap();
        let initial_storage_nodes = client.get_file_locations(filename).unwrap();
        assert_eq!(initial_storage_nodes.len(), 1, "Initial replication factor should be 1");

        // Request higher replication factor
        network.request_higher_replication(&client_id, filename, 2).unwrap();

        // Check updated replication
        let client = network.clients().get(&client_id).unwrap();
        let updated_storage_nodes = client.get_file_locations(filename).unwrap();
        assert_eq!(updated_storage_nodes.len(), 2, "Updated replication factor should be 2");

        // Verify file content on both storage nodes
        for &node_id in updated_storage_nodes {
            let node = network.storage_nodes().get(&node_id).unwrap();
            assert_eq!(node.get_file(filename).unwrap(), &initial_data, "File content mismatch after replication");
        }
    }
}
