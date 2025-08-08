use {
    bincode::{Encode, Decode},
    crate::crypto::hash::Hash,
    std::sync::{RwLock},
    crate::vm::contract::ContractValueType,
    crate::structs::account::Address,
};

#[derive(Debug, PartialEq, Encode, Decode, Eq, Hash)]
#[repr(u32)]
pub enum Event {
    NewBlock(Hash) = 0x00000001,
    NewTx(Hash) = 0x00000002,
    CustomTxEvent((Hash, Address, String, Vec<(String, ContractValueType)>)) = 0x00000003, // (tx_hash, contract_address, event_name, (key, value))
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