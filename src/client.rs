use libp2p::PeerId;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]

pub struct Client {
    #[serde_as(as = "DisplayFromStr")]
    peer_id: PeerId,
    #[serde_as(as = "HashMap<_, Vec<DisplayFromStr>>")]
    files: HashMap<String, Vec<PeerId>>,
}

impl Client {
    pub fn new(peer_id: PeerId) -> Self {
        Client {
            peer_id,
            files: HashMap::new(),
        }
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn add_file(&mut self, filename: String, storage_nodes: Vec<PeerId>) {
        self.files.insert(filename, storage_nodes);
    }

    pub fn remove_file(&mut self, filename: &str) -> bool {
        self.files.remove(filename).is_some()
    }

    pub fn list_files(&self) -> &HashMap<String, Vec<PeerId>> {
        &self.files
    }

    pub fn get_file_locations(&self, filename: &str) -> Option<&Vec<PeerId>> {
        self.files.get(filename)
    }
}
