use {
    crate::{ctx}, 
    bincode::{Decode, Encode}, 
    crate::crypto::keypair::{Keypair, PublicKey, Signature},
    anyhow::Error,
    crate::serialize::Serializable,
};

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct TxHeader {
    pub sender: PublicKey,
    pub signature: Option<Signature>,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Tx {
    pub header: TxHeader,
    pub data: TxData,
}

#[derive(Encode, Decode, PartialEq, Debug)]
#[repr(u16)]
pub enum TxData {
    Coinbase(TxCoinbaseData) = 0,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct TxCoinbaseData {
    pub amount: u128,
}

impl Tx {
    pub fn coinbase() -> Self {
        Self {
            header: TxHeader {
                sender: crate::crypto::keypair::PublicKey::Ed25519([0; 32]), //ctx!(miner_keypair).read().unwrap().as_ref().expect("Invalid keypair").get_pubkey().expect("Invalid pubkey"),
                signature: None,
            },
            data: TxData::Coinbase(TxCoinbaseData {
                amount: ctx!().calc_block_reward(),
            }),
        }
    }

    pub fn sign(&mut self, keypair: Keypair) -> Result<(), Error> {
        self.header.signature = Some(keypair.sign(self.to_bytes()?.as_slice())?);
        self.header.signature.as_ref().unwrap().verify(&keypair, self.to_bytes()?.as_slice())?;
        Ok(())
    }

    pub fn verify(&self, keypair: Keypair) -> Result<(), Error> {
        match &self.header.signature {
            Some(signature) => signature.verify(&keypair, self.to_bytes()?.as_slice()),
            None => return Err(anyhow::Error::msg("No signature")),
        }
    }
}

impl std::fmt::Display for Tx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tx {{ header: {}, data: {} }}", self.header, self.data)
    }
}

impl std::fmt::Display for TxHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.signature {
            Some(signature) => write!(f, "TxHeader {{ sender: {}, signature: {} }}", self.sender, signature),
            None => write!(f, "TxHeader {{ sender: {}, signature: None }}", self.sender),
        }
    }
}

impl std::fmt::Display for TxData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxData::Coinbase(data) => write!(f, "TxData::Coinbase({})", data.amount),
        }
    }
}

impl std::fmt::Display for TxCoinbaseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TxCoinbaseData {{ amount: {} }}", self.amount)
    }
}