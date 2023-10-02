use crate::types::native::NativeConvertible;
use crate::types::managed::ManagedConvertible;
use multiversx_sc_codec::TopEncode;
use multiversx_sc_codec::NestedEncode;

macro_rules! tuple_impl {
    ($(($($num:tt $type_name:ident $native:ident)+) )+) => {
        $(
            impl<$($type_name, )+> NativeConvertible for ($($type_name, )+)
            where
                $($type_name: NativeConvertible, )+
            {
                type Native = ($($type_name::Native, )+);

                fn to_native(&self) -> Self::Native {
                    ($(self.$num.to_native(), )+)
                }
            }

            impl<$($type_name, $native,)+> ManagedConvertible<($($type_name,)+)> for ($($native,)+)
            where
                $($type_name: TopEncode + NestedEncode + NativeConvertible<Native = $native>,)+
                $($native: ManagedConvertible<$type_name>,)+
            {
                fn to_managed(&self) -> ($($type_name,)+) {
                   ($(self.$num.to_managed()),+,)
                }
            }
        )+
    }
}

tuple_impl! {
    (0 T0 N0)
    (0 T0 N0 1 T1 N1)
    (0 T0 N0 1 T1 N1 2 T2 N2)
    (0 T0 N0 1 T1 N1 2 T2 N2 3 T3 N3)
    (0 T0 N0 1 T1 N1 2 T2 N2 3 T3 N3 4 T4 N4)
    (0 T0 N0 1 T1 N1 2 T2 N2 3 T3 N3 4 T4 N4 5 T5 N5)
    (0 T0 N0 1 T1 N1 2 T2 N2 3 T3 N3 4 T4 N4 5 T5 N5 6 T6 N6)
    (0 T0 N0 1 T1 N1 2 T2 N2 3 T3 N3 4 T4 N4 5 T5 N5 6 T6 N6 7 T7 N7)
    (0 T0 N0 1 T1 N1 2 T2 N2 3 T3 N3 4 T4 N4 5 T5 N5 6 T6 N6 7 T7 N7 8 T8 N8)
    (0 T0 N0 1 T1 N1 2 T2 N2 3 T3 N3 4 T4 N4 5 T5 N5 6 T6 N6 7 T7 N7 8 T8 N8 9 T9 N9)
    (0 T0 N0 1 T1 N1 2 T2 N2 3 T3 N3 4 T4 N4 5 T5 N5 6 T6 N6 7 T7 N7 8 T8 N8 9 T9 N9 10 T10 N10)
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::ManagedBuffer;
    use multiversx_sc_codec::multi_types::{MultiValue2, MultiValue7};
    use multiversx_sc_scenario::api::StaticApi;
    use crate::types::managed::ManagedConvertible;

    type MultiBufferValue7 = MultiValue7<
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>,
        ManagedBuffer<StaticApi>
    >;

    #[test]
    fn tuple_2_to_managed() {
        let tuple = (
            "first".to_string(),
            "second".to_string()
        );
        let managed: MultiValue2<ManagedBuffer<StaticApi>, ManagedBuffer<StaticApi>> = tuple.to_managed();

        let expected_first_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("first");
        let expected_second_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("second");
        let expected = MultiValue2::from(
            (expected_first_buffer, expected_second_buffer)
        );

        assert_eq!(
            managed,
            expected
        );
    }

    #[test]
    fn tuple_7_to_managed() {
        let tuple = (
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
            "fourth".to_string(),
            "fifth".to_string(),
            "sixth".to_string(),
            "seventh".to_string(),
        );
        let managed: MultiBufferValue7  = tuple.to_managed();

        let expected_first_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("first");
        let expected_second_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("second");
        let expected_third_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("third");
        let expected_fourth_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("fourth");
        let expected_fifth_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("fifth");
        let expected_sixth_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("sixth");
        let expected_seventh_buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("seventh");
        let expected = MultiValue7::from(
            (
                expected_first_buffer,
                expected_second_buffer,
                expected_third_buffer,
                expected_fourth_buffer,
                expected_fifth_buffer,
                expected_sixth_buffer,
                expected_seventh_buffer
            )
        );

        assert_eq!(
            managed,
            expected
        );
    }
}