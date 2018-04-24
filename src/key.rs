use std::fmt;
use std::sync::Arc;
use std::io;
use ring::der;
use ring::rand::SystemRandom;
use ring::signature::{ self, RSAKeyPair, RSASigningState };
use ring::signature::primitive::verify_rsa;
use untrusted::Input;
use prost::Message;

use data;

const EXPECTED_OID: &'static [u8] = &[42, 134, 72, 134, 247, 13, 1, 1, 1];

#[derive(Clone)]
pub struct RSAPubKey {
    bytes: Vec<u8>,
}

pub struct RSAPrivKey {
    key: Arc<RSAKeyPair>,
    bytes: Vec<u8>,
    pub_key: RSAPubKey,
}

// from PKIX, see RFC 3280 appendix C.3
fn parse_public_key<'a>(input: Input<'a>) -> Result<(Input<'a>, Input<'a>), ()> {
    input.read_all((), |input| {
        der::nested(input, der::Tag::Sequence, (), |input| {
            try!(der::nested(input, der::Tag::Sequence, (), |input| {
                try!(der::nested(input, der::Tag::OID, (), |input| {
                    let oid = input.skip_to_end();
                    if oid == Input::from(EXPECTED_OID) {
                        Ok(())
                    } else {
                        Err(())
                    }
                }));
                try!(der::nested(input, der::Tag::Null, (), |_| Ok(())));
                Ok(())
            }));
            der::nested(input, der::Tag::BitString, (), |input| {
                let unused = try!(input.read_byte().map_err(|_| ()));
                if unused > 0 { return Err(()); } // can't be bothered to handle, shouldn't happen
                der::nested(input, der::Tag::Sequence, (), |input| {
                    let n = try!(der::positive_integer(input).map_err(|_| ()));
                    let e = try!(der::positive_integer(input).map_err(|_| ()));
                    Ok((n, e))
                })
            })
        })
    })
}

impl RSAPubKey {
    pub fn from_protobuf(bytes: &[u8]) -> io::Result<RSAPubKey> {
        let message = data::PublicKey::decode(bytes)?;
        match message.type_ {
            data::KeyType::RSA => {
                Ok(RSAPubKey { bytes: message.data })
            }
            other => {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("unexpected key type {}", Into::<i32>::into(other))))
            }
        }
    }

    pub fn to_protobuf(&self) -> io::Result<Vec<u8>> {
        let serialized = data::PublicKey {
            type_: data::KeyType::RSA,
            data: self.bytes.clone(),
        };
        let mut bytes = Vec::with_capacity(serialized.encoded_len());
        serialized.encode(&mut bytes)?;
        Ok(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> io::Result<()> {
        let pub_key = try!(parse_public_key(Input::from(&self.bytes)).map_err(|_| io::Error::new(io::ErrorKind::Other, "parse public key failed")));
        let result = verify_rsa(
            &signature::RSA_PKCS1_2048_8192_SHA256,
            pub_key,
            Input::from(msg),
            Input::from(sig));
        match result {
            Ok(()) => Ok(()),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "signature verify failed")),
        }
    }
}

impl RSAPrivKey {
    pub fn from_der(priv_bytes: Vec<u8>, pub_bytes: Vec<u8>) -> io::Result<RSAPrivKey> {
        match RSAKeyPair::from_der(Input::from(&priv_bytes)) {
            Ok(key) => Ok(RSAPrivKey { key: Arc::new(key), bytes: priv_bytes, pub_key: RSAPubKey { bytes: pub_bytes } }),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "failed to parse")),
        }
    }

    pub fn pub_key(&self) -> &RSAPubKey {
        &self.pub_key
    }

    pub fn sign(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let rand = SystemRandom::new();
        let mut sig = vec![0; self.key.public_modulus_len()];
        match RSASigningState::new(self.key.clone()) {
            Ok(mut state) => match state.sign(&signature::RSA_PKCS1_SHA256, &rand, bytes, &mut sig) {
                Ok(()) => Ok(sig),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "failed to sign")),
            },
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "failed to sign")),
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
