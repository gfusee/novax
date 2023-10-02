use multiversx_sc::api::{ManagedTypeApi};
use multiversx_sc::codec::{TopDecodeMulti, TopEncodeMulti};
use multiversx_sc::types::MultiValueEncoded;
use multiversx_sc_scenario::api::StaticApi;
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

impl<M: ManagedTypeApi, T: NativeConvertible + TopDecodeMulti + Clone> NativeConvertible for MultiValueEncoded<M, T> {
    type Native = Vec<T::Native>;

    fn to_native(&self) -> Self::Native {
        let mut result: Self::Native = vec![];
        for value in self.clone().into_iter() {
            result.push(value.to_native());
        }

        result
    }
}

impl<C, T> ManagedConvertible<MultiValueEncoded<StaticApi, T>> for Vec<C>
where
    C: ManagedConvertible<T>,
    T: TopEncodeMulti
{
    fn to_managed(&self) -> MultiValueEncoded<StaticApi, T> {
        let mut result: MultiValueEncoded<StaticApi, T> = MultiValueEncoded::new();

        for convertible_value in self {
            result.push(convertible_value.to_managed());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::{ManagedBuffer, MultiValueEncoded};
    use multiversx_sc_scenario::api::StaticApi;
    use crate::types::managed::ManagedConvertible;

    #[test]
    fn test_vec_to_multi_value_encoded() {
        let vec = vec![
            String::from("first"),
            String::from("second"),
            String::from("third")
        ];

        let managed: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> = vec.to_managed();

        let mut expected_multi_value_encoded: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> = MultiValueEncoded::new();
        expected_multi_value_encoded.push(ManagedBuffer::from("first"));
        expected_multi_value_encoded.push(ManagedBuffer::from("second"));
        expected_multi_value_encoded.push(ManagedBuffer::from("third"));

        assert_eq!(
            managed,
            expected_multi_value_encoded
        )
    }
}