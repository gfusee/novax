use crate::printer::ProxyTrait as SelfProxy;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CallModule: ContractBase {
    #[endpoint(callAnotherContractReturnTwoU64)]
    fn call_another_contract_return_u64(
        &self,
        address: ManagedAddress<Self::Api>
    ) -> MultiValue2<u64, u64> {
        self.self_proxy(
            address
        )
            .return_two_u64()
            .execute_on_dest_context()
    }

    #[endpoint(asyncCallAnotherContractReturnTwoU64NoCallback)]
    fn async_call_another_contract_return_u64_no_callback(
        &self,
        address: ManagedAddress<Self::Api>
    ) {
        self.self_proxy(
            address
        )
            .return_two_u64()
            .async_call()
            .call_and_exit()
    }

    #[endpoint(asyncCallAnotherContractReturnTwoU64WithReturningCallback)]
    fn async_call_another_contract_return_u64_with_returning_callback(
        &self,
        address: ManagedAddress<Self::Api>
    ) {
        self.self_proxy(
            address
        )
            .return_two_u64()
            .async_call()
            .with_callback(self.callbacks().callback_that_returns_result())
            .call_and_exit()
    }

    #[endpoint(asyncCallAnotherContractReturnTwoU64WithNonReturningCallback)]
    fn async_call_another_contract_return_u64_with_non_returning_callback(
        &self,
        address: ManagedAddress<Self::Api>
    ) {
        self.self_proxy(
            address
        )
            .return_two_u64()
            .async_call()
            .with_callback(self.callbacks().callback_that_returns_nothing())
            .call_and_exit()
    }

    #[callback]
    fn callback_that_returns_result(
        &self,
        #[call_result] result: ManagedAsyncCallResult<Self::Api, MultiValue2<u64, u64>>
    ) -> MultiValue2<u64, u64> {
        let ManagedAsyncCallResult::Ok(result) = result else {
            sc_panic!("async call result is not ok")
        };

        result
    }

    #[callback]
    fn callback_that_returns_nothing(
        &self,
        #[call_result] result: ManagedAsyncCallResult<Self::Api, MultiValue2<u64, u64>>
    ) -> MultiValue2<u64, u64> {
        let ManagedAsyncCallResult::Ok(result) = result else {
            sc_panic!("async call result is not ok")
        };

        result
    }

    #[proxy]
    fn self_proxy(&self, address: ManagedAddress<Self::Api>) -> crate::Proxy<Self::Api>;
}