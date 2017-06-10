use std::fmt::Debug;

#[macro_use]
pub mod macros;
pub mod chunked;
pub mod sha1;
pub mod sha2;

pub trait Digest: Clone + AsRef<[u8]> + Into<Box<[u8]>> + Debug {}

pub trait DigestAlgorithm {
    type Digest: Digest;

    fn new() -> Self;
    fn update(&mut self, input: &[u8]);
    fn digest(self) -> Self::Digest;

    fn compute(input: &[u8]) -> Self::Digest where Self: Sized {
        let mut state = Self::new();
        state.update(input);
        state.digest()
    }
}
