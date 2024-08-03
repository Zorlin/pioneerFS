use crate::{StorageNode, Client};
use libp2p::PeerId;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

const DEAL_DURATION: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

pub struct Network {
    storage_nodes: HashMap<PeerId, StorageNode>,
    clients: HashMap<PeerId, Client>,
    deals: Vec<Deal>,
    marketplace: VecDeque<StorageOffer>,
}

pub struct StorageOffer {
    storage_node_id: PeerId,
    price_per_gb: u64,
    available_space: usize,
}

#[derive(Clone)]
pub struct Deal {
    client_id: PeerId,
    storage_node_id: PeerId,
    filename: String,
    start_time: Instant,
    duration: Duration,
}

impl Network {
    pub fn new() -> Self {
        Network {
            storage_nodes: HashMap::new(),
            clients: HashMap::new(),
            deals: Vec::new(),
            marketplace: VecDeque::new(),
        }
    }

    pub fn add_storage_node(&mut self, peer_id: PeerId, price_per_gb: u64) {
        self.storage_nodes.insert(peer_id, StorageNode::new(peer_id, price_per_gb));
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

        if let Err(e) = storage_node.store_file(filename.clone(), data.clone()) {
            return Err(e);
        }

        client.add_file(filename.clone(), *storage_node_id);

        // Create a new deal
        self.deals.push(Deal {
            client_id: *client_id,
            storage_node_id: *storage_node_id,
            filename: filename.clone(),
            start_time: Instant::now(),
            duration: DEAL_DURATION,
        });

        Ok(())
    }

    pub fn download_file(&self, _client_id: &PeerId, storage_node_id: &PeerId, filename: &str) -> Result<Vec<u8>, &'static str> {
        let storage_node = self.storage_nodes.get(storage_node_id).ok_or("Storage node not found")?;
        storage_node.get_file(filename).cloned().ok_or("File not found")
    }

    pub fn renew_deal(&mut self, client_id: &PeerId, storage_node_id: &PeerId, filename: &str) -> Result<(), &'static str> {
        let deal = self.deals.iter_mut()
            .find(|d| d.client_id == *client_id && d.storage_node_id == *storage_node_id && d.filename == filename)
            .ok_or("Deal not found")?;

        deal.start_time = Instant::now();
        Ok(())
    }

    pub fn check_deals(&mut self) {
        let mut expired_deals = Vec::new();
        for deal in &self.deals {
            if deal.start_time.elapsed() > deal.duration {
                expired_deals.push(deal.clone());
            }
        }

        for deal in expired_deals {
            if let Err(e) = self.remove_file(&deal.client_id, &deal.storage_node_id, &deal.filename) {
                println!("Error removing expired file: {}", e);
            }
            self.deals.retain(|d| d.filename != deal.filename || d.client_id != deal.client_id);
        }
    }

    pub fn remove_file(&mut self, client_id: &PeerId, storage_node_id: &PeerId, filename: &str) -> Result<(), &'static str> {
        let client = self.clients.get_mut(client_id).ok_or("Client not found")?;
        let storage_node = self.storage_nodes.get_mut(storage_node_id).ok_or("Storage node not found")?;

        if !client.remove_file(filename, storage_node_id) {
            return Err("File not found in client's list");
        }

        storage_node.remove_file(filename)
    }

    pub fn replicate_file(&mut self, source_node_id: &PeerId, target_node_id: &PeerId, filename: &str) -> Result<(), &'static str> {
        let file_data = {
            let source_node = self.storage_nodes.get(source_node_id).ok_or("Source storage node not found")?;
            source_node.get_file(filename).ok_or("File not found on source node")?.clone()
        };

        let target_node = self.storage_nodes.get_mut(target_node_id).ok_or("Target storage node not found")?;
        target_node.store_file(filename.to_string(), file_data)?;

        Ok(())
    }

    pub fn add_storage_offer(&mut self, storage_node_id: PeerId, price_per_gb: u64, available_space: usize) {
        let offer = StorageOffer {
            storage_node_id,
            price_per_gb,
            available_space,
        };
        self.marketplace.push_back(offer);
    }

    pub fn get_storage_offers(&self) -> &VecDeque<StorageOffer> {
        &self.marketplace
    }

    pub fn accept_storage_offer(&mut self, client_id: &PeerId, offer_index: usize, file_size: usize) -> Result<(), &'static str> {
        let offer = self.marketplace.get(offer_index).cloned().ok_or("Offer not found")?;
        let client = self.clients.get_mut(client_id).ok_or("Client not found")?;
        let storage_node = self.storage_nodes.get_mut(&offer.storage_node_id).ok_or("Storage node not found")?;

        if file_size > offer.available_space {
            return Err("Not enough space in the offer");
        }

        let price = (file_size as u64 * offer.price_per_gb) / (1024 * 1024 * 1024); // Convert to GB
        if !client.subtract_balance(price) {
            return Err("Client doesn't have enough balance");
        }

        storage_node.add_balance(price);
        storage_node.reserve_space(file_size)?;

        // Remove the offer and add a new one with updated available space
        self.marketplace.remove(offer_index);
        if offer.available_space > file_size {
            self.add_storage_offer(offer.storage_node_id, offer.price_per_gb, offer.available_space - file_size);
        }

        Ok(())
    }
}
