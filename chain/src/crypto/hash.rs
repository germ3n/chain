use {
    bincode::{Decode, Encode},
    blake3::Hasher as Blake3Hasher,
    crate::serialize::binary::SerializableBytes,
};

pub enum HashAlgorithm {
    #[cfg(feature = "blake3")]
    Blake3_256,
    #[cfg(feature = "blake3")]
    Blake3_512,
    #[cfg(feature = "sha2")]
    Sha256,
    #[cfg(feature = "sha3")]
    Sha3,
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, Hash)]
pub enum Hash {
    Null,
    #[cfg(feature = "blake3")]
    Blake3_256([u8; 32]),
    #[cfg(feature = "blake3")]
    Blake3_512([u8; 64]),
    #[cfg(feature = "sha2")]
    Sha256([u8; 32]),
    #[cfg(feature = "sha3")]
    Sha512([u8; 64]),
}

impl Hash {
    pub fn null() -> Self {
        Self::Null
    }

    pub fn with_hasher(algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Self {
        match algorithm {
            #[cfg(feature = "blake3")]
            HashAlgorithm::Blake3_256 => {
                let mut hasher = Blake3Hasher::new();
                callback(&mut hasher);
                Self::Blake3_256(*hasher.finalize().as_bytes())
            }
            #[cfg(feature = "blake3")]
            HashAlgorithm::Blake3_512 => {
                panic!("Blake3_512 not implemented");
            }
            #[cfg(feature = "sha2")]
            HashAlgorithm::Sha256 => {
                panic!("Sha256 not implemented");
            }
            #[cfg(feature = "sha3")]
            HashAlgorithm::Sha3 => {
                panic!("Sha3 not implemented");
            }
        }
    }

    pub fn from_bytes(algorithm: HashAlgorithm, bytes: &[u8]) -> Self {
        match algorithm {
            #[cfg(feature = "blake3")]
            HashAlgorithm::Blake3_256 => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(bytes);
                Self::Blake3_256(*hasher.finalize().as_bytes())
            }
            #[cfg(feature = "blake3")]
            HashAlgorithm::Blake3_512 => {
                panic!("Blake3_512 not implemented");
            }
            #[cfg(feature = "sha2")]
            HashAlgorithm::Sha256 => {
                panic!("Sha256 not implemented");
            }
            #[cfg(feature = "sha3")]
            HashAlgorithm::Sha3 => {
                panic!("Sha3 not implemented");
            }
        }
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hash::Null => write!(f, "null"),
            #[cfg(feature = "blake3")]
            Hash::Blake3_256(hash) => write!(f, "{}", hex::encode(hash)),
            #[cfg(feature = "blake3")]
            Hash::Blake3_512(hash) => write!(f, "{}", hex::encode(hash)),
            #[cfg(feature = "sha2")]
            Hash::Sha256(hash) => write!(f, "{}", hex::encode(hash)),
            #[cfg(feature = "sha3")]
            Hash::Sha512(hash) => write!(f, "{}", hex::encode(hash)),
        }
    }
}

pub trait Hashable {
    fn hash(&self, algorithm: HashAlgorithm) -> Hash;
    fn hash_with_callback(&self, algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Hash;
}

impl<T: SerializableBytes> Hashable for T
where T: Sized {
    fn hash(&self, algorithm: HashAlgorithm) -> Hash {
        Hash::with_hasher(algorithm, |hasher: &mut dyn std::io::Write| {
            hasher.write(self.to_bytes().expect("Failed to encode hashable").as_slice()).expect("Failed to write to hasher");
        })
    }

    fn hash_with_callback(&self, algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Hash {
        Hash::with_hasher(algorithm, |hasher: &mut dyn std::io::Write| {
            //hasher.write(self.to_bytes().expect("Failed to encode hashable").as_slice()).expect("Failed to write to hasher");
            callback(hasher);
        })
    }
}