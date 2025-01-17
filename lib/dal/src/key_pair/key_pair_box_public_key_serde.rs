use base64::{engine::general_purpose, Engine};
use serde::{self, Deserialize, Deserializer, Serializer};
use sodiumoxide::crypto::box_::PublicKey as BoxPublicKey;

use super::encode_public_key;

pub fn serialize<S>(box_public_key: &BoxPublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = encode_public_key(box_public_key);
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<BoxPublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let box_buffer = general_purpose::STANDARD_NO_PAD
        .decode(s)
        .map_err(serde::de::Error::custom)?;

    BoxPublicKey::from_slice(&box_buffer)
        .ok_or_else(|| serde::de::Error::custom("cannot deserialize public key"))
}
