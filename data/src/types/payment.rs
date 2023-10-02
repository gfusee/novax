use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::types::EsdtTokenPayment;
use multiversx_sc_scenario::api::StaticApi;
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

/// Represents a token payment on the blockchain.
///
/// This struct encapsulates the details of a payment made using a specific token.
///
/// # Example
/// ```
/// # use num_bigint::BigUint;
/// # use novax_data::Payment;
/// let payment = Payment {
///     token_identifier: "WEGLD-d7c6bb".to_string(),
///     token_nonce: 0,
///     amount: BigUint::from(10u8).pow(18)
/// };
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct Payment {
    /// A `String` representing the unique identifier of the token involved in the payment.
    pub token_identifier: String,
    /// A `u64` value representing the nonce associated with the token,
    /// used to differentiate between different instances of the same token.
    pub token_nonce: u64,
    /// A `num_bigint::BigUint` representing the amount of tokens being transferred.
    pub amount: num_bigint::BigUint
}

impl<M: ManagedTypeApi> NativeConvertible for EsdtTokenPayment<M> {
    type Native = Payment;

    fn to_native(&self) -> Self::Native {
        Payment {
            token_identifier: self.token_identifier.to_native(),
            token_nonce: self.token_nonce.to_native(),
            amount: self.amount.to_native()
        }
    }
}

impl ManagedConvertible<EsdtTokenPayment<StaticApi>> for Payment {
    fn to_managed(&self) -> EsdtTokenPayment<StaticApi> {
        EsdtTokenPayment::new(
            self.token_identifier.to_managed(),
            self.token_nonce.to_managed(),
            self.amount.to_managed()
        )
    }
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::{BigUint, EsdtTokenPayment, TokenIdentifier};
    use multiversx_sc_scenario::api::StaticApi;
    use crate::Payment;
    use crate::types::managed::ManagedConvertible;

    #[test]
    fn test_payment_to_managed_payment() {
        let payment = Payment {
            token_identifier: "WEGLD-abcdef".to_string(),
            token_nonce: 14u64,
            amount: num_bigint::BigUint::from(100u8),
        };
        let managed: EsdtTokenPayment<StaticApi> = payment.to_managed();

        let expected = EsdtTokenPayment::new(
            TokenIdentifier::from("WEGLD-abcdef"),
            14u64,
            BigUint::from(100u64)
        );

        assert_eq!(
            managed,
            expected
        );
    }
}