multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct EventWithOnlyData<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub amount: BigUint<M>
}

#[multiversx_sc::module]
pub trait EventsModule: ContractBase {

    #[endpoint(emitEmptyEvent)]
    fn emit_empty_event(&self) {
        self.empty_event();
    }

    #[endpoint(emitEventWithOnlyData)]
    fn emit_event_with_only_data(&self, data: EventWithOnlyData<Self::Api>) {
        self.event_with_only_data(data);
    }

    #[endpoint(emitEventWithMultiValueEncoded)]
    fn emit_event_with_multi_value_encoded(
        &self,
        data: BigUint<Self::Api>,
        values: MultiValueEncoded<Self::Api, BigUint<Self::Api>>,
    ) {
        self.event_with_multi_value_encoded(values, data);
    }

    #[event("emptyEvent")]
    fn empty_event(&self);

    #[event("eventWithOnlyData")]
    fn event_with_only_data(&self, data: EventWithOnlyData<Self::Api>);

    #[event("eventWithMultiValueEncoded")]
    fn event_with_multi_value_encoded(
        &self,
        #[indexed] values: MultiValueEncoded<Self::Api, BigUint<Self::Api>>,
        data: BigUint<Self::Api>
    );
}