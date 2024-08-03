use libp2p::PeerId;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};

const MAX_STORAGE: usize = 1_000_000_000; // 1GB max storage

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]

pub struct StorageNode {
    #[serde_as(as = "DisplayFromStr")]
    peer_id: PeerId,
    stored_files: HashMap<String, Vec<u8>>,
    available_space: usize,
    reputation: u64,
    price_per_gb: u64,
}

impl StorageNode {
    pub fn new(peer_id: PeerId, price_per_gb: u64) -> Self {
        StorageNode {
            peer_id,
            stored_files: HashMap::new(),
            available_space: MAX_STORAGE,
            reputation: 100, // Start with a base reputation
            price_per_gb,
        }
    }

    pub fn price_per_gb(&self) -> u64 {
        self.price_per_gb
    }

    pub fn reputation(&self) -> u64 {
        self.reputation
    }

    pub fn increase_reputation(&mut self, amount: u64) {
        self.reputation += amount;
    }

    pub fn decrease_reputation(&mut self, amount: u64) {
        self.reputation = self.reputation.saturating_sub(amount);
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn store_file(&mut self, filename: String, data: Vec<u8>) -> Result<(), &'static str> {
        if data.len() > self.available_space {
            return Err("Not enough space to store the file");
        }
        self.available_space -= data.len();
        self.stored_files.insert(filename, data);
        Ok(())
    }

    pub fn get_file(&self, filename: &str) -> Option<&Vec<u8>> {
        self.stored_files.get(filename)
    }

    pub fn remove_file(&mut self, filename: &str) -> Result<(), &'static str> {
        if let Some(file) = self.stored_files.remove(filename) {
            self.available_space += file.len();
            Ok(())
        } else {
            Err("File not found")
        }
    }

    pub fn available_space(&self) -> usize {
        self.available_space
    }

    pub fn stored_files(&self) -> &HashMap<String, Vec<u8>> {
        &self.stored_files
    }

    pub fn reserve_space(&mut self, size: usize) -> Result<(), &'static str> {
        if size > self.available_space {
            Err("Not enough available space")
        } else {
            self.available_space -= size;
            Ok(())
        }
    }

    pub fn get_price_per_gb(&self) -> u64 {
        self.price_per_gb
    }

    pub fn set_price_per_gb(&mut self, price: u64) {
        self.price_per_gb = price;
    }
}
