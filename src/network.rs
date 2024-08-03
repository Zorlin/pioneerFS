use crate::{StorageNode, Client, erc20::ERC20};
use libp2p::PeerId;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};
use rand::seq::SliceRandom;

const DEAL_DURATION: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours

#[derive(Debug, Clone, Copy)]
pub enum DebugLevel {
    None,
    Low,
    High,
}

impl DebugLevel {
    pub fn is_enabled(&self) -> bool {
        matches!(self, DebugLevel::Low | DebugLevel::High)
    }
}

impl Network {
    pub fn request_higher_replication(&mut self, client_id: &PeerId, filename: &str, new_replication_factor: usize) -> Result<(), String> {
        self.debug_log(&format!("Requesting higher replication for file: {} from client: {} to factor: {}", filename, client_id, new_replication_factor));

        let current_storage_nodes = {
            let client = self.clients.get(client_id).ok_or_else(|| "Client not found".to_string())?;
            client.get_file_locations(filename).ok_or_else(|| "File not found".to_string())?.clone()
        };


        if new_replication_factor <= current_storage_nodes.len() {
            return Err(format!("New replication factor must be higher than current ({}).", current_storage_nodes.len()));
        }


        let additional_replications = new_replication_factor - current_storage_nodes.len();
        let available_nodes: Vec<PeerId> = self.storage_nodes.keys()
            .filter(|&id| !current_storage_nodes.contains(id))
            .cloned()
            .collect();

        if available_nodes.len() < additional_replications {
            return Err(format!("Not enough additional storage nodes available. Required: {}, Available: {}", additional_replications, available_nodes.len()));
        }


        let selected_nodes: Vec<PeerId> = available_nodes.choose_multiple(&mut rand::thread_rng(), additional_replications).cloned().collect();

        // Retrieve the file data from one of the existing storage nodes
        let file_data = {
            let existing_node_id = current_storage_nodes.first().ok_or_else(|| "No existing storage nodes".to_string())?;
            self.storage_nodes.get(existing_node_id)
                .ok_or_else(|| "Storage node not found".to_string())?
                .get_file(filename)
                .ok_or_else(|| "File not found on storage node".to_string())?
                .clone()
        };

        // Replicate the file to the new nodes
        for &node_id in &selected_nodes {
            let storage_node = self.storage_nodes.get_mut(&node_id).ok_or_else(|| "Storage node not found".to_string())?;
            storage_node.store_file(filename.to_string(), file_data.clone())?;
        };

        // Initialize with at least one storage node and one client
        let _initial_client_id = PeerId::random();
        let _initial_storage_node_id = PeerId::random();

        // Update the client's file locations
        if let Some(client) = self.clients.get_mut(client_id) {
            let mut updated_locations = current_storage_nodes;
            updated_locations.extend(selected_nodes);
            client.add_file(filename.to_string(), updated_locations);
        };

        self.debug_log(&format!("Successfully increased replication factor for file: {} to {}", filename, new_replication_factor));
        Ok(())
    }
}

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
    pub storage_nodes: HashMap<PeerId, StorageNode>,
    pub clients: HashMap<PeerId, Client>,
    pub deals: Vec<Deal>,
    pub marketplace: VecDeque<StorageOffer>,
    pub token: ERC20,
    pub bids: HashMap<String, Vec<Bid>>,
    pub debug_level: DebugLevel,
}

pub struct Bid {
    pub storage_node_id: PeerId,
    pub price_per_gb: u64,
}

impl Bid {
    pub fn new(storage_node_id: PeerId, price_per_gb: u64) -> Self {
        Self {
            storage_node_id,
            price_per_gb,
        };
    }
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
        };
    }

    pub fn start_time(&self) -> Instant {
        self.start_time.unwrap_or_else(Instant::now)
    }
}

