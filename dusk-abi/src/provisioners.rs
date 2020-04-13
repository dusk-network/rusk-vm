use super::impl_serde_for_array;

const PROVISIONERS_SIZE: usize = 32 * 128;

pub struct Provisioners([u8; PROVISIONERS_SIZE]);

impl_serde_for_array!(Provisioners, PROVISIONERS_SIZE);

impl Provisioners {
    pub fn to_bytes(&self) -> [u8; PROVISIONERS_SIZE] {
        self.0
    }

    pub fn from_bytes(bytes: [u8; PROVISIONERS_SIZE]) -> Self {
        Provisioners(bytes)
    }
}

impl Default for Provisioners {
    fn default() -> Self {
        Provisioners([0u8; PROVISIONERS_SIZE])
    }
}

impl core::fmt::Debug for Provisioners {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Provisioners(")?;
        for i in 0..64 {
            write!(f, "{:02x}", self.0[i])?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
