use crate::{StorageNode, Client, erc20::ERC20};
use libp2p::PeerId;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};
use rand::seq::SliceRandom;

const DEAL_DURATION: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours
const REPLICATION_FACTOR: usize = 3; // Default replication factor for the TESTNET
const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunk size for erasure coding

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
        }
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
            bids: HashMap::new(),
        }
    }

    pub fn add_bid(&mut self, filename: String, storage_node_id: PeerId, price_per_gb: u64) {
        let bid = Bid {
            storage_node_id,
            price_per_gb,
        };
        self.bids.entry(filename).or_insert_with(Vec::new).push(bid);
    }

    pub fn get_bids(&self, filename: &str) -> Option<&Vec<Bid>> {
        self.bids.get(filename)
    }

    fn erasure_code_file(&self, data: &[u8]) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        for chunk in data.chunks(CHUNK_SIZE) {
            chunks.push(chunk.to_vec());
        }
        
        // For simplicity, we'll just duplicate each chunk as a basic form of erasure coding
        // In a real implementation, you'd use a proper erasure coding library
        chunks.iter().flat_map(|chunk| vec![chunk.clone(), chunk.clone()]).collect()
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

    pub fn upload_file(&mut self, client_id: &PeerId, filename: String, data: Vec<u8>) -> Result<(), String> {
        // Calculate the cost of storage (1 token per chunk)
        let total_chunks = (data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let storage_cost = total_chunks as u64;

        // Check if the client has enough balance
        if self.get_balance(client_id) < storage_cost {
            return Err("Insufficient balance for file upload".to_string());
        }

        // Select storage nodes
        let available_nodes: Vec<PeerId> = self.storage_nodes.keys().cloned().collect();
        if available_nodes.len() < REPLICATION_FACTOR {
            return Err("Not enough storage nodes available".to_string());
        }

        // Deduct tokens from the client
        self.token.transfer(client_id, &PeerId::random(), storage_cost);

        let selected_nodes: Vec<PeerId> = available_nodes.choose_multiple(&mut rand::thread_rng(), REPLICATION_FACTOR).cloned().collect();

        // Calculate total cost
        let total_size_gb = (encoded_chunks.iter().map(|chunk| chunk.len()).sum::<usize>() as f64 / (1024.0 * 1024.0 * 1024.0)).ceil() as u64;
        let total_cost: u64 = selected_nodes.iter()
            .map(|node_id| total_size_gb * self.storage_nodes.get(node_id).unwrap().price_per_gb())
            .sum();

        // Check if the client has enough balance
        let client_balance = self.token.balance_of(client_id);
        if client_balance < total_cost {
            return Err(format!("Insufficient balance to upload file. Required: {}, Available: {}", total_cost, client_balance));
        }

        // Store chunks and create deals
        let mut stored_nodes = Vec::new();
        for (i, chunk) in encoded_chunks.iter().enumerate() {
            let node_id = &selected_nodes[i % selected_nodes.len()];
            let storage_node = self.storage_nodes.get_mut(node_id).unwrap();
            let chunk_filename = format!("{}_chunk_{}", filename, i);
            
            if let Err(e) = storage_node.store_file(chunk_filename.clone(), chunk.clone()) {
                return Err(format!("Failed to store chunk on node {}: {}", node_id, e));
            }

            let chunk_cost = (chunk.len() as f64 / (1024.0 * 1024.0 * 1024.0)).ceil() as u64 * storage_node.price_per_gb();
            if !self.token.transfer(client_id, node_id, chunk_cost) {
                return Err("Failed to transfer tokens".to_string());
            }

            self.deals.push(Deal::new(
                *client_id,
                *node_id,
                chunk_filename,
                DEAL_DURATION,
            ));

            stored_nodes.push(*node_id);
        }

        // Update client's file record
        let client = self.clients.get_mut(client_id).ok_or_else(|| "Client not found".to_string())?;
        client.add_file(filename.clone(), stored_nodes);

        // Initiate replication process if needed
        if total_chunks > REPLICATION_FACTOR {
            self.replicate_file(client_id, &filename, total_chunks - REPLICATION_FACTOR)?;
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

    pub fn download_file(&self, client_id: &PeerId, filename: &str) -> Result<Vec<u8>, String> {
        let client = self.clients.get(client_id).ok_or_else(|| "Client not found".to_string())?;
        let storage_nodes = client.get_file_locations(filename).ok_or_else(|| "File not found".to_string())?;

        let mut chunks = Vec::new();
        let mut i = 0;
        while let Some(chunk) = self.get_chunk(filename, i, &storage_nodes) {
            chunks.push(chunk);
            i += 1;
        }

        if chunks.is_empty() {
            return Err("No chunks found for the file".to_string());
        }

        // For our simple erasure coding, we just need to take every other chunk
        let original_data: Vec<u8> = chunks.into_iter().step_by(2).flatten().collect();
        Ok(original_data)
    }

    fn get_chunk(&self, filename: &str, chunk_index: usize, storage_nodes: &[PeerId]) -> Option<Vec<u8>> {
        for node_id in storage_nodes {
            if let Some(storage_node) = self.storage_nodes.get(node_id) {
                let chunk_filename = format!("{}_chunk_{}", filename, chunk_index);
                if let Some(chunk) = storage_node.get_file(&chunk_filename) {
                    return Some(chunk.clone());
                }
            }
        }
        None
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

    pub fn replicate_file(&mut self, client_id: &PeerId, filename: &str, remaining_replications: usize) -> Result<(), &'static str> {
        let client = self.clients.get(client_id).ok_or("Client not found")?;
        let storage_nodes = client.get_file_locations(filename).ok_or("File not found")?;
        
        if storage_nodes.is_empty() {
            return Err("No storage nodes found for the file");
        }

        let source_node_id = storage_nodes[0];
        let file_data = {
            let source_node = self.storage_nodes.get(&source_node_id).ok_or("Source storage node not found")?;
            source_node.get_file(filename).ok_or("File not found on source node")?.clone()
        };

        self.chain_upload(&source_node_id, filename, &file_data, remaining_replications)
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
