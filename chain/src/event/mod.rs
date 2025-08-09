use {
    bincode::{Encode, Decode},
    crate::crypto::hash::Hash,
    std::sync::{RwLock},
    crate::vm::contract::ContractValueType,
    crate::structs::account::Address,
    crate::currency::Amount,
    std::collections::HashMap,
};

#[derive(Debug, PartialEq, Encode, Decode, Eq)]
#[repr(u32)]
pub enum Event {
    NewBlock(Hash) = 0xB0000010,
    NewTx(Hash) = 0xB0000020,

    TokenMinted(Address, Address, Address, Amount) = 0xC0000010, // (token_address, from_address, to_address, amount)
    TokenBurned(Address, Address, Address, Amount) = 0xC0000020, // (token_address, from_address, to_address, amount)
    CustomTxEvent((Hash, Address, String, HashMap<String, ContractValueType>)) = 0xC00000F0, // (tx_hash, contract_address, event_name, (key => value))
}

pub trait EventManagerTrait {
    fn new() -> Self;
    fn add_event(&self, event: Event);
}

pub struct EventManager {
    events: RwLock<Vec<Event>>,
}

impl EventManagerTrait for EventManager {
    fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }

    fn add_event(&self, event: Event) {
        self.events.write().unwrap().push(event);
    }
}