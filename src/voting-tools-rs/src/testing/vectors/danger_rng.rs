use rand::{rngs::StdRng, RngCore, SeedableRng};
use rand_core::CryptoRng;

/// A struct which wraps an RNG and declares that it is cryptographically secure
///
/// Warning - this will be accepted in cryptographically sensitive contexts, regardless of the
/// security properties of the underlying RNG
///
/// TLDR: don't use this in prod code
#[derive(Debug, Clone)]
pub struct DangerRng {
    inner: StdRng
}

impl RngCore for DangerRng {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.inner.try_fill_bytes(dest)
    }
}

impl SeedableRng for DangerRng {
    type Seed = <StdRng as SeedableRng>::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        Self { inner: StdRng::from_seed(seed) }
    }
}

impl CryptoRng for DangerRng {}
