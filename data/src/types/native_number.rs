use multiversx_sc_scenario::api::StaticApi;
use num_bigint::{BigInt, BigUint};
use crate::types::managed::{managed_convertible_impl_self, ManagedConvertible};

type SCBigInt = multiversx_sc::types::BigInt<StaticApi>;
type SCBigUint = multiversx_sc::types::BigUint<StaticApi>;

impl ManagedConvertible<SCBigInt> for BigInt {
    fn to_managed(&self) -> SCBigInt {
        SCBigInt::from(self)
    }
}

impl ManagedConvertible<SCBigUint> for BigUint {
    fn to_managed(&self) -> SCBigUint {
        SCBigUint::from(self)
    }
}

managed_convertible_impl_self! {
    BigInt BigUint
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::{BigInt, BigUint};
    use multiversx_sc_scenario::api::StaticApi;
    use crate::types::native::NativeConvertible;

    #[test]
    fn test_biguint_to_native() {
        let biguint: BigUint<StaticApi> = BigUint::from(10u64).pow(18);
        let native = biguint.to_native();

        let expected = num_bigint::BigUint::from(10u64).pow(18);

        assert_eq!(
            native,
            expected
        )
    }

    #[test]
    fn test_bigint_to_native() {
        let bigint: BigInt<StaticApi> = BigInt::from(10i64).pow(18);
        let native = bigint.to_native();

        let expected = num_bigint::BigInt::from(10i64).pow(18);

        assert_eq!(
            native,
            expected
        )
    }

    #[test]
    fn test_negative_bigint_to_native() {
        let bigint: BigInt<StaticApi> = BigInt::from(-10i64).pow(18);
        let native = bigint.to_native();

        let expected = num_bigint::BigInt::from(-10i64).pow(18);

        assert_eq!(
            native,
            expected
        )
    }
}