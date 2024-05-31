#![no_std]

mod adder;
mod printer;
mod call;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Tester: ContractBase + adder::AdderModule + printer::PrinterModule + call::CallModule {
    #[init]
    fn init(&self, initial_value: BigUint<Self::Api>) {
        self.sum().set(initial_value);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
