use multiversx_sc::imports::{EgldOrMultiEsdtPayment, EsdtTokenPayment, ManagedVec};
use multiversx_sc_scenario::imports::StaticApi;
use num_bigint::BigUint;

use crate::{ExecutorError, TokenTransfer};
use crate::error::transaction::TransactionError;

pub enum EgldOrMultiEsdtTransfers {
    Egld(BigUint),
    MultiEsdt(Vec<TokenTransfer>)
}

impl From<EgldOrMultiEsdtTransfers> for EgldOrMultiEsdtPayment<StaticApi> {
    fn from(value: EgldOrMultiEsdtTransfers) -> Self {
        match value {
            EgldOrMultiEsdtTransfers::Egld(value) => Self::Egld(multiversx_sc::types::BigUint::from(value)),
            EgldOrMultiEsdtTransfers::MultiEsdt(transfers) => {
                let mut managed_transfers = ManagedVec::new();

                for transfer in transfers {
                    managed_transfers.push(
                        EsdtTokenPayment::new(
                            transfer.identifier.as_str().into(),
                            transfer.nonce,
                            transfer.amount.into()
                        )
                    )
                }

                Self::MultiEsdt(managed_transfers)
            }
        }
    }
}

pub fn get_egld_or_esdt_transfers(
    egld_value: BigUint,
    esdt_transfers: Vec<TokenTransfer>
) -> Result<EgldOrMultiEsdtTransfers, ExecutorError> {
    let result = if esdt_transfers.is_empty() {
        EgldOrMultiEsdtTransfers::Egld(egld_value)
    } else {
        if egld_value > BigUint::from(0u8) {
            return Err(TransactionError::EgldAndEsdtPaymentsDetected.into())
        }

        EgldOrMultiEsdtTransfers::MultiEsdt(esdt_transfers)
    };

    Ok(result)
}