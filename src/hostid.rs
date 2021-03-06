use std::fmt;
use std::io;

use mhash::MultiHash;

use key::{ RSAPrivKey, RSAPubKey };
use peerid::PeerId;

#[derive(Clone)]
pub struct HostId {
    hash: MultiHash,
    key: RSAPrivKey,
}

impl HostId {
    pub fn new(hash: MultiHash, key: RSAPrivKey) -> Result<HostId, ()> {
        let key_bytes = key.pub_key().to_protobuf().map_err(|_| ())?;
        if Some(Ok(true)) != hash.validate(&key_bytes) {
            return Err(());
        }

        Ok(HostId { hash, key })
    }

    pub fn from_der(priv_bytes: Vec<u8>, pub_bytes: Vec<u8>) -> io::Result<HostId> {
        let key = RSAPrivKey::from_der(priv_bytes, pub_bytes)?;
        let hash = MultiHash::generate_sha2_256(&key.pub_key().to_protobuf()?);
        Ok(HostId { hash, key })
    }

    pub fn sign(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        self.key.sign(bytes)
    }

    pub fn pub_key(&self) -> &RSAPubKey {
        self.key.pub_key()
    }

    pub fn hash(&self) -> &MultiHash {
        &self.hash
    }

    pub fn to_peerid(&self) -> PeerId {
        PeerId::Proven {
            hash: self.hash.clone(),
            key: self.pub_key().clone(),
        }
    }
}

impl fmt::Debug for HostId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "HostId(\"{}\")", self.hash)
        } else {
            f.debug_struct("HostId")
                .field("hash", &self.hash)
                .field("key", &self.key)
                .finish()
        }
    }
}
