use multiversx_sc::types::CodeMetadata;
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

impl NativeConvertible for CodeMetadata {
    type Native = u16;

    fn to_native(&self) -> Self::Native {
        self.bits()
    }
}

impl ManagedConvertible<CodeMetadata> for u16 {
    fn to_managed(&self) -> CodeMetadata {
        CodeMetadata::from(*self)
    }
}
