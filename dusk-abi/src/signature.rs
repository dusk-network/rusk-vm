use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Compability signature type, with no impls
pub struct Signature([u8; 64]);

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unimplemented!()
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}
