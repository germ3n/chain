use {
    bincode::{Decode, Encode},
    blake3::Hasher as Blake3Hasher,
    crate::serialize::binary::SerializableBytes,
    anyhow::Error,
    xxhash_rust::xxh3::xxh3_64,
    serde::{Serialize, Deserialize},
    std::borrow::Cow,
};

#[cfg(feature = "keccak")]
use {
    sha3::Keccak256,
    sha3::Digest as Keccak256Digest,
};

pub enum HashAlgorithm {
    XXHash3_64,
    #[cfg(feature = "blake3")]
    Blake3_256,
    #[cfg(feature = "blake3")]
    Blake3_512,
    #[cfg(feature = "sha2")]
    Sha256,
    #[cfg(feature = "sha3")]
    Sha3,
    #[cfg(feature = "keccak")]
    Keccak256,
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, Hash)]
#[repr(u8)]
pub enum Hash {
    Null = 0,
    XXHash3_64(u64) = 1,
    #[cfg(feature = "blake3")]
    Blake3_256([u8; 32]) = 2,
    #[cfg(feature = "blake3")]  
    Blake3_512([u8; 64]) = 3,
    #[cfg(feature = "sha2")]
    Sha256([u8; 32]) = 4,
    #[cfg(feature = "sha3")]
    Sha512([u8; 64]) = 5,
    #[cfg(feature = "keccak")]
    Keccak256([u8; 32]) = 6,
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("0x{}", self))
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Hash::from_bytes(HashAlgorithm::Blake3_256, s.as_bytes()))
    }
}

impl Hash {
    pub fn null() -> Self {
        Self::Null
    }

    pub fn with_hasher(algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Self {
        match algorithm {
            HashAlgorithm::XXHash3_64 => {
                unimplemented!("XXHash not implemented");
            }
            #[cfg(feature = "blake3")]
            HashAlgorithm::Blake3_256 => {
                let mut hasher = Blake3Hasher::new();
                callback(&mut hasher);
                Self::Blake3_256(*hasher.finalize().as_bytes())
            }
            #[cfg(feature = "blake3")]
            HashAlgorithm::Blake3_512 => {
                unimplemented!("Blake3_512 not implemented");
            }
            #[cfg(feature = "sha2")]
            HashAlgorithm::Sha256 => {
                unimplemented!("Sha256 not implemented");
            }
            #[cfg(feature = "sha3")]
            HashAlgorithm::Sha3 => {
                unimplemented!("Sha3 not implemented");
            }
            #[cfg(feature = "keccak")]
            HashAlgorithm::Keccak256 => {
                let mut hasher = Keccak256::new();
                callback(&mut hasher);
                Self::Keccak256(hasher.finalize().into())
            }
        }
    }

    pub fn from_bytes(algorithm: HashAlgorithm, bytes: &[u8]) -> Self {
        match algorithm {
            HashAlgorithm::XXHash3_64 => {
                Self::XXHash3_64(xxh3_64(bytes))
            }
            #[cfg(feature = "blake3")]
            HashAlgorithm::Blake3_256 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(bytes);
                Self::Blake3_256(*hasher.finalize().as_bytes())
            }
            #[cfg(feature = "blake3")]
            HashAlgorithm::Blake3_512 => {
                unimplemented!("Blake3_512 not implemented");
            }
            #[cfg(feature = "sha2")]
            HashAlgorithm::Sha256 => {
                unimplemented!("Sha256 not implemented");
            }
            #[cfg(feature = "sha3")]
            HashAlgorithm::Sha3 => {
                unimplemented!("Sha3 not implemented");
            }
            #[cfg(feature = "keccak")]
            HashAlgorithm::Keccak256 => {
                let mut hasher = Keccak256::new();
                hasher.update(bytes);
                Self::Keccak256(hasher.finalize().into())
            }
        }
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hash::Null => write!(f, "null"),
            Hash::XXHash3_64(hash) => write!(f, "{:016x}", hash),
            #[cfg(feature = "blake3")]
            Hash::Blake3_256(hash) => write!(f, "{}", hex::encode(hash)),
            #[cfg(feature = "blake3")]
            Hash::Blake3_512(hash) => write!(f, "{}", hex::encode(hash)),
            #[cfg(feature = "sha2")]
            Hash::Sha256(hash) => write!(f, "{}", hex::encode(hash)),
            #[cfg(feature = "sha3")]
            Hash::Sha512(hash) => write!(f, "{}", hex::encode(hash)),
            #[cfg(feature = "keccak")]
            Hash::Keccak256(hash) => write!(f, "{}", hex::encode(hash)),
        }
    }
}

pub trait Hashable {
    fn hash(&self, algorithm: HashAlgorithm) -> Hash;
    fn hash_with_callback(&self, algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Hash;
    fn hash_bytes(&self, algorithm: HashAlgorithm) -> Vec<u8>;
}

pub trait HashableBytes {
    fn get_hashable_bytes(&self) -> Result<Cow<[u8]>, Error>;
}

impl<T: SerializableBytes + HashableBytes> Hashable for T
where T: Sized {
    fn hash(&self, algorithm: HashAlgorithm) -> Hash {
        let bytes = self.get_hashable_bytes().expect("Failed to encode hashable");
        Hash::from_bytes(algorithm, &bytes)
    }

    fn hash_with_callback(&self, algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Hash {
        Hash::with_hasher(algorithm, |hasher: &mut dyn std::io::Write| {
            callback(hasher);
        })
    }

    fn hash_bytes(&self, algorithm: HashAlgorithm) -> Vec<u8> {
        match self.hash(algorithm) {
            Hash::Blake3_256(hash) => hash.to_vec(),
            _ => vec![],
        }
    }
}

impl<T: AsRef<[u8]>> HashableBytes for T {
    fn get_hashable_bytes(&self) -> Result<Cow<[u8]>, Error> {
        Ok(Cow::Borrowed(self.as_ref()))
    }
}