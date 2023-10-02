use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::types::{BigInt, BigUint};
use crate::types::native::NativeConvertible;

impl<M: ManagedTypeApi> NativeConvertible for BigInt<M> {
    type Native = num_bigint::BigInt;

    fn to_native(&self) -> Self::Native {
        num_bigint::BigInt::from_signed_bytes_be(self.to_signed_bytes_be().as_slice())
    }
}

impl<M: ManagedTypeApi> NativeConvertible for BigUint<M> {
    type Native = num_bigint::BigUint;

    fn to_native(&self) -> Self::Native {
        num_bigint::BigUint::from_bytes_be(self.to_bytes_be().as_slice())
    }
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::{BigInt, BigUint};
    use multiversx_sc_scenario::api::StaticApi;
    use crate::types::managed::ManagedConvertible;

    #[test]
    fn test_biguint_to_managed() {
        let biguint = num_bigint::BigUint::from(10u8).pow(18);
        let managed: BigUint<StaticApi> = biguint.to_managed();

        let expected = BigUint::<StaticApi>::from(10u8).pow(18);

        assert_eq!(
            managed,
            expected
        )
    }

    #[test]
    fn test_bigint_to_managed() {
        let bigint = num_bigint::BigInt::from(10i8).pow(18);
        let managed: BigInt<StaticApi> = bigint.to_managed();

        let expected = BigInt::<StaticApi>::from(10i8).pow(18);

        assert_eq!(
            managed,
            expected
        )
    }

    #[test]
    fn test_negative_bigint_to_managed() {
        let bigint = num_bigint::BigInt::from(-10i8).pow(18);
        let managed: BigInt<StaticApi> = bigint.to_managed();

        let expected = BigInt::<StaticApi>::from(10i8).pow(18);

        assert_eq!(
            managed,
            expected
        )
    }
}