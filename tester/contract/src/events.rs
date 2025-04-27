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

    #[allow_multiple_var_args]
    #[endpoint(emitEventWithMultiValue)]
    fn emit_event_with_multi_value(
        &self,
        first_value: ManagedAddress<Self::Api>,
        multi_value_first: TokenIdentifier<Self::Api>,
        multi_value_second: TokenIdentifier<Self::Api>,
        second_value: BigUint<Self::Api>,
        data: BigUint<Self::Api>,
        values: MultiValueEncoded<Self::Api, BigUint<Self::Api>>,
    ) {
        self.event_with_multi_value(
            first_value,
            MultiValue2::from(
                (
                    multi_value_first,
                    multi_value_second,
                )
            ),
            second_value,
            values,
            data
        );
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

    #[event("eventWithMultiValue")]
    fn event_with_multi_value(
        &self,
        #[indexed] first_value: ManagedAddress<Self::Api>,
        #[indexed] multi_value2: MultiValue2<TokenIdentifier<Self::Api>, TokenIdentifier<Self::Api>>,
        #[indexed] second_value: BigUint<Self::Api>,
        #[indexed] values: MultiValueEncoded<Self::Api, BigUint<Self::Api>>,
        data: BigUint<Self::Api>
    );
}