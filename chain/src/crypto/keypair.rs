#![allow(non_camel_case_types)]

use {
    ed25519_dalek::{
        SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey, Verifier as Ed25519Verifier, Signer as Ed25519Signer, Signature as Ed25519Signature,
    },
    ed25519_dalek_blake3::{
        Signature as EdBlakeSignature, Verifier as EdBlakeVerifier, PublicKey as EdBlakePublicKey, Keypair as EdBlakeKeypair, ExpandedSecretKey as EdBlakeExpandedSecretKey
    },
    secp256k1::{
        SecretKey as Secp256k1SecretKey, PublicKey as Secp256k1PublicKey, Message as Secp256k1Message,
        ecdsa::Signature as EcdsaSignature, SECP256K1,
    },
    rand_core::OsRng,
    anyhow::Error,
    bincode::{Decode, Encode},
    crate::serialize::binary::{SerializableBytes, DeserializableBytes},
    crate::crypto::hash::{Hash, HashAlgorithm},
    crate::structs::account::{Account, Address},
    serde::{Serialize, Deserialize},
};

#[repr(u8)]
pub enum Keypair {
    Null = 0,
    #[cfg(feature = "ed25519")]
    Ed25519(Ed25519SigningKey, Ed25519VerifyingKey) = 1,
    #[cfg(feature = "edblake")]
    EdBlake(EdBlakeExpandedSecretKey, EdBlakePublicKey) = 2,
    #[cfg(feature = "secp256k1")]
    Secp256k1_Ecdsa(Secp256k1SecretKey, Secp256k1PublicKey) = 3,
    #[cfg(feature = "secp256k1")]
    Secp256k1_Schnorr(Secp256k1SecretKey, Secp256k1PublicKey) = 4,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PublicKey {
    Null = 0,
    #[cfg(feature = "ed25519")]
    Ed25519([u8; 32]) = 1,
    #[cfg(feature = "edblake")]
    EdBlake([u8; 32]) = 2,
    #[cfg(feature = "secp256k1")]
    Secp256k1_Ecdsa([u8; 32]) = 3,
    #[cfg(feature = "secp256k1")]
    Secp256k1_Schnorr([u8; 32]) = 4,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Eq, Hash)]
