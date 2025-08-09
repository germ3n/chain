use {
    crate::{ctx}, 
    bincode::{Decode, Encode}, 
    crate::crypto::keypair::{Keypair, PublicKey, Signature, SignableBytes},
    anyhow::Error,
    crate::serialize::binary::to_bytes,
    serde::{Serialize, Deserialize},
    crate::crypto::hash::HashableBytes,
};

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize)]
pub struct TxHeader {
    pub signature: Option<Signature>,
    pub version: TxVersion,
    pub sender: PublicKey,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tx {
    pub header: TxHeader,
    pub data: TxData,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize)]
#[repr(u16)]
pub enum TxData {
    Coinbase(TxCoinbaseData) = 0,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize)]
pub struct TxCoinbaseData {
    pub amount: u128,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum TxVersion {
    V1 = 0,
}

impl Tx {
    pub fn coinbase() -> Self {
        Self {
            header: TxHeader {
                sender: ctx!(miner_keypair).read().expect("Failed to read miner_keypair").as_ref().expect("Invalid keypair").pubkey().expect("Invalid pubkey"),
                version: TxVersion::V1,
                signature: None,
            },
            data: TxData::Coinbase(TxCoinbaseData {
                amount: ctx!().calc_block_reward(),
            }),
        }
    }

    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), Error> {
        self.header.signature = Some(keypair.sign(self.to_signable_bytes()?.as_slice())?);
        //self.header.signature.as_ref().unwrap().verify(&keypair, self.to_bytes()?.as_slice())?;
        Ok(())
    }

    pub fn verify(&self, keypair: &Keypair) -> Result<(), Error> {
        match &self.header.signature {
            Some(signature) => signature.verify(&keypair, self.to_signable_bytes()?.as_slice()),
            None => return Err(anyhow::Error::msg("No signature")),
        }
    }
}

impl SignableBytes for Tx {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, Error> {
        to_bytes(&(
            &self.header.sender, 
            &self.header.version, 
            &self.data
        ))
    }
}

impl HashableBytes for Tx {
    fn get_hashable_bytes(&self) -> Result<Vec<u8>, Error> {
        to_bytes(&(
            &self.header.sender, 
            &self.header.version, 
            &self.data
        ))
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