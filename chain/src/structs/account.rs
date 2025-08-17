use {
    crate::crypto::{keypair::{PublicKey}, hash::{Hashable, HashAlgorithm, Hash}},
    anyhow::Error,  
    bincode::{Encode, Decode},
    crate::crypto::hash::HashableBytes,
    std::borrow::Cow,
    cutedogs::document,
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
    #[document(summary = "Creates a new account", returns = "The new account")]
    pub fn new() -> Self {
        Self { 
            nonce: 0,
            balance: 0,
        }
    }

    #[document(summary = "Creates an address from a public key", params = {
        "pubkey": "The public key to create an address from"
    }, returns = "The address")]
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

    #[document(summary = "Increments the nonce", returns = "The new nonce")]
    pub fn increment_nonce(&mut self) -> u64 {
        self.nonce += 1;
        self.nonce
    }

    #[document(summary = "Transfers an amount of currency", params = {
        "amount": "The amount of currency to transfer"
    }, returns = "The old balance")]
    pub fn transfer(&mut self, amount: u128) -> Result<u128, Error> {
        if self.balance < amount {
            return Err(Error::msg("Insufficient balance"));
        }
        let old_balance = self.balance;
        self.balance -= amount;
        Ok(old_balance)
    }

    #[document(summary = "Receives an amount of currency", params = {
        "amount": "The amount of currency to receive"
    }, returns = "The old balance")]
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

    pub fn algorithm_id(&self) -> u8 {
        match self {
            Address::Null => 0,
            Address::Ed25519(_) => 1,
            Address::EdBlake(_) => 2,
        }
    }
}

impl HashableBytes for Address {
    fn get_hashable_bytes(&self) -> Result<Cow<[u8]>, Error> {
        match self {
            Address::Ed25519(pubkey) => Ok(Cow::Borrowed(pubkey)),
            Address::EdBlake(pubkey) => Ok(Cow::Borrowed(pubkey)),
            _ => Err(Error::msg("Invalid address")),
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::Ed25519(address) => write!(f, "0x{}", hex::encode(address)),
            Address::EdBlake(address) => write!(f, "0x{}", hex::encode(address)),
            _ => write!(f, "Null"),
        }
    }
}