#[repr(u8)]
pub enum Signature {
    Null = 0,
    #[cfg(feature = "ed25519")]
    Ed25519([u8; 64]) = 1,
    #[cfg(feature = "edblake")]
    EdBlake([u8; 64]) = 2,
    #[cfg(feature = "secp256k1")]
    Secp256k1_Ecdsa([u8; 64]) = 3,
    #[cfg(feature = "secp256k1")]
    Secp256k1_Schnorr([u8; 64]) = 4,
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("0x{}", self))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Signature::from_bytes(s.as_bytes()).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl Keypair {
    pub fn null() -> Self {
        Keypair::Null
    }

    #[cfg(feature = "ed25519")]
    pub fn new_ed25519() -> Result<Self, Error> {
        let mut csprng = OsRng;
        let signing_key = Ed25519SigningKey::generate(&mut csprng);
        let verifying_key = Ed25519VerifyingKey::from(&signing_key);
        Ok(Keypair::Ed25519(signing_key, verifying_key))
    }

    #[cfg(feature = "edblake")]
    pub fn new_edblake() -> Result<Self, Error> {
        let mut csprng = OsRng;
        let keypair = EdBlakeKeypair::generate(&mut csprng);
        Ok(Keypair::EdBlake(EdBlakeExpandedSecretKey::from(&keypair.secret), keypair.public))
    }

    #[cfg(feature = "secp256k1")]
    pub fn new_secp256k1_ecdsa() -> Result<Self, Error> {
        //let mut csprng = OsRng;
        //let (secret_key, public_key) = secp256k1_generate_keypair(&mut csprng);
        //Keypair::Secp256k1(secret_key, public_key)
        unimplemented!("Secp256k1_Ecdsa not implemented")
    }

    #[cfg(feature = "secp256k1")]
    pub fn new_secp256k1_schnorr() -> Result<Self, Error> {
        unimplemented!("Secp256k1_Schnorr not implemented")
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature, Error> {
        match self {
            Keypair::Null => Err(Error::msg("Null keypair")),
            #[cfg(feature = "ed25519")]
            Keypair::Ed25519(signing_key, _) => {
                let signature = Signature::Ed25519(signing_key.sign(message).to_bytes());
                //signature.verify(self, message)?;
                Ok(signature)
            }
            #[cfg(feature = "edblake")]
            Keypair::EdBlake(_signing_key, _verifying_key) => {
                //let signature = Signature::EdBlake(signing_key.sign(message, verifying_key).to_bytes());
                //signature.verify(self, message)?;
                //Ok(signature)
                unimplemented!("EdBlake not implemented")
            }
            #[cfg(feature = "secp256k1")]
            Keypair::Secp256k1_Ecdsa(signing_key, _) => {
                let hash = Hash::with_hasher(HashAlgorithm::Blake3_256, |hasher: &mut dyn std::io::Write| {
                    hasher.write(message).unwrap();
                });
                let message = Secp256k1Message::from_digest(
                    match hash {
                        Hash::Blake3_256(hash) => hash,
                        _ => return Err(Error::msg("Invalid hash")),
                    }
                );
                let signature = SECP256K1.sign_ecdsa(message, &signing_key);
                Ok(Signature::Secp256k1_Ecdsa(signature.serialize_der().to_vec().try_into().unwrap()))
            }
            #[cfg(feature = "secp256k1")]
            Keypair::Secp256k1_Schnorr(_, _) => {
                unimplemented!("Secp256k1_Schnorr not implemented")
            }
        }
    }

    pub fn pubkey(&self) -> Result<PublicKey, Error> {
        match self {
            Keypair::Null => Ok(PublicKey::Null),
            #[cfg(feature = "ed25519")]
            Keypair::Ed25519(_, verifying_key) => Ok(PublicKey::Ed25519(verifying_key.to_bytes())),
            #[cfg(feature = "edblake")]
            Keypair::EdBlake(_, verifying_key) => Ok(PublicKey::EdBlake(verifying_key.to_bytes())),
            #[cfg(feature = "secp256k1")]
            Keypair::Secp256k1_Ecdsa(_, _) => unimplemented!("Secp256k1_Ecdsa not implemented"),
            #[cfg(feature = "secp256k1")]
            Keypair::Secp256k1_Schnorr(_, _) => unimplemented!("Secp256k1_Schnorr not implemented"),
        }
    }

    pub fn address(&self) -> Result<Address, Error> {
        Account::address_from_pubkey(&self.pubkey()?)
    }
}

impl Signature {
    pub fn null() -> Self {
        Signature::Null
    }

    pub fn verify(&self, keypair: &Keypair, message: &[u8]) -> Result<(), Error> {
        match keypair {
            Keypair::Null => Err(Error::msg("Null keypair")),
            #[cfg(feature = "ed25519")]
            Keypair::Ed25519(_, verifying_key) => {
                match self {
                    Signature::Ed25519(signature) => {
                        verifying_key.verify(message, &Ed25519Signature::from_bytes(signature))?;
                        Ok(())
                    }
                    _ => Err(Error::msg("Invalid signature")),
                }
            }
            #[cfg(feature = "edblake")]
            Keypair::EdBlake(_, verifying_key) => {
                match self {
                    Signature::EdBlake(signature) => {
                        verifying_key.verify(message, &EdBlakeSignature::from_bytes(signature)?)?;
                        Ok(())
                    }
                    _ => Err(Error::msg("Invalid signature")),
                }
            }
            #[cfg(feature = "secp256k1")]
            Keypair::Secp256k1_Ecdsa(_, verifying_key) => {
                match self {
                    Signature::Secp256k1_Ecdsa(signature) => {
                        //let public_key  = Secp256k1PublicKey::from_slice(verifying_key).unwrap();
                        let message = Secp256k1Message::from_digest(message.try_into().unwrap());
                        let signature = EcdsaSignature::from_der(signature).unwrap();

                        SECP256K1.verify_ecdsa(
                            message,
                            &signature,
                            &verifying_key
                        )?;
                        Ok(())
                    }
                    _ => Err(Error::msg("Invalid signature")),
                }
            }
            #[cfg(feature = "secp256k1")]
            Keypair::Secp256k1_Schnorr(_, _) => {
                unimplemented!("Secp256k1_Schnorr not implemented")
            }
        }
    }
}

impl PublicKey {
    pub fn null() -> Self {
        PublicKey::Null
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), Error> {
        match self {
            PublicKey::Null => Err(Error::msg("Null public key")),
            #[cfg(feature = "ed25519")]
            PublicKey::Ed25519(pubkey) => {
                match signature {
                    Signature::Ed25519(signature) => {
                        let verifying_key = Ed25519VerifyingKey::from_bytes(pubkey).unwrap();
                        verifying_key.verify(message, &Ed25519Signature::from_bytes(signature)).map_err(|e| Error::msg(format!("Invalid signature: {}", e)))
                    }
                    _ => Err(Error::msg("Invalid signature")),
                }
            }
            #[cfg(feature = "edblake")]
            PublicKey::EdBlake(pubkey) => {
                match signature {
                    &Signature::EdBlake(signature) => {
                        let verifying_key = EdBlakePublicKey::from_bytes(pubkey).unwrap();
                        verifying_key.verify(message, &EdBlakeSignature::from_bytes(&signature)?).map_err(|e| Error::msg(format!("Invalid signature: {}", e)))
                    }
                    _ => Err(Error::msg("Invalid signature")),
                }
            }
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1_Ecdsa(_pubkey) => {
                unimplemented!("Secp256k1_Ecdsa not implemented")
            }
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1_Schnorr(_pubkey) => {
                unimplemented!("Secp256k1_Schnorr not implemented")
            }
        }
    }
}

impl SerializableBytes for Keypair {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        match self {
            Keypair::Null => Ok(vec![]),
            Keypair::Ed25519(signing_key, verifying_key) => {
                let mut bytes = Vec::with_capacity(signing_key.to_bytes().len() + verifying_key.to_bytes().len() + 1);
                bytes.push(Keypair::Ed25519 as u8);
                bytes.extend_from_slice(&signing_key.to_bytes());
                bytes.extend_from_slice(&verifying_key.to_bytes());
                Ok(bytes)
            }
            //Keypair::EdBlake(key) => Ok(key.to_bytes().to_vec()),
            //Keypair::Secp256k1_Ecdsa(key) => Ok(key.serialize().to_vec()),
            //Keypair::Secp256k1_Schnorr(key) => Ok(key.serialize().to_vec()),
            _ => Err(Error::msg("Invalid keypair")),
        }
    }
}

