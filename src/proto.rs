use proof::lemma_proto::SiblingHash;
use prost::Message;
use ring::digest::Algorithm;

use crate::proof::{Lemma, Positioned, Proof};

pub use self::proof::{LemmaProto, ProofProto};

pub mod proof {
    include!(concat!(env!("OUT_DIR"), "/proof.rs"));
}

impl<T> Proof<T> {
    /// Constructs a `Proof` struct from its Protobuf representation.
    pub fn from_protobuf(algorithm: &'static Algorithm, proto: ProofProto) -> Option<Self>
    where
        T: From<Vec<u8>>,
    {
        proto.into_proof(algorithm)
    }

    /// Encode this `Proof` to its Protobuf representation.
    pub fn into_protobuf(self) -> ProofProto
    where
        T: Into<Vec<u8>>,
    {
        ProofProto::from_proof(self)
    }

    /// Parse a `Proof` from its Protobuf binary representation.
    pub fn parse_from_bytes(
        bytes: &[u8],
        algorithm: &'static Algorithm,
    ) -> Result<Option<Self>, prost::DecodeError>
    where
        T: From<Vec<u8>>,
    {
        proof::ProofProto::decode(bytes).map(|proto| proto.into_proof(algorithm))
    }

    /// Serialize this `Proof` with Protobuf.
    pub fn write_to_bytes(self) -> Result<Vec<u8>, prost::EncodeError>
    where
        T: Into<Vec<u8>>,
    {
        let p = self.into_protobuf();
        let mut buf = Vec::new();
        buf.reserve(p.encoded_len());
        p.encode(&mut buf)?;
        Ok(buf)
    }
}

impl ProofProto {
    pub fn from_proof<T>(proof: Proof<T>) -> Self
    where
        T: Into<Vec<u8>>,
    {
        Self {
            root_hash: proof.root_hash,
            lemma: Some(LemmaProto::from_lemma(proof.lemma)),
            value: proof.value.into(),
        }
    }

    pub fn into_proof<T>(self, algorithm: &'static Algorithm) -> Option<Proof<T>>
    where
        T: From<Vec<u8>>,
    {
        if self.root_hash.is_empty() {
            return None;
        }

        let Some(lemma) = self.lemma.clone() else {
            return None;
        };

        lemma
            .into_lemma()
            .map(|lemma| Proof::new(algorithm, self.root_hash, lemma, self.value.into()))
    }
}

impl LemmaProto {
    pub fn from_lemma(lemma: Lemma) -> Self {
        Self {
            node_hash: lemma.node_hash,
            sub_lemma: lemma.sub_lemma.map(|l| Box::new(Self::from_lemma(*l))),

            sibling_hash: lemma.sibling_hash.map(|sh| match sh {
                Positioned::Left(hash) => SiblingHash::LeftSiblingHash(hash),
                Positioned::Right(hash) => SiblingHash::RightSiblingHash(hash),
            }),
        }
    }

    pub fn into_lemma(self) -> Option<Lemma> {
        if self.node_hash.is_empty() {
            return None;
        }

        let node_hash = self.node_hash;

        let sibling_hash = match self.sibling_hash {
            Some(SiblingHash::LeftSiblingHash(hash)) => Some(Positioned::Left(hash)),
            Some(SiblingHash::RightSiblingHash(hash)) => Some(Positioned::Right(hash)),
            None => None,
        };

        if let Some(sub_lemma) = self.sub_lemma {
            // If a `sub_lemma` is present is the Protobuf,
            // then we expect it to unserialize to a valid `Lemma`,
            // otherwise we return `None`
            sub_lemma.into_lemma().map(|sub_lemma| Lemma {
                node_hash,
                sibling_hash,
                sub_lemma: Some(Box::new(sub_lemma)),
            })
        } else {
            // We might very well not have a sub_lemma,
            // in which case we just set it to `None`,
            // but still return a potentially valid `Lemma`.
            Some(Lemma {
                node_hash,
                sibling_hash,
                sub_lemma: None,
            })
        }
    }
}
