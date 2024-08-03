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

    pub fn add_file(&mut self, filename: String, storage_node: PeerId) {
        self.files.entry(filename).or_insert_with(Vec::new).push(storage_node);
    }

    pub fn remove_file(&mut self, filename: &str, storage_node: &PeerId) -> bool {
        if let Some(nodes) = self.files.get_mut(filename) {
            if let Some(pos) = nodes.iter().position(|&node| node == *storage_node) {
                nodes.remove(pos);
                if nodes.is_empty() {
                    self.files.remove(filename);
                }
                return true;
            }
        }
        false
    }

    pub fn list_files(&self) -> &HashMap<String, Vec<PeerId>> {
        &self.files
    }

    pub fn upload_file(&mut self, filename: String, _data: Vec<u8>, storage_node: PeerId) -> Result<(), &'static str> {
        self.files.entry(filename).or_insert_with(Vec::new).push(storage_node);
        Ok(())
    }

    pub fn download_file(&self, filename: &str) -> Option<&Vec<PeerId>> {
        self.files.get(filename)
    }
}
