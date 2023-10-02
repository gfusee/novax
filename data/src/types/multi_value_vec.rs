use multiversx_sc_codec::multi_types::MultiValueVec;
use multiversx_sc_codec::{TopDecodeMulti, TopEncodeMulti};
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

impl<T> NativeConvertible for MultiValueVec<T>
where
    T: TopDecodeMulti + NativeConvertible + Clone
{
    type Native = Vec<T::Native>;

    fn to_native(&self) -> Self::Native {
        self.0.clone().into_iter().map(|e| e.to_native()).collect()
    }
}

impl<T, N> ManagedConvertible<MultiValueVec<T>> for Vec<N>
where
    N: ManagedConvertible<T> + Clone,
    T: TopEncodeMulti
{
    fn to_managed(&self) -> MultiValueVec<T> {
        MultiValueVec::from(
            self
                .iter()
                .map(|e| e.to_managed())
                .collect::<Vec<T>>()
        )
    }
}