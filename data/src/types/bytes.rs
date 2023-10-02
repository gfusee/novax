use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

impl<const N: usize> NativeConvertible for [u8; N] {
    type Native = Vec<u8>; // [u8; N] is not handled by serde for N > 32

    fn to_native(&self) -> Self::Native {
        self.to_vec()
    }
}

impl<const N: usize> ManagedConvertible<[u8; N]> for [u8; N] {
    fn to_managed(&self) -> [u8; N] {
        *self
    }
}