impl DeserializableBytes for Keypair {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        match bytes[0] {
            0/*Keypair::Null as u8*/ => Ok(Keypair::Null),
            1/*Keypair::Ed25519 as u8*/ => Ok(Keypair::Ed25519(Ed25519SigningKey::from_bytes(&bytes[1..33].try_into().expect("Invalid signing key")), Ed25519VerifyingKey::from_bytes(&bytes[33..65].try_into().expect("Invalid verifying key"))?)),
            _ => Err(Error::msg("Invalid keypair")),
        }
    }
}

pub trait Signable {
    fn sign(&self, keypair: &Keypair) -> Result<Signature, Error>;
}

impl<T: SerializableBytes + SignableBytes> Signable for T {
    fn sign(&self, keypair: &Keypair) -> Result<Signature, Error> {
        keypair.sign(self.to_signable_bytes()?.as_slice())
    }
}

pub trait SignableBytes {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, Error>;
}

/*impl<T: SerializableBytes> SignableBytes for T {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, Error> {
        self.to_bytes()
    }
}*/

impl std::fmt::Display for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keypair::Null => write!(f, "Keypair(Null)"),
            #[cfg(feature = "ed25519")]
            Keypair::Ed25519(_, verifying_key) => write!(f, "Keypair(Ed25519, 0x{})", hex::encode(verifying_key.to_bytes())),
            #[cfg(feature = "edblake")]
            Keypair::EdBlake(_, verifying_key) => write!(f, "Keypair(EdBlake, 0x{})", hex::encode(verifying_key.to_bytes())),
            #[cfg(feature = "secp256k1")]
            Keypair::Secp256k1_Ecdsa(_, verifying_key) => write!(f, "Keypair(Secp256k1_Ecdsa, 0x{})", hex::encode(verifying_key.serialize())),
            #[cfg(feature = "secp256k1")]
            Keypair::Secp256k1_Schnorr(_, verifying_key) => write!(f, "Keypair(Secp256k1_Schnorr, 0x{})", hex::encode(verifying_key.serialize())),
        }
    }
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicKey::Null => write!(f, "PublicKey(Null)"),
            #[cfg(feature = "ed25519")]
            PublicKey::Ed25519(pubkey) => write!(f, "PublicKey(Ed25519, 0x{})", hex::encode(pubkey)),
            #[cfg(feature = "edblake")]
            PublicKey::EdBlake(pubkey) => write!(f, "PublicKey(EdBlake, 0x{})", hex::encode(pubkey)),
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1_Ecdsa(pubkey) => write!(f, "PublicKey(Secp256k1_Ecdsa, 0x{})", hex::encode(pubkey)),
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1_Schnorr(pubkey) => write!(f, "PublicKey(Secp256k1_Schnorr, 0x{})", hex::encode(pubkey)),
        }
    }
}

impl std::fmt::Display for Signature {  
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Signature::Null => write!(f, "Signature(Null)"),
            #[cfg(feature = "ed25519")]
            Signature::Ed25519(signature) => write!(f, "Signature(Ed25519, 0x{})", hex::encode(signature)),
            #[cfg(feature = "edblake")]
            Signature::EdBlake(signature) => write!(f, "Signature(EdBlake, 0x{})", hex::encode(signature)),
            #[cfg(feature = "secp256k1")]
            Signature::Secp256k1_Ecdsa(signature) => write!(f, "Signature(Secp256k1_Ecdsa, 0x{})", hex::encode(signature)),
            #[cfg(feature = "secp256k1")]
            Signature::Secp256k1_Schnorr(signature) => write!(f, "Signature(Secp256k1_Schnorr, 0x{})", hex::encode(signature)),
        }
    }
}

impl std::cmp::PartialEq for Keypair {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Keypair::Null, Keypair::Null) => true,
            #[cfg(feature = "ed25519")]
            (Keypair::Ed25519(_, verifying_key), Keypair::Ed25519(_, other_verifying_key)) => {
                verifying_key.to_bytes() == other_verifying_key.to_bytes()
            }
            #[cfg(feature = "edblake")]
            (Keypair::EdBlake(_, verifying_key), Keypair::EdBlake(_, other_verifying_key)) => {
                verifying_key.to_bytes() == other_verifying_key.to_bytes()
            }
            _ => false,
        }
    }
}