use std::fmt;
use std::io;
use ring::rand::SecureRandom;
use ring::signature::{ self, RSAKeyPair };
use protobuf::{ CodedInputStream, Message, ProtobufError };
use untrusted::Input;

use data;

#[derive(Clone)]
pub struct RSAPubKey {
    bytes: Vec<u8>,
}

pub struct RSAPrivKey {
    key: RSAKeyPair,
    bytes: Vec<u8>,
    pub_key: RSAPubKey,
}

fn pbetio(e: ProtobufError) -> io::Error {
    match e {
        ProtobufError::IoError(e) => e,
        ProtobufError::WireError(m) => io::Error::new(io::ErrorKind::Other, m),
        ProtobufError::MessageNotInitialized { message: m } =>
            io::Error::new(io::ErrorKind::Other, m),
    }
}


impl RSAPubKey {
    pub fn from_protobuf(bytes: &[u8]) -> io::Result<RSAPubKey> {
        let mut serialized = data::PublicKey::new();
        let mut stream = CodedInputStream::from_bytes(bytes);
        try!(serialized.merge_from(&mut stream).map_err(pbetio));
        if serialized.get_Type() != data::KeyType::RSA {
            return Err(io::Error::new(io::ErrorKind::Other, "Only handle rsa pub keys"));
        }
        Ok(RSAPubKey { bytes: serialized.take_Data() })
    }

    pub fn to_protobuf(&self) -> io::Result<Vec<u8>> {
        let mut serialized = data::PublicKey::new();
        serialized.set_Type(data::KeyType::RSA);
        serialized.set_Data(self.bytes.clone());
        serialized.write_to_bytes().map_err(pbetio)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl RSAPrivKey {
    pub fn from_der(priv_bytes: Vec<u8>, pub_bytes: Vec<u8>) -> io::Result<RSAPrivKey> {
        match RSAKeyPair::from_der(Input::from(&priv_bytes)) {
            Ok(key) => Ok(RSAPrivKey { key: key, bytes: priv_bytes, pub_key: RSAPubKey { bytes: pub_bytes } }),
            Err(()) => Err(io::Error::new(io::ErrorKind::Other, "failed to parse")),
        }
    }

    pub fn pub_key(&self) -> &RSAPubKey {
        &self.pub_key
    }

    pub fn sign(&self, rand: &SecureRandom, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut sig = vec![0; self.key.public_modulus_len()];
        match self.key.sign(&signature::RSA_PKCS1_SHA256, rand, bytes, &mut sig) {
            Ok(()) => Ok(sig),
            Err(()) => Err(io::Error::new(io::ErrorKind::Other, "failed to sign")),
        }
    }
}

impl fmt::Debug for RSAPubKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RSAPubKey")
            .field("bytes", &self.bytes)
            .finish()
    }
}

impl fmt::Debug for RSAPrivKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RSAPrivKey")
            .field("key", &self.bytes)
            .finish()
    }
}

impl Clone for RSAPrivKey {
    fn clone(&self) -> RSAPrivKey {
        RSAPrivKey::from_der(self.bytes.clone(), self.pub_key.bytes.clone()).unwrap()
    }
}
