use crate::types::native::NativeConvertible;
use crate::types::managed::ManagedConvertible;
use multiversx_sc_codec::TopEncodeMulti;

macro_rules! multi_value_native_convertible_impl {
        ($(($mv_struct:ident $len:tt $($n:tt $name:ident $native:ident)+) )+) => {
        $(
            use multiversx_sc_codec::multi_types::$mv_struct;
            impl<$($name: NativeConvertible,)+> NativeConvertible for $mv_struct<$($name,)+> {
                type Native = ($($name::Native,)+);

                fn to_native(&self) -> Self::Native {
                    ($((self.as_tuple()).$n.to_native()),+)
                }
            }
        
            impl<$($name: TopEncodeMulti, $native: ManagedConvertible<$name>,)+> ManagedConvertible<$mv_struct<$($name,)+>> for ($($native,)+) {
                fn to_managed(&self) -> $mv_struct<$($name,)+> {
                    $mv_struct::from(
                        (
                           ($(self.$n.to_managed()),+)
                        )
                    )
                }
            }
        )+
    }
}

multi_value_native_convertible_impl! {
    (MultiValue2   2 0 T0 C0 1 T1 C1)
    (MultiValue3   3 0 T0 C0 1 T1 C1 2 T2 C2)
    (MultiValue4   4 0 T0 C0 1 T1 C1 2 T2 C2 3 T3 C3)
    (MultiValue5   5 0 T0 C0 1 T1 C1 2 T2 C2 3 T3 C3 4 T4 C4)
    (MultiValue6   6 0 T0 C0 1 T1 C1 2 T2 C2 3 T3 C3 4 T4 C4 5 T5 C5)
    (MultiValue7   7 0 T0 C0 1 T1 C1 2 T2 C2 3 T3 C3 4 T4 C4 5 T5 C5 6 T6 C6)
    (MultiValue8   8 0 T0 C0 1 T1 C1 2 T2 C2 3 T3 C3 4 T4 C4 5 T5 C5 6 T6 C6 7 T7 C7)
    (MultiValue9   9 0 T0 C0 1 T1 C1 2 T2 C2 3 T3 C3 4 T4 C4 5 T5 C5 6 T6 C6 7 T7 C7 8 T8 C8)
    (MultiValue10 10 0 T0 C0 1 T1 C1 2 T2 C2 3 T3 C3 4 T4 C4 5 T5 C5 6 T6 C6 7 T7 C7 8 T8 C8 9 T9 C9)
    (MultiValue11 11 0 T0 C0 1 T1 C1 2 T2 C2 3 T3 C3 4 T4 C4 5 T5 C5 6 T6 C6 7 T7 C7 8 T8 C8 9 T9 C9 10 T10 C10)
}