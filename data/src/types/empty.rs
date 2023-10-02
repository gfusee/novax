use multiversx_sc_codec::Empty;
use crate::types::native::NativeConvertible;

impl NativeConvertible for Empty {
    type Native = Empty;

    fn to_native(&self) -> Self::Native {
        Empty
    }
}

impl NativeConvertible for () {
    type Native = ();

    fn to_native(&self) -> Self::Native {

    }
}