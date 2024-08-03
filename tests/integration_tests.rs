use libp2p::PeerId;
use pioneerfs::Network;

#[test]
fn test_network_operations() {
    let mut network = Network::new();

    // Add clients and storage nodes
    let client1_id = PeerId::random();
    let client2_id = PeerId::random();
    let storage_node1_id = PeerId::random();
    let storage_node2_id = PeerId::random();

    network.add_client(client1_id);
    network.add_client(client2_id);
    network.add_storage_node(storage_node1_id);
    network.add_storage_node(storage_node2_id);

    // Upload a file
    let filename = "test.txt".to_string();
    let data = b"Hello, Pioneer!".to_vec();
    assert!(network.upload_file(&client1_id, &storage_node1_id, filename.clone(), data.clone()).is_ok());

    // Download the file
    let downloaded_data = network.download_file(&client1_id, &storage_node1_id, &filename).unwrap();
    assert_eq!(data, downloaded_data);

    // Replicate the file
    assert!(network.replicate_file(&storage_node1_id, &storage_node2_id, &filename).is_ok());

    // Download the replicated file
    let replicated_data = network.download_file(&client2_id, &storage_node2_id, &filename).unwrap();
    assert_eq!(data, replicated_data);

    // Remove the file from the first storage node
    assert!(network.remove_file(&client1_id, &storage_node1_id, &filename).is_ok());

    // Verify the file is still available on the second storage node
    let still_available_data = network.download_file(&client2_id, &storage_node2_id, &filename).unwrap();
    assert_eq!(data, still_available_data);
}

#[test]
fn test_balance_operations() {
    let mut network = Network::new();

    let client_id = PeerId::random();
    let storage_node_id = PeerId::random();

    network.add_client(client_id);
    network.add_storage_node(storage_node_id);

    let filename = "balance_test.txt".to_string();
    let data = b"Test balance operations".to_vec();
    let initial_balance = 100000;
    let upload_cost = (data.len() as u64) * 1; // 1 PIO per byte

    // Upload a file
    assert!(network.upload_file(&client_id, &storage_node_id, filename.clone(), data.clone()).is_ok());

    // Check balances after upload
    let client = network.clients().get(&client_id).unwrap();
    let storage_node = network.storage_nodes().get(&storage_node_id).unwrap();

    assert_eq!(client.balance(), initial_balance - upload_cost);
    assert_eq!(storage_node.balance(), upload_cost);

    // Try to upload a file with insufficient balance
    let large_data = vec![0; 200000]; // 200KB file
    assert!(network.upload_file(&client_id, &storage_node_id, "large_file.txt".to_string(), large_data).is_err());

    // Check balances haven't changed after failed upload
    let client = network.clients().get(&client_id).unwrap();
    let storage_node = network.storage_nodes().get(&storage_node_id).unwrap();

    assert_eq!(client.balance(), initial_balance - upload_cost);
    assert_eq!(storage_node.balance(), upload_cost);
}
