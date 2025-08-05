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
    std::fmt::Formatter,
    bincode::{Decode, Encode},
    crate::serialize::Serializable,
    crate::crypto::hash::{Hash, HashAlgorithm},
};

#[repr(u8)]
pub enum Keypair {
    Null = 0,
    Ed25519(Ed25519SigningKey, Ed25519VerifyingKey) = 1,
    EdBlake(EdBlakeExpandedSecretKey, EdBlakePublicKey) = 2,
    Secp256k1_Ecdsa(Secp256k1SecretKey, Secp256k1PublicKey) = 3,
    Secp256k1_Schnorr(Secp256k1SecretKey, Secp256k1PublicKey) = 4,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[repr(u8)]
pub enum PublicKey {
    Null = 0,
    Ed25519([u8; 32]) = 1,
    EdBlake([u8; 32]) = 2,
    Secp256k1_Ecdsa([u8; 32]) = 3,
    Secp256k1_Schnorr([u8; 32]) = 4,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[repr(u8)]
pub enum Signature {
    Null = 0,
    Ed25519([u8; 64]) = 1,
    EdBlake([u8; 64]) = 2,
    Secp256k1_Ecdsa([u8; 64]) = 3,
    Secp256k1_Schnorr([u8; 64]) = 4,
}

impl Keypair {
    pub fn null() -> Self {
        Keypair::Null
    }

    pub fn new_ed25519() -> Result<Self, Error> {
        let mut csprng = OsRng;
        let signing_key = Ed25519SigningKey::generate(&mut csprng);
        let verifying_key = Ed25519VerifyingKey::from(&signing_key);
        Ok(Keypair::Ed25519(signing_key, verifying_key))
    }

    pub fn new_edblake() -> Result<Self, Error> {
        let mut csprng = OsRng;
        let keypair = EdBlakeKeypair::generate(&mut csprng);
        Ok(Keypair::EdBlake(EdBlakeExpandedSecretKey::from(&keypair.secret), keypair.public))
    }

    pub fn new_secp256k1_ecdsa() -> Result<Self, Error> {
        //let mut csprng = OsRng;
        //let (secret_key, public_key) = secp256k1_generate_keypair(&mut csprng);
        //Keypair::Secp256k1(secret_key, public_key)
        Err(Error::msg("Not implemented"))
    }

    pub fn new_secp256k1_schnorr() -> Result<Self, Error> {
        Err(Error::msg("Not implemented"))
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature, Error> {
        match self {
            Keypair::Null => Err(Error::msg("Null keypair")),
            Keypair::Ed25519(signing_key, _) => {
                let signature = Signature::Ed25519(signing_key.sign(message).to_bytes());
                signature.verify(self, message)?;
                Ok(signature)
            }
            Keypair::EdBlake(_signing_key, _verifying_key) => {
                //let signature = Signature::EdBlake(signing_key.sign(message, verifying_key).to_bytes());
                //signature.verify(self, message)?;
                //Ok(signature)
                Err(Error::msg("Not implemented"))
            }
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
            Keypair::Secp256k1_Schnorr(_, _) => {
                Err(Error::msg("Not implemented"))
            }
        }
    }

    pub fn get_pubkey(&self) -> Result<PublicKey, Error> {
        match self {
            Keypair::Null => Ok(PublicKey::Null),
            Keypair::Ed25519(_, verifying_key) => Ok(PublicKey::Ed25519(verifying_key.to_bytes())),
            Keypair::EdBlake(_, verifying_key) => Ok(PublicKey::EdBlake(verifying_key.to_bytes())),
            //Keypair::Secp256k1_Ecdsa(_, verifying_key) => PublicKey::Secp256k1_Ecdsa(verifying_key.serialize()),
            //Keypair::Secp256k1_Schnorr(_, verifying_key) => PublicKey::Secp256k1_Schnorr(verifying_key.serialize()),
            _ => Err(Error::msg("Invalid keypair")),
        }
    }
}

impl Signature {
    pub fn null() -> Self {
        Signature::Null
    }

    pub fn verify(&self, keypair: &Keypair, message: &[u8]) -> Result<(), Error> {
        match keypair {
            Keypair::Null => Err(Error::msg("Null keypair")),
            Keypair::Ed25519(_, verifying_key) => {
                match self {
                    Signature::Ed25519(signature) => {
                        verifying_key.verify(message, &Ed25519Signature::from_bytes(signature))?;
                        Ok(())
                    }
                    _ => Err(Error::msg("Invalid signature")),
                }
            }
            Keypair::EdBlake(_, verifying_key) => {
                match self {
                    Signature::EdBlake(signature) => {
                        verifying_key.verify(message, &EdBlakeSignature::from_bytes(signature)?)?;
                        Ok(())
                    }
                    _ => Err(Error::msg("Invalid signature")),
                }
            }
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
            Keypair::Secp256k1_Schnorr(_, _) => {
                Err(Error::msg("Not implemented"))
            }
        }
    }
}

impl PublicKey {
    pub fn null() -> Self {
        PublicKey::Null
    }
}

impl Serializable for Keypair {
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
    fn verify(&self, keypair: &Keypair) -> Result<(), Error>;
}

impl<T: Serializable> Signable for T {
    fn sign(&self, keypair: &Keypair) -> Result<Signature, Error> {
        keypair.sign(self.to_bytes()?.as_slice())
    }

    fn verify(&self, _keypair: &Keypair) -> Result<(), Error> {
        //keypair.verify(self.to_bytes()?.as_slice())
        Err(Error::msg("Not implemented"))
    }
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Keypair::Null => {
                write!(f, "Keypair(Null)")
            }
            Keypair::Ed25519(_, verifying_key) => {
                write!(f, "Keypair(Ed25519, {:?})", verifying_key.to_bytes())
            }
            Keypair::EdBlake(_, verifying_key) => {
                write!(f, "Keypair(EdBlake, {:?})", verifying_key.to_bytes())
            }
            Keypair::Secp256k1_Ecdsa(_, verifying_key) => {
                write!(f, "Keypair(Secp256k1_Ecdsa, {:?})", verifying_key.serialize())
            }
            Keypair::Secp256k1_Schnorr(_, verifying_key) => {
                write!(f, "Keypair(Secp256k1_Schnorr, {:?})", verifying_key.serialize())
            }
        }
    }
}

impl std::cmp::PartialEq for Keypair {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Keypair::Ed25519(_, verifying_key), Keypair::Ed25519(_, other_verifying_key)) => {
                verifying_key.to_bytes() == other_verifying_key.to_bytes()
            }
            (Keypair::EdBlake(_, verifying_key), Keypair::EdBlake(_, other_verifying_key)) => {
                verifying_key.to_bytes() == other_verifying_key.to_bytes()
            }
            _ => false,
        }
    }
}