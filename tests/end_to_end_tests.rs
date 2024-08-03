use pioneerfs::{Network, DebugLevel};
use libp2p::PeerId;
use rand::Rng;

#[test]
fn test_end_to_end_storage() {
    let mut network = Network::new();
    network.set_debug_level(DebugLevel::Low);

    // Spawn 10 nodes
    for _ in 0..10 {
        let sp_id = PeerId::random();
        network.add_storage_node(sp_id, rand::thread_rng().gen_range(10..20));
    }

    let client_id = PeerId::random();
    network.add_client(client_id);

    let filename = "end_to_end_test_file.txt".to_string();
    let data = b"End-to-end test data".to_vec();

    // Test replication on 3, 5, 7, 9, and 10 nodes
    for &replication_factor in &[3, 5, 7, 9, 10] {
        network.upload_file(&client_id, filename.clone(), data.clone(), replication_factor)
            .unwrap_or_else(|e| panic!("Failed to upload file with replication factor {}: {}", replication_factor, e));

        let locations = network.get_file_locations(&client_id, &filename).unwrap();
        assert_eq!(locations.len(), replication_factor, "File should be replicated on {} nodes", replication_factor);
    }
}
#[test]
fn test_poss_retrieval() {
    let mut network = Network::new();
    network.set_debug_level(DebugLevel::Low);

    // Spawn 10 nodes
    for _ in 0..10 {
        let sp_id = PeerId::random();
        network.add_storage_node(sp_id, rand::thread_rng().gen_range(10..20));
    }

    let client_id = PeerId::random();
    network.add_client(client_id);

    let filename = "poss_retrieval_test_file.txt".to_string();
    let data = b"PoSS retrieval test data".to_vec();
    let replication_factor = 5;

    network.upload_file(&client_id, filename.clone(), data.clone(), replication_factor)
        .unwrap_or_else(|e| panic!("Failed to upload file: {}", e));

    // Retrieve the file using PoSS-style retrieval
    let retrieved_data = network.download_file(&client_id, &filename).unwrap();
    assert_eq!(data, retrieved_data, "Retrieved data should match the original data");
}
#[test]
fn test_split_retrieval() {
    let mut network = Network::new();
    network.set_debug_level(DebugLevel::Low);

    // Spawn 10 nodes
    for _ in 0..10 {
        let sp_id = PeerId::random();
        network.add_storage_node(sp_id, rand::thread_rng().gen_range(10..20));
    }

    let client_id = PeerId::random();
    network.add_client(client_id);

    let filename = "split_retrieval_test_file.txt".to_string();
    let data = b"Split retrieval test data".to_vec();
    let replication_factor = 5;

    network.upload_file(&client_id, filename.clone(), data.clone(), replication_factor)
        .unwrap_or_else(|e| panic!("Failed to upload file: {}", e));

    // Retrieve the file using split retrieval
    let locations = network.get_file_locations(&client_id, &filename).unwrap();
    let mut retrieved_data = Vec::new();

    for node_id in locations {
        if let Ok(part) = network.get_file_content(&node_id, &filename) {
            retrieved_data.extend(part);
        }
    }

    assert_eq!(data, retrieved_data, "Retrieved data should match the original data");
}
use pioneerfs::{Network, DebugLevel};
use libp2p::PeerId;
use rand::Rng;

#[test]
fn test_end_to_end_storage() {
    let mut network = Network::new();
    network.set_debug_level(DebugLevel::Low);

    // Spawn 10 nodes
    for _ in 0..10 {
        let sp_id = PeerId::random();
        network.add_storage_node(sp_id, rand::thread_rng().gen_range(10..20));
    }

    let client_id = PeerId::random();
    network.add_client(client_id);

    let filename = "end_to_end_test_file.txt".to_string();
    let data = b"End-to-end test data".to_vec();

    // Test replication on 3, 5, 7, 9, and 10 nodes
    for &replication_factor in &[3, 5, 7, 9, 10] {
        network.upload_file(&client_id, filename.clone(), data.clone(), replication_factor)
            .unwrap_or_else(|e| panic!("Failed to upload file with replication factor {}: {}", replication_factor, e));

        let locations = network.get_file_locations(&client_id, &filename).unwrap();
        assert_eq!(locations.len(), replication_factor, "File should be replicated on {} nodes", replication_factor);
    }
}

#[test]
fn test_poss_retrieval() {
    let mut network = Network::new();
    network.set_debug_level(DebugLevel::Low);

    // Spawn 10 nodes
    for _ in 0..10 {
        let sp_id = PeerId::random();
        network.add_storage_node(sp_id, rand::thread_rng().gen_range(10..20));
    }

    let client_id = PeerId::random();
    network.add_client(client_id);

    let filename = "poss_retrieval_test_file.txt".to_string();
    let data = b"PoSS retrieval test data".to_vec();
    let replication_factor = 5;

    network.upload_file(&client_id, filename.clone(), data.clone(), replication_factor)
        .unwrap_or_else(|e| panic!("Failed to upload file: {}", e));

    // Retrieve the file using PoSS-style retrieval
    let retrieved_data = network.download_file(&client_id, &filename).unwrap();
    assert_eq!(data, retrieved_data, "Retrieved data should match the original data");
}

#[test]
fn test_split_retrieval() {
    let mut network = Network::new();
    network.set_debug_level(DebugLevel::Low);

    // Spawn 10 nodes
    for _ in 0..10 {
        let sp_id = PeerId::random();
        network.add_storage_node(sp_id, rand::thread_rng().gen_range(10..20));
    }

    let client_id = PeerId::random();
    network.add_client(client_id);

    let filename = "split_retrieval_test_file.txt".to_string();
    let data = b"Split retrieval test data".to_vec();
    let replication_factor = 5;

    network.upload_file(&client_id, filename.clone(), data.clone(), replication_factor)
        .unwrap_or_else(|e| panic!("Failed to upload file: {}", e));

    // Retrieve the file using split retrieval
    let locations = network.get_file_locations(&client_id, &filename).unwrap();
    let mut retrieved_data = Vec::new();

    for node_id in locations {
        if let Ok(part) = network.get_file_content(&node_id, &filename) {
            retrieved_data.extend(part);
        }
    }

    assert_eq!(data, retrieved_data, "Retrieved data should match the original data");
}
