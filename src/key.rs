use std::fmt;
use std::io;
use openssl::crypto::pkey::PKey;
use protobuf::{ CodedInputStream, Message, ProtobufError };

use data;

#[derive(Clone)]
pub struct RSAPubKey {
    key: PKey,
}

#[derive(Clone)]
pub struct RSAPrivKey {
    key: PKey,
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
    pub fn from_bytes(bytes: &[u8]) -> io::Result<RSAPubKey> {
        let mut serialized = data::PublicKey::new();
        let mut stream = CodedInputStream::from_bytes(bytes);
        try!(serialized.merge_from(&mut stream).map_err(pbetio));
        if serialized.get_Type() != data::KeyType::RSA {
            return Err(io::Error::new(io::ErrorKind::Other, "Only handle rsa pub keys"));
        }
        let mut key = PKey::new();
        key.load_pub(serialized.get_Data());
        Ok(RSAPubKey { key: key })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut serialized = data::PublicKey::new();
        serialized.set_Type(data::KeyType::RSA);
        serialized.set_Data(self.key.save_pub());
        serialized.write_to_bytes().map_err(pbetio)
    }
}

impl RSAPrivKey {
    pub fn generate() -> RSAPrivKey {
        let mut key = PKey::new();
        key.gen(256);
        RSAPrivKey::from_key(key)
    }

    // TODO: Validate the key has a private key
    fn from_key(priv_key: PKey) -> RSAPrivKey {
        let mut pub_key = PKey::new();
        pub_key.load_pub(&priv_key.save_pub());
        RSAPrivKey { key: priv_key, pub_key: RSAPubKey { key: pub_key } }
    }

    pub fn pub_key(&self) -> &RSAPubKey {
        &self.pub_key
    }
}

impl fmt::Debug for RSAPubKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RSAPubKey")
            .field("key", &self.key.get_rsa())
            .finish()
    }
}

impl fmt::Debug for RSAPrivKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RSAPrivKey")
            .field("key", &self.key.get_rsa())
            .finish()
    }
}