impl Network {
    pub fn new() -> Self {
        let mut network = Network {
            storage_nodes: HashMap::new(),
            clients: HashMap::new(),
            deals: Vec::new(),
            marketplace: VecDeque::new(),
            token: ERC20::new("PioDollar".to_string(), "PIO".to_string(), 1_000_000_000), // 1 billion initial supply
            bids: HashMap::new(),
            debug_level: DebugLevel::None,
        };

        // Initialize with at least one storage node and one client
        let initial_client_id = PeerId::random();
        let initial_storage_node_id = PeerId::random();
        network.add_client(initial_client_id);
        network.add_storage_node(initial_storage_node_id, 10); // Example price per GB

        network
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

    fn debug_log(&self, message: &str) {
        if self.debug_level.is_enabled() {
            println!("[DEBUG] {}", message);
        }
    }

    pub fn set_debug_level(&mut self, level: DebugLevel) {
        self.debug_level = level;
    }

    pub fn add_storage_node(&mut self, peer_id: PeerId, price_per_gb: u64) {
        self.storage_nodes.insert(peer_id, StorageNode::new(peer_id, price_per_gb));
    }

    pub fn add_client(&mut self, peer_id: PeerId) {
        self.clients.insert(peer_id, Client::new(peer_id));
        // Initialize client with 1,000,000 tokens
        self.token.mint(&peer_id, 1_000_000);
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

    pub fn upload_file(&mut self, client_id: &PeerId, filename: String, data: Vec<u8>, replication_factor: usize) -> Result<Vec<PeerId>, String> {
        self.debug_log(&format!("Uploading file: {} for client: {} with replication factor: {}", filename, client_id, replication_factor));

        // Select storage nodes
        let available_nodes: Vec<PeerId> = self.storage_nodes.keys().cloned().collect();
        if available_nodes.len() < replication_factor {
            return Err(format!("Not enough storage nodes available. Required: {}, Available: {}", replication_factor, available_nodes.len()));
        }

        let selected_nodes: Vec<PeerId> = available_nodes.choose_multiple(&mut rand::thread_rng(), replication_factor).cloned().collect();
        self.debug_log(&format!("Selected nodes for storage: {:?}", selected_nodes));

        // Calculate total cost
        let file_size_gb = (data.len() as f64 / (1024.0 * 1024.0 * 1024.0)).ceil() as u64;
        let total_cost: u64 = selected_nodes.iter()
            .map(|node_id| file_size_gb * self.storage_nodes.get(node_id).unwrap().price_per_gb())
            .sum();
        self.debug_log(&format!("Total cost for upload: {} tokens", total_cost));

        // Check if the client has enough balance
        let client_balance = self.token.balance_of(client_id);
        if client_balance < total_cost {
            return Err(format!("Insufficient balance to upload file. Required: {}, Available: {}", total_cost, client_balance));
        }

        // Store file and create deals
        let mut stored_nodes = Vec::new();
        for node_id in &selected_nodes {
            let storage_node = self.storage_nodes.get_mut(node_id).unwrap();
            
            if let Err(e) = storage_node.store_file(filename.clone(), data.clone()) {
                return Err(format!("Failed to store file on node {}: {}", node_id, e));
            }

            let node_cost = file_size_gb * storage_node.price_per_gb();
            if !self.token.transfer(client_id, node_id, node_cost) {
                return Err("Failed to transfer tokens".to_string());
            }
            self.debug_log(&format!("Transferred {} tokens from {} to {} for storage", node_cost, client_id, node_id));

            self.deals.push(Deal::new(
                *client_id,
                *node_id,
                filename.clone(),
                DEAL_DURATION,
            ));
            self.debug_log(&format!("Created new deal: client {} with storage node {} for file {}", client_id, node_id, filename));

            stored_nodes.push(*node_id);
        }

        // Update client's file record
        let client = self.clients.get_mut(client_id).ok_or_else(|| "Client not found".to_string())?;
        client.add_file(filename.clone(), stored_nodes.clone());
        self.debug_log(&format!("Updated client {} file record for {}", client_id, filename));

        Ok(stored_nodes)
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

    pub fn download_file(&self, client_id: &PeerId, filename: &str) -> Result<Vec<u8>, String> {
        let client = self.clients.get(client_id).ok_or_else(|| "Client not found".to_string())?;
        let storage_nodes = client.get_file_locations(filename).ok_or_else(|| "File not found".to_string())?;

        for node_id in storage_nodes {
            if let Some(storage_node) = self.storage_nodes.get(node_id) {
                if let Some(file_data) = storage_node.get_file(filename) {
                    return Ok(file_data.clone());
                }
            }
        }

        Err("File not found on any storage node".to_string())
    }

    pub fn get_file_locations(&self, client_id: &PeerId, filename: &str) -> Result<Vec<PeerId>, String> {
        let client = self.clients.get(client_id).ok_or_else(|| "Client not found".to_string())?;
        client.get_file_locations(filename).cloned().ok_or_else(|| "File not found".to_string())
    }

    pub fn get_file_content(&self, node_id: &PeerId, filename: &str) -> Result<Vec<u8>, String> {
        let storage_node = self.storage_nodes.get(node_id).ok_or_else(|| "Storage node not found".to_string())?;
        storage_node.get_file(filename).cloned().ok_or_else(|| "File not found on storage node".to_string())
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
            if let Err(e) = self.remove_file(&deal.client_id, &deal.filename) {
                println!("Error removing expired file: {}", e);
            }
            self.deals.retain(|d| d.filename != deal.filename || d.client_id != deal.client_id);
        }
    }

    pub fn remove_file(&mut self, client_id: &PeerId, filename: &str) -> Result<(), &'static str> {
        let client = self.clients.get_mut(client_id).ok_or("Client not found")?;
        let storage_nodes = client.get_file_locations(filename).ok_or("File not found")?;

        for &node_id in storage_nodes {
            if let Some(storage_node) = self.storage_nodes.get_mut(&node_id) {
                storage_node.remove_file(filename)?;
            }
        }

        client.remove_file(filename);
        self.deals.retain(|d| d.filename != filename || d.client_id != *client_id);
        Ok(())
    }

    pub fn replicate_file(&mut self, client_id: &PeerId, filename: &str, remaining_replications: usize) -> Result<(), String> {
        let client = self.clients.get(client_id).ok_or("Client not found".to_string())?;
        let storage_nodes = client.get_file_locations(filename).ok_or("File not found".to_string())?;
        
        if storage_nodes.is_empty() {
            return Err("No storage nodes found for the file".to_string());
        }

        let source_node_id = storage_nodes[0];
        let file_data = {
            let source_node = self.storage_nodes.get(&source_node_id).ok_or("Source storage node not found".to_string())?;
            source_node.get_file(filename).ok_or("File not found on source node".to_string())?.clone()
        };

        self.chain_upload(&source_node_id, filename, &file_data, remaining_replications)
            .map_err(|e| e.to_string())
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
