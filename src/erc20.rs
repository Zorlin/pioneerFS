use std::collections::HashMap;
use libp2p::PeerId;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct ERC20 {
    name: String,
    symbol: String,
    total_supply: u64,
    balances: HashMap<PeerId, u64>,
    allowances: HashMap<PeerId, HashMap<PeerId, u64>>,
    debug: bool,
    pub message_sender: Option<Sender<String>>,
}

impl ERC20 {
    pub fn new(name: String, symbol: String, initial_supply: u64) -> Self {
        ERC20 {
            name,
            symbol,
            total_supply: initial_supply,
            balances: HashMap::new(),
            allowances: HashMap::new(),
            debug: false,
            message_sender: None,
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    fn debug_log(&self, message: &str) {
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(format!("[ERC20 DEBUG] {}", message));
        } else if self.debug {
            println!("[ERC20 DEBUG] {}", message);
        }
    }

    pub fn balance_of(&self, account: &PeerId) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }

    pub fn transfer(&mut self, from: &PeerId, to: &PeerId, amount: u64) -> bool {
        self.debug_log(&format!("Attempting transfer: {} tokens from {} to {}", amount, from, to));
        if self.balance_of(from) < amount {
            self.debug_log("Transfer failed: Insufficient balance");
            return false;
        }
        *self.balances.entry(*from).or_insert(0) -= amount;
        *self.balances.entry(*to).or_insert(0) += amount;
        self.debug_log("Transfer successful");
        true
    }

    pub fn approve(&mut self, owner: &PeerId, spender: &PeerId, amount: u64) -> bool {
        self.allowances.entry(*owner).or_default().insert(*spender, amount);
        true
    }

    pub fn allowance(&self, owner: &PeerId, spender: &PeerId) -> u64 {
        *self.allowances.get(owner).and_then(|inner| inner.get(spender)).unwrap_or(&0)
    }

    pub fn transfer_from(&mut self, from: &PeerId, to: &PeerId, amount: u64) -> bool {
        let allowance = self.allowance(from, to);
        if allowance < amount || self.balance_of(from) < amount {
            return false;
        }
        *self.allowances.get_mut(from).unwrap().get_mut(to).unwrap() -= amount;
        *self.balances.entry(*from).or_insert(0) -= amount;
        *self.balances.entry(*to).or_insert(0) += amount;
        true
    }

    pub fn mint(&mut self, to: &PeerId, amount: u64) {
        *self.balances.entry(*to).or_insert(0) += amount;
        self.total_supply += amount;
    }

    pub fn burn(&mut self, from: &PeerId, amount: u64) -> bool {
        if self.balance_of(from) < amount {
            return false;
        }
        *self.balances.entry(*from).or_insert(0) -= amount;
        self.total_supply -= amount;
        true
    }
}
