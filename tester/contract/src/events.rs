multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule: ContractBase {
    #[event("emptyEvent")]
    fn empty_event(&self);
}