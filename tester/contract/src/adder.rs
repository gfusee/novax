multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AdderModule: ContractBase {
    #[view(getSum)]
    #[storage_mapper("sum")]
    fn sum(&self) -> SingleValueMapper<Self::Api, BigUint<Self::Api>>;

    #[endpoint]
    fn add(&self, value: BigUint<Self::Api>) {
        self.sum().update(|sum| *sum += value);
    }
}