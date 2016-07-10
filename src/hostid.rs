use std::io;
use multihash::MultiHash;
use ring::rand::SecureRandom;

use key::{ RSAPrivKey, RSAPubKey };

#[derive(Debug)]
pub struct HostId {
    hash: MultiHash,
    key: RSAPrivKey,
}

impl HostId {
    pub fn new(hash: MultiHash, key: RSAPrivKey) -> Result<HostId, ()> {
        let key_bytes = try!(key.pub_key().to_protobuf().map_err(|_| ()));
        if Some(Ok(true)) != hash.validate(key_bytes) {
            return Err(());
        }

        Ok(HostId {
            hash: hash,
            key: key,
        })
    }

    pub fn from_der(priv_bytes: Vec<u8>, pub_bytes: Vec<u8>) -> io::Result<HostId> {
        Ok(HostId {
            hash: MultiHash::generate(&pub_bytes),
            key: try!(RSAPrivKey::from_der(priv_bytes, pub_bytes)),
        })
    }

    pub fn sign(&self, rand: &SecureRandom, bytes: &[u8]) -> io::Result<Vec<u8>> {
        self.key.sign(rand, bytes)
    }

    pub fn pub_key(&self) -> &RSAPubKey {
        self.key.pub_key()
    }
}
