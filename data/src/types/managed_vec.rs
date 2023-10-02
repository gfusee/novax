use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::codec::{NestedEncode, TopEncodeMulti};
use multiversx_sc::types::{ManagedVec, ManagedVecItem};
use multiversx_sc_scenario::api::StaticApi;
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

impl<M: ManagedTypeApi, T: NativeConvertible + ManagedVecItem> NativeConvertible for ManagedVec<M, T> {
    type Native = Vec<T::Native>;

    fn to_native(&self) -> Self::Native {
        self.into_iter().map(|e| e.to_native()).collect()
    }
}

impl<C, T> ManagedConvertible<ManagedVec<StaticApi, T>> for Vec<C>
where
    C: ManagedConvertible<T>,
    T: TopEncodeMulti + NestedEncode + ManagedVecItem
{
    fn to_managed(&self) -> ManagedVec<StaticApi, T> {
        let mut result = ManagedVec::new();
        for convertible_value in self {
            result.push(convertible_value.to_managed());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::{ManagedBuffer, ManagedVec};
    use multiversx_sc_scenario::api::StaticApi;
    use crate::types::managed::ManagedConvertible;
    use crate::types::native::NativeConvertible;

    #[test]
    fn test_managed_vec_to_native() {
        let mut managed_vec: ManagedVec<StaticApi, ManagedBuffer<StaticApi>> = ManagedVec::new();

        managed_vec.push(ManagedBuffer::from("first"));
        managed_vec.push(ManagedBuffer::from("second"));
        managed_vec.push(ManagedBuffer::from("third"));

        let native = managed_vec.to_native();
        let expected = vec![
            String::from("first"),
            String::from("second"),
            String::from("third")
        ];

        assert_eq!(
            native,
            expected
        )
    }

    #[test]
    fn test_vec_to_managed() {
        let vec = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string()
        ];


        let mut expected_managed_vec: ManagedVec<StaticApi, ManagedBuffer<StaticApi>> = ManagedVec::new();

        expected_managed_vec.push(ManagedBuffer::from("first"));
        expected_managed_vec.push(ManagedBuffer::from("second"));
        expected_managed_vec.push(ManagedBuffer::from("third"));

        let result: ManagedVec<StaticApi, ManagedBuffer<StaticApi>> = vec.to_managed();

        assert_eq!(
            result,
            expected_managed_vec
        )
    }
}