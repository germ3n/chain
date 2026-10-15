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
    crate::generate_enum_with_type_id,
    crate::structs::account::Address,
    crate::vm::VMType,
    crate::vm::contract::ContractValueType,
};

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct TxHeader {
    pub signature: Option<Signature>,
    pub version: TxVersion,
    pub sender: PublicKey,
    pub recv_chain: Vec<Address>,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct Tx {
    pub header: TxHeader,
    pub data: TxData,
}

generate_enum_with_type_id! {
    u16,

    #[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
    #[repr(u16)]
    pub enum TxData {
        Coinbase(TxCoinbaseData) = 0,
        Batch(Vec<Tx>) = 1,
        ContractCall(TxContractCallData) = 2,
    }
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct TxCoinbaseData {
    pub amount: Amount,
}

#[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct TxContractCallData {
    pub contract_address: Address,
    pub function_name: String,
    pub vm_type: VMType,
    pub args: Vec<ContractValueType>,
}

generate_enum_with_type_id! {
    u32,
    
    #[derive(Encode, Decode, PartialEq, Debug, Serialize, Deserialize, PartialOrd, Ord, Eq, Hash)]
    #[repr(u32)]
    pub enum TxVersion {
        V1 = 0,
    }
}

impl Tx {
    #[document(summary = "Creates a coinbase transaction", returns = "The coinbase transaction")]
    pub fn coinbase() -> Self {
        Self {
            header: TxHeader {
                sender: ctx!(miner_keypair).read().expect("Failed to read miner_keypair").as_ref().expect("Invalid keypair").pubkey().expect("Invalid pubkey"),
                version: TxVersion::V1,
                signature: None,
                recv_chain: vec![],
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
        self.verify_signature(keypair)
    }

    #[document(summary = "Verifies a transaction", params = {
        "keypair": "The keypair to verify the transaction with"
    }, returns = "The result of the verification")]
    pub fn verify_signature(&self, keypair: &Keypair) -> Result<(), Error> {
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
            TxData::ContractCall(data) => write!(f, "TxData::ContractCall({})", data.contract_address),
        }
    }
}

impl std::fmt::Display for TxCoinbaseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TxCoinbaseData {{ amount: {} }}", self.amount)
    }
}

impl std::fmt::Display for TxContractCallData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TxContractCallData {{ contract_address: {}, function_name: {}, vm_type: {}, args: {} }}", self.contract_address, self.function_name, self.vm_type, self.args.len())
    }
}