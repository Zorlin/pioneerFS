use libp2p::PeerId;

pub struct Client {
    peer_id: PeerId,
    balance: u64,
    files: Vec<String>,
}

impl Client {
    pub fn new(peer_id: PeerId) -> Self {
        Client {
            peer_id,
            balance: 100000, // Start with 100,000 PIO
            files: Vec::new(),
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

    pub fn add_file(&mut self, filename: String) {
        self.files.push(filename);
    }

    pub fn remove_file(&mut self, filename: &str) -> bool {
        if let Some(index) = self.files.iter().position(|f| f == filename) {
            self.files.remove(index);
            true
        } else {
            false
        }
    }

    pub fn list_files(&self) -> &[String] {
        &self.files
    }
}
