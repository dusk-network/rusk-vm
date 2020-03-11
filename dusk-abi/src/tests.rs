// Test correctness of the macro which implements Serialize/Deserialize
// for tuple structs with big arrays.

#[cfg(test)]
mod tests {
    use super::super::{encoding, impl_serde_for_array};

    const LEN_ARRAY: usize = 100;
    struct BigArray([u8; LEN_ARRAY]);

    #[test]
    fn impl_serde() {
        impl_serde_for_array!(BigArray, LEN_ARRAY);

        let arr = BigArray([171u8; LEN_ARRAY]);

        let mut buf = [0u8; LEN_ARRAY];
        encoding::encode(&arr, &mut buf)
            .expect("should be able to encode BigArray");

        let decoded_arr: BigArray =
            encoding::decode(&buf).expect("should be able to decode BigArray");

        for (i, byte) in arr.0.iter().enumerate() {
            assert_eq!(*byte, decoded_arr.0[i]);
        }
    }
}
