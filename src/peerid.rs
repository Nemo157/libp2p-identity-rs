use std::io;
use multihash::MultiHash;

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
        let key_bytes = try!(key.to_bytes().map_err(|_| ()));
        if Some(Ok(true)) != hash.validate(key_bytes) {
            return Err(());
        }

        Ok(PeerId::Proven {
            hash: hash,
            key: key,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<PeerId> {
        RSAPubKey::from_bytes(bytes).and_then(PeerId::from_key)
    }

    pub fn from_key(key: RSAPubKey) -> io::Result<PeerId> {
        let key_bytes = try!(key.to_bytes());
        Ok(PeerId::Proven {
            hash: MultiHash::generate(key_bytes),
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
}
