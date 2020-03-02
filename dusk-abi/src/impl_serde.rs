/// Easy way to implement Serialize/Deserialize for types which hold
/// a fixed-size array that's larger than 32 bytes.
/// Takes the type itself, and the length of the contained array as arguments.
#[macro_export]
macro_rules! impl_serde_for_array {
    ($arr:ident, $len:expr) => {
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        impl Serialize for $arr {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                use serde::ser::SerializeTuple;
                let mut seq = serializer.serialize_tuple($len)?;
                for byte in self.0.iter() {
                    seq.serialize_element(byte)?;
                }
                seq.end()
            }
        }

        impl<'de> Deserialize<'de> for $arr {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                use serde::de::Visitor;
                struct DummyVisitor;

                impl<'de> Visitor<'de> for DummyVisitor {
                    type Value = $arr;

                    fn expecting(
                        &self,
                        formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        formatter.write_fmt(format_args!("{} bytes", $len))
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<$arr, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let mut bytes = [0u8; $len];
                        for i in 0..$len {
                            bytes[i] = seq.next_element()?.ok_or(
                                serde::de::Error::invalid_length(
                                    i,
                                    &"invalid length",
                                ),
                            )?;
                        }

                        Ok($arr(bytes))
                    }
                }

                deserializer.deserialize_tuple($len, DummyVisitor)
            }
        }
    };
}
