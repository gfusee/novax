use multiversx_sc::imports::{EgldOrMultiEsdtPayment, EsdtTokenPayment, ManagedVec, TokenIdentifier};
use multiversx_sc_scenario::api::StaticApi;
use num_bigint::BigUint;
use crate::error::transaction::TransactionError;
use crate::{ExecutorError, TokenTransfer};

pub fn get_egld_or_esdt_transfers(
    egld_value: &BigUint,
    esdt_transfers: &[TokenTransfer]
) -> Result<EgldOrMultiEsdtPayment<StaticApi>, ExecutorError> {
    let result = if esdt_transfers.is_empty() {
        EgldOrMultiEsdtPayment::Egld(multiversx_sc::types::BigUint::<StaticApi>::from(egld_value))
    } else {
        if egld_value > &BigUint::from(0u8) {
            return Err(TransactionError::EgldAndEsdtPaymentsDetected.into())
        }

        let mut payments = ManagedVec::new();

        for esdt_transfer in esdt_transfers {
            payments.push(
                EsdtTokenPayment::new(
                    TokenIdentifier::from(&esdt_transfer.identifier),
                    esdt_transfer.nonce,
                    multiversx_sc::types::BigUint::from(&esdt_transfer.amount)
                )
            )
        }

        EgldOrMultiEsdtPayment::MultiEsdt(payments)
    };

    Ok(result)
}