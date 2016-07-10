use std::io;
use multihash::MultiHash;

use key::{ RSAPrivKey, RSAPubKey };

#[derive(Debug)]
pub struct HostId {
    hash: MultiHash,
    key: RSAPrivKey,
}

impl HostId {
    pub fn new(hash: MultiHash, key: RSAPrivKey) -> Result<HostId, ()> {
        let key_bytes = try!(key.pub_key().to_bytes().map_err(|_| ()));
        if Some(Ok(true)) != hash.validate(key_bytes) {
            return Err(());
        }

        Ok(HostId {
            hash: hash,
            key: key,
        })
    }

    pub fn generate() -> io::Result<HostId> {
        HostId::from_key(RSAPrivKey::generate())
    }

    pub fn from_key(key: RSAPrivKey) -> io::Result<HostId> {
        let key_bytes = try!(key.pub_key().to_bytes());
        Ok(HostId {
            hash: MultiHash::generate(key_bytes),
            key: key,
        })
    }

    pub fn pub_key(&self) -> &RSAPubKey {
        self.key.pub_key()
    }
}
