use crate::{StorageNode, Client};
use libp2p::PeerId;
use std::collections::HashMap;

const STORAGE_COST_PER_BYTE: u64 = 1; // 1 PIO per byte
const REPLICATION_COST_PER_BYTE: u64 = 1; // 1 PIO per byte for replication

pub struct Network {
    storage_nodes: HashMap<PeerId, StorageNode>,
    clients: HashMap<PeerId, Client>,
}

impl Network {
    pub fn new() -> Self {
        Network {
            storage_nodes: HashMap::new(),
            clients: HashMap::new(),
        }
    }

    pub fn add_storage_node(&mut self, peer_id: PeerId) {
        self.storage_nodes.insert(peer_id, StorageNode::new(peer_id));
    }

    pub fn add_client(&mut self, peer_id: PeerId) {
        self.clients.insert(peer_id, Client::new(peer_id));
    }

    pub fn list_clients(&self) -> Vec<PeerId> {
        self.clients.keys().cloned().collect()
    }

    pub fn list_storage_nodes(&self) -> Vec<PeerId> {
        self.storage_nodes.keys().cloned().collect()
    }

    pub fn storage_nodes(&self) -> &HashMap<PeerId, StorageNode> {
        &self.storage_nodes
    }

    pub fn clients(&self) -> &HashMap<PeerId, Client> {
        &self.clients
    }

    pub fn upload_file(&mut self, client_id: &PeerId, storage_node_id: &PeerId, filename: String, data: Vec<u8>) -> Result<(), &'static str> {
        let client = self.clients.get_mut(client_id).ok_or("Client not found")?;
        let storage_node = self.storage_nodes.get_mut(storage_node_id).ok_or("Storage node not found")?;

        let cost = (data.len() as u64) * STORAGE_COST_PER_BYTE;
        if !client.subtract_balance(cost) {
            return Err("Insufficient balance for upload");
        }

        if let Err(e) = storage_node.store_file(filename.clone(), data) {
            client.add_balance(cost); // Refund the client if storage fails
            return Err(e);
        }

        storage_node.add_balance(cost);
        client.add_file(filename);
        Ok(())
    }

    pub fn download_file(&self, _client_id: &PeerId, storage_node_id: &PeerId, filename: &str) -> Result<Vec<u8>, &'static str> {
        let storage_node = self.storage_nodes.get(storage_node_id).ok_or("Storage node not found")?;
        storage_node.get_file(filename).cloned().ok_or("File not found")
    }

    pub fn remove_file(&mut self, client_id: &PeerId, storage_node_id: &PeerId, filename: &str) -> Result<(), &'static str> {
        let client = self.clients.get_mut(client_id).ok_or("Client not found")?;
        let storage_node = self.storage_nodes.get_mut(storage_node_id).ok_or("Storage node not found")?;

        if !client.remove_file(filename) {
            return Err("File not found in client's list");
        }

        storage_node.remove_file(filename)
    }

    pub fn replicate_file(&mut self, source_node_id: &PeerId, target_node_id: &PeerId, filename: &str) -> Result<(), &'static str> {
        // First, get the file data and check the balance
        let file_data = {
            let source_node = self.storage_nodes.get(source_node_id).ok_or("Source storage node not found")?;
            let file_data = source_node.get_file(filename).ok_or("File not found on source node")?.clone();
            let replication_cost = (file_data.len() as u64) * REPLICATION_COST_PER_BYTE;

            if source_node.balance() < replication_cost {
                return Err("Insufficient balance for replication");
            }
            (file_data, replication_cost)
        };

        // Now, store the file in the target node
        {
            let target_node = self.storage_nodes.get_mut(target_node_id).ok_or("Target storage node not found")?;
            if let Err(e) = target_node.store_file(filename.to_string(), file_data.0.clone()) {
                return Err(e);
            }
            target_node.add_balance(file_data.1);
        }

        // Finally, update the source node's balance
        let source_node = self.storage_nodes.get_mut(source_node_id).ok_or("Source storage node not found")?;
        source_node.subtract_balance(file_data.1);

        Ok(())
    }
}
