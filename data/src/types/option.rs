use multiversx_sc_codec::multi_types::OptionalValue;
use multiversx_sc_codec::{NestedDecode, NestedEncode, TopDecode, TopEncode, TopEncodeMulti};
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

impl<T: Clone + NativeConvertible> NativeConvertible for Option<T> {
    type Native = Option<T::Native>;

    fn to_native(&self) -> Self::Native {
        self.clone().map(|e| e.to_native())
    }
}

impl<T: Clone + NativeConvertible> NativeConvertible for OptionalValue<T> {
    type Native = Option<T::Native>;

    fn to_native(&self) -> Self::Native {
       self.clone().into_option().to_native()
    }
}

impl<N, T> ManagedConvertible<Option<T>> for Option<N>
where
    N: ManagedConvertible<T>,
    T: TopEncode + TopDecode + NestedEncode + NestedDecode
{
    fn to_managed(&self) -> Option<T> {
        self.as_ref().map(|e| e.to_managed())
    }
}

impl<N, T> ManagedConvertible<OptionalValue<T>> for Option<N>
    where
        N: ManagedConvertible<T>,
        T: TopEncodeMulti
{
    fn to_managed(&self) -> OptionalValue<T> {
        OptionalValue::from(self.as_ref().map(|e| e.to_managed()))
    }
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::ManagedBuffer;
    use multiversx_sc_codec::multi_types::OptionalValue;
    use multiversx_sc_scenario::api::StaticApi;
    use crate::types::managed::ManagedConvertible;
    use crate::types::native::NativeConvertible;

    #[test]
    fn test_none_option_to_managed_option() {
        let option: Option<String> = None;
        let managed: Option<ManagedBuffer<StaticApi>> = option.to_managed();

        assert!(managed.is_none());
    }

    #[test]
    fn test_some_option_to_managed_option() {
        let option: Option<String> = Some("some".to_string());
        let managed: Option<ManagedBuffer<StaticApi>> = option.to_managed();
        let expected = Some(ManagedBuffer::<StaticApi>::from("some"));

        assert_eq!(
            managed,
            expected
        );
    }

    #[test]
    fn test_none_option_to_managed_optional_value() {
        let option: Option<String> = None;
        let managed: OptionalValue<ManagedBuffer<StaticApi>> = option.to_managed();

        assert!(managed.is_none());
    }

    #[test]
    fn test_some_option_to_managed_optional_value() {
        let option: Option<String> = Some("some".to_string());
        let managed: OptionalValue<ManagedBuffer<StaticApi>> = option.to_managed();
        let expected = OptionalValue::Some(ManagedBuffer::<StaticApi>::from("some"));

        assert_eq!(
            managed.into_option().unwrap(),
            expected.into_option().unwrap()
        );
    }

    #[test]
    fn test_none_option_to_native() {
        trait IsString {
            fn is_string(&self) -> bool;
        }

        impl IsString for Option<ManagedBuffer<StaticApi>> {
            fn is_string(&self) -> bool {
                false
            }
        }

        impl IsString for Option<String> {
            fn is_string(&self) -> bool {
                true
            }
        }

        let result: Option<ManagedBuffer<StaticApi>> = None;
        let native = result.to_native();

        assert!(!result.is_string());
        assert!(native.is_string());
        assert!(native.is_none());
    }

    #[test]
    fn test_some_option_to_native() {
        let buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("some");
        let result: Option<ManagedBuffer<StaticApi>> = Some(buffer);
        let native = result.to_native();

        let expected_result = String::from("some");

        assert_eq!(
            native.unwrap(),
            expected_result
        );
    }

    #[test]
    fn test_none_optional_value_to_native() {
        trait IsString {
            fn is_string(&self) -> bool;
        }

        impl IsString for OptionalValue<ManagedBuffer<StaticApi>> {
            fn is_string(&self) -> bool {
                false
            }
        }

        impl IsString for Option<String> {
            fn is_string(&self) -> bool {
                true
            }
        }

        let result: OptionalValue<ManagedBuffer<StaticApi>> = OptionalValue::None;
        let native = result.to_native();

        assert!(!result.is_string());
        assert!(native.is_string());
        assert!(native.is_none());
    }

    #[test]
    fn test_some_optional_value_to_native() {
        let buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("some");
        let result: OptionalValue<ManagedBuffer<StaticApi>> = OptionalValue::Some(buffer);
        let native = result.to_native();

        let expected_result = String::from("some");

        assert_eq!(
            native.unwrap(),
            expected_result
        );
    }
}