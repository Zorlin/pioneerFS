use libp2p::PeerId;
use std::collections::HashMap;

const MAX_STORAGE: usize = 1_000_000_000; // 1GB max storage

pub struct StorageNode {
    peer_id: PeerId,
    balance: u64,
    stored_files: HashMap<String, Vec<u8>>,
    available_space: usize,
}

impl StorageNode {
    pub fn new(peer_id: PeerId) -> Self {
        StorageNode {
            peer_id,
            balance: 0,
            stored_files: HashMap::new(),
            available_space: MAX_STORAGE,
        }
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn balance(&self) -> u64 {
        self.balance
    }

    pub fn add_balance(&mut self, amount: u64) {
        self.balance += amount;
    }

    pub fn subtract_balance(&mut self, amount: u64) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            true
        } else {
            false
        }
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
}
