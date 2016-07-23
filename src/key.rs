use std::fmt;
use std::io;
use ring::rand::SecureRandom;
use ring::rsa;
use ring::signature::{ self, RSAKeyPair };
use protobuf::{ parse_from_bytes, Message, ProtobufError };
use untrusted::Input;
use ring::der;

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

// from PKIX, see RFC 3280 appendix C.3
fn parse_public_key<'a>(input: Input<'a>) -> Result<(&'a [u8], &'a [u8]), ()> {
    input.read_all((), |input| {
        der::nested(input, der::Tag::Sequence, (), |input| {
            try!(der::nested(input, der::Tag::Sequence, (), |input| {
                try!(der::nested(input, der::Tag::OID, (), |input| {
                    let oid = input.skip_to_end();
                    if oid == Input::from(&[42, 134, 72, 134, 247, 13, 1, 1, 1]) {
                        Ok(())
                    } else {
                        Err(())
                    }
                }));
                try!(der::nested(input, der::Tag::Null, (), |_| Ok(())));
                Ok(())
            }));
            der::nested(input, der::Tag::BitString, (), |input| {
                let unused = try!(input.read_byte());
                if unused > 0 { return Err(()); } // can't be bothered to handle, shouldn't happen
                der::nested(input, der::Tag::Sequence, (), |input| {
                    let n = try!(der::positive_integer(input));
                    let e = try!(der::positive_integer(input));
                    Ok((n.as_slice_less_safe(), e.as_slice_less_safe()))
                })
            })
        })
    })
}

impl RSAPubKey {
    pub fn from_protobuf(bytes: &[u8]) -> io::Result<RSAPubKey> {
        let mut serialized: data::PublicKey = try!(parse_from_bytes(bytes).map_err(pbetio));
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

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> io::Result<()> {
        let pub_key = try!(parse_public_key(Input::from(&self.bytes)).map_err(|_| io::Error::new(io::ErrorKind::Other, "parse public key failed")));
        let result = rsa::verify(
            &rsa::RSA_PKCS1_2048_8192_SHA256_INTERNAL,
            pub_key,
            Input::from(msg),
            Input::from(sig));
        match result {
            Ok(()) => Ok(()),
            Err(()) => Err(io::Error::new(io::ErrorKind::Other, "signature verify failed")),
        }
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
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

impl fmt::Debug for RSAPrivKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RSAPrivKey")
            .field("bytes", &self.bytes.len())
            .field("pub_key", &self.pub_key)
            .finish()
    }
}

impl Clone for RSAPrivKey {
    fn clone(&self) -> RSAPrivKey {
        RSAPrivKey::from_der(self.bytes.clone(), self.pub_key.bytes.clone()).unwrap()
    }
}
