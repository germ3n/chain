use {
    crate::crypto::{keypair::{PublicKey}, hash::{Hashable, HashAlgorithm, Hash}},
    anyhow::Error,  
    bincode::{Encode, Decode},
};

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Address {
    Null = 0,
    Ed25519([u8; 32]) = 1,
    EdBlake([u8; 32]) = 2,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct Account {
    pub nonce: u64,
    pub balance: u128,
}

impl Account {
    pub fn new() -> Self {
        Self { 
            nonce: 0,
            balance: 0,
        }
    }

    pub fn address_from_pubkey(pubkey: &PublicKey) -> Result<Address, Error> {
        match pubkey {
            PublicKey::Ed25519(pubkey) => Ok(Address::Ed25519(match pubkey.hash(HashAlgorithm::Blake3_256) {
                Hash::Blake3_256(hash) => hash.as_slice().try_into().expect("Invalid pubkey"),
                _ => return Err(Error::msg("Invalid pubkey")),
            })),
            PublicKey::EdBlake(pubkey) => Ok(Address::EdBlake(match pubkey.hash(HashAlgorithm::Blake3_256) {
                Hash::Blake3_256(hash) => hash.as_slice().try_into().expect("Invalid pubkey"),
                _ => return Err(Error::msg("Invalid pubkey")),
            })),
            _ => Err(Error::msg("Invalid pubkey")),
        }
    }

    pub fn increment_nonce(&mut self) -> u64 {
        self.nonce += 1;
        self.nonce
    }

    pub fn transfer(&mut self, amount: u128) -> Result<u128, Error> {
        if self.balance < amount {
            return Err(Error::msg("Insufficient balance"));
        }
        let old_balance = self.balance;
        self.balance -= amount;
        Ok(old_balance)
    }

    pub fn receive(&mut self, amount: u128) -> Result<u128, Error> {
        if amount == 0 {
            return Err(Error::msg("Amount must be greater than 0"));
        }
        let old_balance = self.balance;
        self.balance += amount;
        Ok(old_balance)
    }
}

impl Address {
    pub fn null() -> Self {
        Self::Null
    }
}