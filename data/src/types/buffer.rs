use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::types::{EgldOrEsdtTokenIdentifier, ManagedBuffer, TokenIdentifier};
use multiversx_sc_scenario::api::StaticApi;
use crate::constants::EGLD_TOKEN_IDENTIFIER;
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

impl NativeConvertible for String {
    type Native = String;

    fn to_native(&self) -> Self::Native {
        self.clone()
    }
}

impl<M: ManagedTypeApi> NativeConvertible for ManagedBuffer<M> {
    type Native = String;

    fn to_native(&self) -> Self::Native {
        let bytes = self.to_boxed_bytes();
        let result = String::from_utf8_lossy(bytes.as_slice());

        result.to_string()
    }
}

impl<M: ManagedTypeApi> NativeConvertible for TokenIdentifier<M> {
    type Native = String;

    fn to_native(&self) -> Self::Native {
        self.as_managed_buffer().to_native()
    }
}

impl<M: ManagedTypeApi> NativeConvertible for EgldOrEsdtTokenIdentifier<M> {
    type Native = String;

    fn to_native(&self) -> Self::Native {
        if self.is_egld() {
            EGLD_TOKEN_IDENTIFIER.to_string()
        } else {
            self.clone().unwrap_esdt().to_native()
        }
    }
}

impl ManagedConvertible<ManagedBuffer<StaticApi>> for String {
    fn to_managed(&self) -> ManagedBuffer<StaticApi> {
        ManagedBuffer::from(self.as_bytes())
    }
}

impl ManagedConvertible<TokenIdentifier<StaticApi>> for String {
    fn to_managed(&self) -> TokenIdentifier<StaticApi> {
        TokenIdentifier::from(self.as_bytes())
    }
}

impl ManagedConvertible<EgldOrEsdtTokenIdentifier<StaticApi>> for String {
    fn to_managed(&self) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        if self == EGLD_TOKEN_IDENTIFIER {
            EgldOrEsdtTokenIdentifier::egld()
        } else {
            let token_identifier: TokenIdentifier<StaticApi> = self.to_managed();
            EgldOrEsdtTokenIdentifier::esdt(token_identifier)
        }
    }
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::{ManagedBuffer, TokenIdentifier};
    use multiversx_sc_scenario::api::StaticApi;
    use crate::types::managed::ManagedConvertible;
    use crate::types::native::NativeConvertible;

    #[test]
    fn test_managed_buffer_to_native() {
        let buffer: ManagedBuffer<StaticApi> = ManagedBuffer::from("This is a buffer");
        let native = buffer.to_native();

        let expected = String::from("This is a buffer");

        assert_eq!(
            native,
            expected
        );
    }

    #[test]
    fn test_token_identifier_to_native() {
        let buffer: TokenIdentifier<StaticApi> = TokenIdentifier::from("WEGLD-abcdef");
        let native = buffer.to_native();

        let expected = String::from("WEGLD-abcdef");

        assert_eq!(
            native,
            expected
        );
    }

    #[test]
    fn test_string_to_managed_buffer() {
        let value = "This is a buffer".to_string();
        let managed: ManagedBuffer<StaticApi> = value.to_managed();

        assert_eq!(
            value.as_bytes(),
            managed.to_boxed_bytes().as_slice()
        );
    }

    #[test]
    fn test_string_to_token_identifier() {
        let value = "WEGLD-abcdef".to_string();
        let managed: TokenIdentifier<StaticApi> = value.to_managed();

        assert_eq!(
            value.as_bytes(),
            managed.to_boxed_bytes().as_slice()
        );
    }
}