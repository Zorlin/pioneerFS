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
    let storage_node3_id = PeerId::random();

    network.add_client(client1_id);
    network.add_client(client2_id);
    network.add_storage_node(storage_node1_id, 10); // 10 PIO per GB
    network.add_storage_node(storage_node2_id, 12); // 12 PIO per GB
    network.add_storage_node(storage_node3_id, 11); // 11 PIO per GB

    // Upload a file
    let filename = "test.txt".to_string();
    let data = b"Hello, Pioneer! This is a test file for erasure coding.".to_vec();
    assert!(network.upload_file(&client1_id, filename.clone(), data.clone(), 3).is_ok());

    // Download the file
    let downloaded_data = network.download_file(&client1_id, &filename).unwrap();
    assert_eq!(data, downloaded_data);

    // Check if the file is stored across multiple nodes
    let storage_nodes = {
        let client = network.clients().get(&client1_id).unwrap();
        client.get_file_locations(&filename).unwrap().clone()
    };
    assert_eq!(storage_nodes.len(), 3); // Assuming REPLICATION_FACTOR is 3

    // Remove one storage node and try to download again
    network.storage_nodes.remove(&storage_nodes[0]);
    let downloaded_data = network.download_file(&client1_id, &filename).unwrap();
    assert_eq!(data, downloaded_data);

    // Check deals
    assert_eq!(network.deals.len(), 3); // Assuming REPLICATION_FACTOR is 3

    // Check balances
    for &node_id in &storage_nodes {
        assert!(network.get_balance(&node_id) > 0);
    }
    assert!(network.get_balance(&client1_id) < 1_000_000);
}

#[test]
fn test_marketplace() {
    let mut network = Network::new();

    let client_id = PeerId::random();
    let sp_id = PeerId::random();

    network.add_client(client_id);
    network.add_storage_node(sp_id, 10);

    // Add a storage offer
    network.add_storage_offer(sp_id, 10, 1_000_000_000);

    // List storage offers
    let offers = network.get_storage_offers();
    assert_eq!(offers.len(), 1);

    // Accept storage offer
    assert!(network.accept_storage_offer(&client_id, 0, 500_000_000).is_ok());

    // Check if the offer was updated
    let updated_offers = network.get_storage_offers();
    assert_eq!(updated_offers.len(), 1);
    assert_eq!(updated_offers[0].available_space, 500_000_000);
}

