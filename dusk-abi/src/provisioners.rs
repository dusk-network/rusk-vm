use super::impl_serde_for_array;

const PROVISIONERS_SIZE: usize = 32 * 128;

pub struct Provisioners(pub [u8; PROVISIONERS_SIZE]);

impl_serde_for_array!(Provisioners, PROVISIONERS_SIZE);

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
        Ok(())
    }
}
