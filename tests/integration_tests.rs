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
    network.add_storage_node(storage_node1_id, 10); // 10 PIO per GB
    network.add_storage_node(storage_node2_id, 12); // 12 PIO per GB

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

