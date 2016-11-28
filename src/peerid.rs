use std::io;
use mhash::MultiHash;

use key::RSAPubKey;

#[derive(Debug, Clone)]
pub enum PeerId {
    Unknown,
    Candidate {
        hash: MultiHash,
    },
    Proven {
        hash: MultiHash,
        key: RSAPubKey,
    }
}

impl PeerId {
    pub fn new(hash: MultiHash, key: RSAPubKey) -> Result<PeerId, ()> {
        let key_bytes = key.to_protobuf().map_err(|_| ())?;
        if Some(Ok(true)) != hash.validate(&key_bytes) {
            return Err(());
        }

        Ok(PeerId::Proven {
            hash: hash,
            key: key,
        })
    }

    pub fn from_protobuf(bytes: &[u8]) -> io::Result<PeerId> {
        RSAPubKey::from_protobuf(bytes).and_then(PeerId::from_key)
    }

    pub fn from_key(key: RSAPubKey) -> io::Result<PeerId> {
        Ok(PeerId::Proven {
            hash: MultiHash::generate_sha2_256(&key.to_protobuf()?),
            key: key,
        })
    }

    pub fn from_hash(hash: MultiHash) -> PeerId {
        PeerId::Candidate {
            hash: hash,
        }
    }

    pub fn hash(&self) -> Option<&MultiHash> {
        Some(match *self {
            PeerId::Unknown => return None,
            PeerId::Proven { ref hash, .. } => hash,
            PeerId::Candidate { ref hash, .. } => hash,
        })
    }

    pub fn proven(&self) -> bool {
        match *self {
            PeerId::Proven { .. } => true,
            _ => false,
        }
    }

    pub fn matches(&self, other: &PeerId) -> bool {
        self.hash() != None && self.hash() == other.hash()
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> io::Result<()> {
        match *self {
            PeerId::Proven { ref key, .. } => key.verify(msg, sig),
            _ => Err(io::Error::new(io::ErrorKind::Other, "no public key")),
        }
    }
}
