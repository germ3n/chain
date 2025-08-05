use {
    bincode::{Decode, Encode},
    blake3::Hasher as Blake3Hasher,
    crate::serialize::Serializable,
};

pub enum HashAlgorithm {
    Blake3_256,
    Blake3_512,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum Hash {
    Null,
    Blake3_256([u8; 32]),
    Blake3_512([u8; 64]),
}

impl Hash {
    pub fn null() -> Self {
        Self::Null
    }

    pub fn with_hasher(algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Self {
        match algorithm {
            HashAlgorithm::Blake3_256 => {
                let mut hasher = Blake3Hasher::new();
                callback(&mut hasher);
                Self::Blake3_256(*hasher.finalize().as_bytes())
            }
            HashAlgorithm::Blake3_512 => {
                panic!("Blake3_512 not implemented");
            }
        }
    }
}

pub trait Hashable {
    fn hash(&self, algorithm: HashAlgorithm) -> Hash;
    fn hash_with_callback(&self, algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Hash;
}

impl<T: Serializable> Hashable for T {
    fn hash(&self, algorithm: HashAlgorithm) -> Hash {
        Hash::with_hasher(algorithm, |hasher: &mut dyn std::io::Write| {
            hasher.write(self.to_bytes().expect("Failed to encode hashable").as_slice()).expect("Failed to write to hasher");
        })
    }

    fn hash_with_callback(&self, algorithm: HashAlgorithm, callback: impl FnOnce(&mut dyn std::io::Write)) -> Hash {
        Hash::with_hasher(algorithm, |hasher: &mut dyn std::io::Write| {
            hasher.write(self.to_bytes().expect("Failed to encode hashable").as_slice()).expect("Failed to write to hasher");
            callback(hasher);
        })
    }
}
