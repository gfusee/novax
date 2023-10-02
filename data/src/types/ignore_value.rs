use multiversx_sc_codec::multi_types::IgnoreValue;
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

impl NativeConvertible for IgnoreValue {
    type Native = ();

    fn to_native(&self) -> Self::Native {

    }
}

impl ManagedConvertible<IgnoreValue> for () {
    fn to_managed(&self) -> IgnoreValue {
        IgnoreValue
    }
}