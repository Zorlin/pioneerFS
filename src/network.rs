use crate::{StorageNode, Client, erc20::ERC20};
use libp2p::PeerId;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};

const DEAL_DURATION: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours
const REPLICATION_FACTOR: usize = 3; // Default replication factor for the TESTNET

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub storage_nodes: HashMap<PeerId, StorageNodeStatus>,
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub clients: HashMap<PeerId, ClientStatus>,
    pub deals: Vec<Deal>,
    pub marketplace: VecDeque<StorageOffer>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StorageNodeStatus {
    pub available_space: usize,
    pub stored_files: Vec<String>,
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct ClientStatus {
    #[serde_as(as = "HashMap<_, Vec<DisplayFromStr>>")]
    pub files: HashMap<String, Vec<PeerId>>,
}

pub struct Network {
    storage_nodes: HashMap<PeerId, StorageNode>,
    clients: HashMap<PeerId, Client>,
    deals: Vec<Deal>,
    marketplace: VecDeque<StorageOffer>,
    pub token: ERC20,
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct StorageOffer {
    #[serde_as(as = "DisplayFromStr")]
    pub storage_node_id: PeerId,
    pub price_per_gb: u64,
    pub available_space: usize,
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct Deal {
    #[serde_as(as = "DisplayFromStr")]
    client_id: PeerId,
    #[serde_as(as = "DisplayFromStr")]
    storage_node_id: PeerId,
    filename: String,
    #[serde(skip)]
    start_time: Option<Instant>,
    #[serde(with = "serde_millis")]
    duration: Duration,
}

impl Deal {
    pub fn new(client_id: PeerId, storage_node_id: PeerId, filename: String, duration: Duration) -> Self {
        Self {
            client_id,
            storage_node_id,
            filename,
            start_time: Some(Instant::now()),
            duration,
        }
    }

    pub fn start_time(&self) -> Instant {
        self.start_time.unwrap_or_else(Instant::now)
    }
}

impl Network {
    pub fn new() -> Self {
        Network {
            storage_nodes: HashMap::new(),
            clients: HashMap::new(),
            deals: Vec::new(),
            marketplace: VecDeque::new(),
            token: ERC20::new("PioDollar".to_string(), "PIO".to_string(), 1_000_000_000), // 1 billion initial supply
        }
    }

    pub fn get_network_status(&self) -> NetworkStatus {
        NetworkStatus {
            storage_nodes: self.storage_nodes.iter().map(|(id, node)| (*id, StorageNodeStatus {
                available_space: node.available_space(),
                stored_files: node.stored_files().keys().cloned().collect(),
            })).collect(),
            clients: self.clients.iter().map(|(id, client)| (*id, ClientStatus {
                files: client.list_files().clone(),
            })).collect(),
            deals: self.deals.clone(),
            marketplace: self.marketplace.clone(),
        }
    }

    pub fn add_storage_node(&mut self, peer_id: PeerId, price_per_gb: u64) {
        self.storage_nodes.insert(peer_id, StorageNode::new(peer_id, price_per_gb));
    }

    pub fn add_client(&mut self, peer_id: PeerId) {
        self.clients.insert(peer_id, Client::new(peer_id));
        // Initialize client with 1,000,000 tokens
        self.token.transfer(&PeerId::random(), &peer_id, 1_000_000);
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

    pub fn get_balance(&self, peer_id: &PeerId) -> u64 {
        self.token.balance_of(peer_id)
    }

    pub fn upload_file(&mut self, client_id: &PeerId, storage_node_id: &PeerId, filename: String, data: Vec<u8>) -> Result<(), String> {
        let storage_node = self.storage_nodes.get_mut(storage_node_id).ok_or_else(|| "Storage node not found".to_string())?;

        // Calculate the cost of storage
        let file_size_gb = (data.len() as f64 / (1024.0 * 1024.0 * 1024.0)).ceil() as u64;
        let cost = file_size_gb * storage_node.price_per_gb();

        // Check if the client has enough balance and transfer tokens
        let client_balance = self.token.balance_of(client_id);
        if client_balance < cost {
            return Err(format!("Insufficient balance to upload file. Required: {}, Available: {}", cost, client_balance));
        }

        if !self.token.transfer(client_id, storage_node_id, cost) {
            return Err("Failed to transfer tokens".to_string());
        }

        if let Err(e) = storage_node.store_file(filename.clone(), data.clone()) {
            // If storing fails, refund the client
            self.token.transfer(storage_node_id, client_id, cost);
            return Err(format!("Failed to store file: {}", e));
        }

        let client = self.clients.get_mut(client_id).ok_or_else(|| "Client not found".to_string())?;
        client.add_file(filename.clone(), *storage_node_id);

        // Create a new deal
        self.deals.push(Deal::new(
            *client_id,
            *storage_node_id,
            filename.clone(),
            DEAL_DURATION,
        ));

        // Chain upload to other storage nodes
        if let Err(e) = self.chain_upload(storage_node_id, &filename, &data, REPLICATION_FACTOR - 1) {
            println!("Warning: Failed to replicate file: {}", e);
        }

        Ok(())
    }

    fn chain_upload(&mut self, source_node_id: &PeerId, filename: &str, data: &Vec<u8>, remaining_replications: usize) -> Result<(), &'static str> {
        if remaining_replications == 0 {
            return Ok(());
        }

        let available_nodes: Vec<PeerId> = self.storage_nodes.keys()
            .filter(|&id| id != source_node_id)
            .cloned()
            .collect();

        if available_nodes.is_empty() {
            return Err("No available storage nodes for replication");
        }

        let target_node_id = available_nodes[rand::random::<usize>() % available_nodes.len()];
        let target_node = self.storage_nodes.get_mut(&target_node_id).unwrap();

        target_node.store_file(filename.to_string(), data.clone())?;

        // Recursively continue the chain upload
        self.chain_upload(&target_node_id, filename, data, remaining_replications - 1)
    }

    pub fn download_file(&self, _client_id: &PeerId, storage_node_id: &PeerId, filename: &str) -> Result<Vec<u8>, &'static str> {
        let storage_node = self.storage_nodes.get(storage_node_id).ok_or("Storage node not found")?;
        storage_node.get_file(filename).cloned().ok_or("File not found")
    }

    pub fn renew_deal(&mut self, client_id: &PeerId, storage_node_id: &PeerId, filename: &str) -> Result<(), &'static str> {
        let deal = self.deals.iter_mut()
            .find(|d| d.client_id == *client_id && d.storage_node_id == *storage_node_id && d.filename == filename)
            .ok_or("Deal not found")?;

        deal.start_time = Some(Instant::now());
        Ok(())
    }

    pub fn check_deals(&mut self) {
        let mut expired_deals = Vec::new();
        for deal in &self.deals {
            if deal.start_time().elapsed() > deal.duration {
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
        let storage_node = self.storage_nodes.get_mut(&offer.storage_node_id).ok_or("Storage node not found")?;

        if file_size > offer.available_space {
            return Err("Not enough space in the offer");
        }

        let price = (file_size as u64 * offer.price_per_gb) / (1024 * 1024 * 1024); // Convert to GB
        if !self.token.transfer(client_id, &offer.storage_node_id, price) {
            return Err("Client doesn't have enough balance");
        }

        storage_node.reserve_space(file_size)?;

        // Remove the offer and add a new one with updated available space
        self.marketplace.remove(offer_index);
        if offer.available_space > file_size {
            self.add_storage_offer(offer.storage_node_id, offer.price_per_gb, offer.available_space - file_size);
        }

        Ok(())
    }
}
