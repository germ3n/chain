use {
    crate::{ctx}, 
    bincode::{Decode, Encode}, 
    crate::crypto::keypair::{Keypair, PublicKey, Signature, SignableBytes},
    anyhow::Error,
    crate::serialize::binary::to_bytes,
    serde::{Serialize, Deserialize},
    crate::crypto::hash::HashableBytes,
    std::borrow::Cow,
    crate::currency::Amount,
    cutedogs::document,
};

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct TxHeader {
    pub signature: Option<Signature>,
    pub version: TxVersion,
    pub sender: PublicKey,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct Tx {
    pub header: TxHeader,
    pub data: TxData,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
#[repr(u16)]
pub enum TxData {
    Coinbase(TxCoinbaseData) = 0,
    Batch(Vec<Tx>) = 1,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct TxCoinbaseData {
    pub amount: Amount,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, PartialOrd, Ord, Eq, Hash)]
#[repr(u32)]
pub enum TxVersion {
    V1 = 0,
}

impl Tx {
    #[document(summary = "Creates a coinbase transaction", returns = "The coinbase transaction")]
    pub fn coinbase() -> Self {
        Self {
            header: TxHeader {
                sender: ctx!(miner_keypair).read().expect("Failed to read miner_keypair").as_ref().expect("Invalid keypair").pubkey().expect("Invalid pubkey"),
                version: TxVersion::V1,
                signature: None,
            },
            data: TxData::Coinbase(TxCoinbaseData {
                amount: Amount::U128(ctx!().calc_block_reward()),
            }),
        }
    }

    #[document(summary = "Signs a transaction", params = {
        "keypair": "The keypair to sign the transaction with"
    }, returns = "The result of the signing")]
    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), Error> {
        self.header.signature = Some(keypair.sign(self.to_signable_bytes()?.as_slice())?);
        //self.header.signature.as_ref().unwrap().verify(&keypair, self.to_bytes()?.as_slice())?;
        Ok(())
    }

    #[document(summary = "Verifies a transaction", params = {
        "keypair": "The keypair to verify the transaction with"
    }, returns = "The result of the verification")]
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
    fn get_hashable_bytes(&self) -> Result<Cow<[u8]>, Error> {
        let bytes = to_bytes(&(
            &self.header.sender, 
            &self.header.version, 
            &self.data
        ))?;
        Ok(Cow::Owned(bytes))
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
            TxData::Batch(txs) => write!(f, "TxData::Batch({})", txs.len()),
        }
    }
}

impl std::fmt::Display for TxCoinbaseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TxCoinbaseData {{ amount: {} }}", self.amount)
    }
}