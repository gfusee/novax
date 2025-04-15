use convert_case::Case;
use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use crate::abi::constructor::{AbiConstructor, AbiOutputs};
use crate::abi::result::{Abi, AbiEndpoints, AbiEvents, AbiTypes};
use crate::abi::endpoint::{AbiEndpoint, AbiInputs, AbiPossibleMutability};
use crate::abi::event::{AbiEvent, AbiEventInputs};
use crate::abi::event_input::AbiEventInput;
use crate::abi::output::AbiOutput;
use crate::errors::build_error::BuildError;
use crate::generator::impl_abi_types::{impl_abi_event_filter_struct_type, impl_abi_event_struct_type};
use crate::utils::get_api_generic_ident::get_api_generic_ident;
use crate::utils::get_native_struct_managed_name::get_native_struct_managed_name;
use crate::utils::parse_abi_type_name_to_managed_ident::parse_abi_type_name_to_managed_ident;

pub(crate) fn impl_contract(mod_name: &str, abi: &Abi) -> Result<TokenStream, BuildError> {
    let mod_name_ident = format_ident!("{}", mod_name);
    let name = format_ident!("{}", abi.get_contract_name());
    let proxy_name = abi.get_proxy_name();
    let proxy_name_ident = format_ident!("{}", proxy_name);
    let proxy_mod_name = proxy_name.to_case(Case::Snake);
    let proxy_mod_name_ident = format_ident!("{}", proxy_mod_name);
    let contract_info_name = format!("{name}Infos");
    let contract_info_ident = format_ident!("{}", contract_info_name);
    let call_name = format_ident!("{}", abi.get_call_name());
    let query_name = format_ident!("{}", abi.get_query_name());
    let query_events_name = format_ident!("{}", abi.get_query_events_name());
    let (calls_impls, queries_impls) = impl_abi_endpoints(
        &contract_info_name,
        &abi.endpoints,
        &abi.types
    )?;

    let (events_structs_impls, events_filters_structs_impls, query_events_impls) = if let Some(abi_events) = &abi.events {
        let (struct_impls, filter_structs_impls, query_impls) = impl_abi_events(
            &contract_info_name,
            abi_events,
            &abi.types
        )?;

        (Some(struct_impls), Some(filter_structs_impls), Some(query_impls))
    } else {
        (None, None, None)
    };

    let deploy_impl = impl_abi_constructor(&contract_info_name, &abi.constructor, &abi.types)?;
    let proxy_impls = impl_proxy_functions(&abi.constructor, &abi.endpoints, &abi.types)?;

    Ok(
        quote! {
            use crate::#mod_name_ident::#mod_name_ident::Proxy as CodegenProxy;

            type #contract_info_ident = ContractInfo<CodegenProxy<StaticApi>>;

            #[allow(missing_docs)]
            mod #proxy_mod_name_ident {
                use super::*;

                #[multiversx_sc::proxy]
                trait #proxy_name_ident {
                    #proxy_impls
                }
            }

            use #proxy_mod_name_ident::*;

            /// Generated smart contract interface for interacting with the deployed contract on the blockchain.
            /// The name of the struct is constructed by concatenating the ABI's name field with the string "Contract".
            ///
            /// This struct facilitates interactions with the smart contract through the provided methods,
            /// offering a native Rust interface to perform smart contract operations like querying, calling functions,
            /// and deploying new instances of the contract.
            ///
            /// The struct is parameterized over a type `A` that represents an address or reference to an address,
            /// where `A` is expected to be dereferenceable to a type from which an `Address` can be derived.
            ///
            /// # Methods
            ///
            /// - `query(...)`: This method facilitates querying data from the smart contract. It accepts necessary
            /// parameters as defined in the ABI and returns a result as per the ABI's specification for the particular query.
            ///
            /// - `call(...)`: This method allows for calling smart contract functions. It handles the formation of the
            /// transaction, its submission to the blockchain, and returns the transaction result along with any data
            /// returned by the smart contract function, as per the ABI's specification.
            ///
            /// - `deploy(...)`: This is a static method useful for deploying a new instance of the smart contract to the blockchain.
            /// It accepts deployment parameters as defined in the ABI, and returns an instance of the generated contract struct,
            /// now associated with the newly deployed contract's address on the blockchain.
            ///
            /// # Example
            ///
            /// ```ignore
            /// let contract = #name::deploy(&deploy_executor, deploy_data)?;
            /// let query_result = contract.query(&query_executor, query_data)?;
            /// let call_result = contract.call(&call_executor, call_data)?;
            /// ```
            ///
            /// # Note
            ///
            /// The actual method signatures and types will be generated based on the ABI provided.
            #[derive(Copy, Clone)]
            pub struct #name<A>
            where
                A: Deref + Send + Sync,
                Address: for<'a> From<&'a A::Target>
            {
                address: A
            }

            /// Struct representing a smart contract function call.
            /// This struct contains all necessary information to perform a function call on the smart contract,
            /// including the executor to handle transaction execution, value and token transfers to be sent along with the call,
            /// gas limit for the call, and the address of the contract to call.
            /// It has methods representing endpoints generated from the ABI.
            /// Additionally, convenient methods are added to set information like ESDT transfers and others.
            #[derive(Clone, Debug)]
            pub struct #call_name<Executor, A>
            where
                Executor: TransactionExecutor,
                A: Deref + Send + Sync,
                Address: for<'a> From<&'a A::Target>
            {
                executor: Executor,
                egld_value: num_bigint::BigUint,
                token_transfers: Vec<TokenTransfer>,
                gas_limit: u64,
                contract_address: A
            }

            /// Struct representing a smart contract query.
            /// This struct contains all necessary information to perform a query on the smart contract,
            /// including the executor to handle query execution, the value to be sent along with the query,
            /// and the address of the contract to query.
            /// It has methods representing views generated from the ABI.
            /// Additionally, a convenient method `with_caching` is provided to allow caching the result through a `CachingStrategy`.
            /// By default, `CachingNone` is used, but more caching strategies can be found in the "novax-caching" crate.
            #[derive(Clone, Debug)]
            pub struct #query_name<Executor, Caching, A>
            where
                Executor: QueryExecutor,
                Caching: CachingStrategy,
                A: Deref + Send + Sync,
                Address: for<'a> From<&'a A::Target>
            {
                contract_address: A,
                egld_value: num_bigint::BigUint,
                executor: Executor,
                caching: Caching
            }

            #[derive(Clone, Debug)]
            pub struct #query_events_name<Executor, Caching, A>
            where
                Executor: QueryEventsExecutor,
                Caching: CachingStrategy,
                A: Deref + Send + Sync,
                Address: for<'a> From<&'a A::Target>
            {
                contract_address: A,
                executor: Executor,
                caching: Caching
            }

            /// The main struct representing the smart contract.
            /// This struct provides methods to create instances for contract call and query.
            /// It also provides a method for deploying a new instance of the smart contract.
            impl<A> #name<A>
            where
                A: Deref + Send + Sync,
                Address: for<'a> From<&'a A::Target>
            {
                /// Creates a new instance of the smart contract interface with the provided address.
                pub fn new(address: A) -> #name<A> {
                    #name {
                        address
                    }
                }

                /// Returns a new instance of `#call_name` struct to perform function calls on the smart contract.
                pub fn call<Executor: TransactionExecutor>(
                    self,
                    executor: Executor,
                    gas_limit: u64
                ) -> #call_name<Executor, A> {
                    #call_name {
                        executor,
                        gas_limit,
                        egld_value: num_bigint::BigUint::from(0u8),
                        token_transfers: vec![],
                        contract_address: self.address
                    }
                }

                /// Returns a new instance of `#query_name` struct to perform queries on the smart contract.
                pub fn query<Executor: QueryExecutor>(self, executor: Executor) -> #query_name<Executor, CachingNone, A> {
                    #query_name {
                        contract_address: self.address,
                        egld_value: num_bigint::BigUint::from(0u8),
                        executor,
                        caching: CachingNone
                    }
                }

                /// Returns a new instance of `#query_name` struct to perform queries on the smart contract.
                pub fn query_events<Executor: QueryEventsExecutor>(self, executor: Executor) -> #query_events_name<Executor, CachingNone, A> {
                    #query_events_name {
                        contract_address: self.address,
                        executor,
                        caching: CachingNone
                    }
                }
            }

            impl #name<String> { // String is used but is not used
                #deploy_impl
            }

            impl<Executor, Caching, A> #query_name<Executor, Caching, A>
            where
                Executor: QueryExecutor,
                Caching: CachingStrategy,
                A: Deref + Send + Sync,
                Address: for<'a> From<&'a A::Target>
            {
                /// Modifies the caching strategy used for the query.
                /// This method allows changing the caching strategy to a different type,
                /// useful in cases where varying levels of caching are desired.
                ///
                /// # Parameters
                /// * `strategy`: A reference to the new caching strategy to be used.
                ///
                /// # Returns
                /// A new instance of `#query_name` with the updated caching strategy.
                pub fn with_caching_strategy<C2: CachingStrategy + Clone>(self, strategy: &C2) -> #query_name<Executor, C2, A> {
                    #query_name {
                        contract_address: self.contract_address,
                        executor: self.executor,
                        egld_value: self.egld_value,
                        caching: strategy.clone(),
                    }
                }

                /// Modifies the EGLD value associated with the query.
                /// This method allows setting a new EGLD value to be used in the query.
                ///
                /// # Parameters
                /// * `egld_value`: The new EGLD value to be used.
                ///
                /// # Returns
                /// A new instance of `#query_name` with the updated EGLD value.
                pub fn with_egld_value(self, egld_value: num_bigint::BigUint) -> #query_name<Executor, Caching, A> {
                    #query_name {
                        egld_value,
                        ..self
                    }
                }

                // Other query implementations generated from the ABI
                #queries_impls
            }

            #events_structs_impls
            #events_filters_structs_impls

            impl<Executor, Caching, A> #query_events_name<Executor, Caching, A>
            where
                Executor: QueryEventsExecutor,
                Caching: CachingStrategy,
                A: Deref + Send + Sync,
                Address: for<'a> From<&'a A::Target>
            {
                /// Modifies the caching strategy used for the event query.
                /// This method allows changing the caching strategy to a different type,
                /// useful in cases where varying levels of caching are desired.
                ///
                /// # Parameters
                /// * `strategy`: A reference to the new caching strategy to be used.
                ///
                /// # Returns
                /// A new instance of `#query_name` with the updated caching strategy.
                pub fn with_caching_strategy<C2: CachingStrategy + Clone>(self, strategy: &C2) -> #query_events_name<Executor, C2, A> {
                    #query_events_name {
                        contract_address: self.contract_address,
                        executor: self.executor,
                        caching: strategy.clone(),
                    }
                }

                // Other query implementations generated from the ABI
                #query_events_impls
            }

            impl<Executor, A> #call_name<Executor, A>
            where
                Executor: TransactionExecutor,
                A: Deref + Send + Sync,
                Address: for<'a> From<&'a A::Target>
            {
                /// Modifies the EGLD value associated with the call.
                /// This method allows setting a new EGLD value for the call transaction.
                ///
                /// # Parameters
                /// * `egld_value`: The new EGLD value to be used.
                ///
                /// # Returns
                /// A new instance of `#call_name` with the updated EGLD value.
                pub fn with_egld_value(self, egld_value: num_bigint::BigUint) -> #call_name<Executor, A> {
                    #call_name {
                        egld_value,
                        ..self
                    }
                }

                /// Modifies the ESDT transfers associated with the call.
                /// This method allows setting a new list of ESDT transfers for the call transaction.
                ///
                /// # Parameters
                /// * `token_transfers`: A reference to a vector of `TokenTransfer` objects representing the new ESDT transfers to be used.
                ///
                /// # Returns
                /// A new instance of `#call_name` with the updated ESDT transfers.
                pub fn with_esdt_transfers(self, token_transfers: &Vec<TokenTransfer>) -> #call_name<Executor, A> {
                    #call_name {
                        token_transfers: token_transfers.clone(),
                        ..self
                    }
                }

                /// Modifies the gas limit associated with the call.
                /// This method allows setting a new gas limit for the call transaction.
                ///
                /// # Parameters
                /// * `gas_limit`: The new gas limit to be used.
                ///
                /// # Returns
                /// A new instance of `#call_name` with the updated gas limit.
                pub fn with_gas_limit(self, gas_limit: u64) -> #call_name<Executor, A> {
                    #call_name {
                        gas_limit,
                        ..self
                    }
                }

                // Other call implementations generated from the ABI
                #calls_impls
            }
        }
    )
}

fn impl_proxy_functions(abi_constructor: &AbiConstructor, abi_endpoints: &AbiEndpoints, abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let mut endpoints_impls: Vec<TokenStream> = vec![];
    endpoints_impls.push(impl_proxy_endpoint(&abi_constructor.clone().into_endpoint(), abi_types)?);
    for endpoint in abi_endpoints {
        endpoints_impls.push(impl_proxy_endpoint(endpoint, abi_types)?)
    }

    Ok(
        quote!{
            #(#endpoints_impls)*
        }
    )
}

fn impl_proxy_endpoint(abi_endpoint: &AbiEndpoint, abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let debug_api = quote!{Self::Api};
    let endpoint_name_ident = format_ident!("{}", abi_endpoint.name);
    let function_name_ident = format_ident!("{}", abi_endpoint.name.to_case(Case::Snake));

    let mut managed_inputs: Vec<TokenStream> = vec![];
    for input in &abi_endpoint.inputs {
        let input_name_ident = format_ident!("{}", input.name);
        let type_ident = parse_abi_type_name_to_managed_ident(&input.r#type, abi_types, &debug_api)?;

        managed_inputs.push(quote!{#input_name_ident: #type_ident})
    }

    let mut managed_outputs_idents: Vec<TokenStream> = vec![];
    for output in &abi_endpoint.outputs {
        let (managed_output, _) = impl_endpoint_output_for_query(output, abi_types, &debug_api)?;
        managed_outputs_idents.push(managed_output);
    }

    let function_managed_outputs = if abi_endpoint.outputs.is_empty() {
        quote! {}
    } else if abi_endpoint.outputs.len() == 1 {
        let managed_output_ident = managed_outputs_idents.first().unwrap();
        quote! { -> #managed_output_ident}
    } else {
        let length = format_ident!("MultiValue{}", abi_endpoint.outputs.len());
        quote! { -> #length<#(#managed_outputs_idents), *>}
    };

    let proc_macro_to_use_ident = if abi_endpoint.mutability == AbiPossibleMutability::Constructor {
        format_ident!("init").to_token_stream()
    } else {
        quote! { endpoint(#endpoint_name_ident) }
    };

    let result = quote! {
        #[#proc_macro_to_use_ident]
        fn #function_name_ident(&self, #(#managed_inputs), *)#function_managed_outputs;
    };

    Ok(result)
}

// (TokenStream, TokenStream) = (calls, queries)
fn impl_abi_endpoints(
    contract_info_name: &str,
    abi_endpoints: &AbiEndpoints,
    abi_types: &AbiTypes
) -> Result<(TokenStream, TokenStream), BuildError> {
    let mut calls_impls: Vec<TokenStream> = vec![];
    let mut queries_impls: Vec<TokenStream> = vec![];
    for endpoint in abi_endpoints {
        let endpoint_impls = impl_abi_endpoint_call_query(
            contract_info_name,
            endpoint,
            abi_types
        )?;

        calls_impls.push(endpoint_impls.0);
        queries_impls.push(endpoint_impls.1);
    }

    Ok(
        (
            quote!{
                #(#calls_impls)*
            },
            quote!{
                #(#queries_impls)*
            }
        )
    )
}

// (TokenStream, TokenStream, TokenStream) = (event structs impls, event filter structs impls, query functions impls)
fn impl_abi_events(
    contract_info_name: &str,
    abi_events: &AbiEvents,
    abi_types: &AbiTypes,
) -> Result<(TokenStream, TokenStream, TokenStream), BuildError> {
    let mut events_queries_impls: Vec<TokenStream> = vec![];
    let mut events_structs_impls: Vec<TokenStream> = vec![];
    let mut events_filter_structs_impls: Vec<TokenStream> = vec![];

    for abi_event in abi_events {
        let event_impl = impl_abi_event_query(
            contract_info_name,
            abi_event,
            abi_types
        )?;

        events_structs_impls.push(event_impl.0);
        events_filter_structs_impls.push(event_impl.1);
        events_queries_impls.push(event_impl.2);
    }

    Ok(
        (
            quote! {
                #(#events_structs_impls)*
            },
            quote! {
                #(#events_filter_structs_impls)*
            },
            quote! {
                #(#events_queries_impls)*
            }
        )
    )
}

fn impl_abi_endpoint_call_query(
    contract_info_name: &str,
    abi_endpoint: &AbiEndpoint,
    abi_types: &AbiTypes
) -> Result<(TokenStream, TokenStream), BuildError> {
    let debug_api = get_api_generic_ident();
    let contract_info_ident = format_ident!("{}", contract_info_name);
    let endpoint_name = abi_endpoint.name.as_str();
    let function_name_ident = format_ident!("{}", abi_endpoint.name.to_case(Case::Snake));
    let function_inputs = impl_endpoint_inputs(true, &abi_endpoint.inputs, abi_types)?;
    let mut managed_outputs_idents: Vec<TokenStream> = vec![];
    let mut native_outputs_idents: Vec<TokenStream> = vec![];
    for output in &abi_endpoint.outputs {
        let (managed_output, native_output) = impl_endpoint_output_for_query(output, abi_types, &debug_api)?;
        managed_outputs_idents.push(managed_output);
        native_outputs_idents.push(native_output);
    }
    let (function_managed_outputs, function_native_outputs) = impl_endpoint_outputs(&abi_endpoint.outputs, abi_types, &debug_api)?;

    let endpoint_query_key = impl_endpoint_key_for_query(&abi_endpoint.name, &abi_endpoint.inputs);

    let (endpoint_args_let_statements, endpoint_args_inputs) = impl_endpoint_args_for_call(&abi_endpoint.inputs, abi_types)?;

    let common_token = quote! {
        let _novax_request_arc = crate::utils::static_request_arc::get_static_request_arc_clone();

        let _novax_contract_address = Address::from(&self.contract_address);
        let _novax_contract_address_value: AddressValue = (&_novax_contract_address).into();
        let mut _novax_contract = #contract_info_ident::new(&_novax_contract_address_value);

        #endpoint_args_let_statements

        let _novax_payment = if self.egld_value > num_bigint::BigUint::from(0u8) {
            EgldOrMultiEsdtPayment::Egld(BigUint::<#debug_api>::from(self.egld_value.clone()))
        } else {
            EgldOrMultiEsdtPayment::Egld(BigUint::<#debug_api>::from(0u8))
        };

        let mut _novax_bytes_args: std::vec::Vec<std::vec::Vec<u8>> = vec![];
        #(#endpoint_args_inputs) *
    };

    let call_token = quote! {
        /// Executes the `#function_name` function on the smart contract.
        ///
        /// # Description
        /// #abi_description
        ///
        /// # Parameters
        /// #function_inputs
        ///
        /// # Returns
        /// A `Result` containing a `CallResult` with the `#function_native_outputs` or a `NovaXError` if the call fails.
        pub async fn #function_name_ident(#function_inputs) -> Result<CallResult<#function_native_outputs>, NovaXError> {
            #common_token

            let result = self.executor.sc_call::<#function_managed_outputs>(
                    &_novax_contract_address,
                    #endpoint_name.to_string(),
                    _novax_bytes_args,
                    self.gas_limit,
                    self.egld_value.clone(),
                    self.token_transfers.clone(),
            ).await?;

            Result::Ok(result)
        }
    };

    let query_token = quote! {
        /// Executes the `#function_name` query on the smart contract.
        ///
        /// # Description
        /// #abi_description
        ///
        /// # Parameters
        /// #function_inputs
        ///
        /// # Returns
        /// A `Result` containing the `#function_native_outputs` or a `NovaXError` if the query fails.
        pub async fn #function_name_ident(#function_inputs) -> Result<#function_native_outputs, NovaXError> {
            #common_token
            #endpoint_query_key
            self.caching.get_or_set_cache(
                _novax_key,
                async {
                    let result = self.executor
                        .execute::<#function_managed_outputs>(
                            &_novax_contract_address,
                            #endpoint_name.to_string(),
                            _novax_bytes_args,
                            self.egld_value.clone(),
                            vec![],
                        ).await;

                    if let Result::Ok(result) = result {
                        Result::Ok::<_, NovaXError>(result)
                    } else {
                        let error: NovaXError = result.unwrap_err().into();
                        Result::Err(error)
                    }
                }
            ).await
        }
    };

    Ok((call_token, query_token))
}

// (TokenStream, TokenStream, TokenStream) = (event struct impls, event filter struct impls, function impl)
fn impl_abi_event_query(
    contract_info_name: &str,
    abi_event: &AbiEvent,
    abi_types: &AbiTypes
) -> Result<(TokenStream, TokenStream, TokenStream), BuildError> {
    let debug_api = get_api_generic_ident();
    let contract_info_ident = format_ident!("{}", contract_info_name);
    let event_identifier = abi_event.identifier.as_str();

    if event_identifier.is_empty() {
        return Ok((quote! {}, quote! {}, quote! {}))
    }

    let event_identifier_ident = format_ident!("{}", event_identifier.to_case(Case::Snake));
    let mut managed_inputs_idents: Vec<TokenStream> = vec![];
    let mut native_inputs_idents: Vec<TokenStream> = vec![];
    for input in &abi_event.inputs {
        let (managed_input, native_input) = impl_event_input_for_query(input, abi_types, &debug_api)?;
        managed_inputs_idents.push(managed_input);
        native_inputs_idents.push(native_input);
    }
    let (event_managed_inputs, event_managed_inputs_types, event_native_inputs_types) = impl_event_inputs(&abi_event.inputs, abi_types, &debug_api)?;
    let abi_event_field_names = abi_event.inputs
        .clone()
        .into_iter()
        .map(|input| input.name)
        .collect::<Vec<_>>();

    let event_field_managed_names_and_types = abi_event_field_names
        .clone()
        .into_iter()
        .zip(event_managed_inputs_types)
        .collect::<Vec<_>>();

    let event_field_native_names_and_types = abi_event_field_names
        .into_iter()
        .zip(event_native_inputs_types)
        .collect::<Vec<_>>();

    let (event_return_struct_type, event_return_struct_type_impls) = impl_abi_event_struct_type(event_identifier, event_field_native_names_and_types)?;
    let (event_filters_struct_type, event_filters_struct_type_impls) = impl_abi_event_filter_struct_type(event_identifier, event_field_managed_names_and_types)?;
    let endpoint_query_key = impl_endpoint_key_for_query(event_identifier, &vec![]); // TODO

    let event_query_token = quote! {
        pub async fn #event_identifier_ident(
            &self,
            event_filters: Option<#event_filters_struct_type>,
        ) -> Result<std::vec::Vec<EventQueryResult<#event_return_struct_type>>, NovaXError> {
            let _novax_request_arc = crate::utils::static_request_arc::get_static_request_arc_clone();

            let _novax_contract_address = Address::from(&self.contract_address);
            let _novax_contract_address_value: AddressValue = (&_novax_contract_address).into();
            let mut _novax_contract = #contract_info_ident::new(&_novax_contract_address_value);

            #endpoint_query_key
            self.caching.get_or_set_cache(
                _novax_key,
                async {
                    let result_native_tuple: Result<std::vec::Vec<EventQueryResult<_>>, _> = self.executor
                        .execute::<#event_managed_inputs, #event_filters_struct_type>(
                            &_novax_contract_address,
                            #event_identifier,
                            event_filters,
                        ).await;

                    if let Result::Ok(result_native_tuple) = result_native_tuple {
                        Result::Ok::<std::vec::Vec<EventQueryResult<#event_return_struct_type>>, NovaXError>(
                            result_native_tuple
                                .into_iter()
                                .map(|event_query_result| {
                                    EventQueryResult {
                                        timestamp: event_query_result.timestamp,
                                        event: event_query_result.event.into(),
                                    }
                                })
                                .collect()
                        )
                    } else {
                        let error: NovaXError = result_native_tuple.unwrap_err().into();
                        Result::Err(error)
                    }
                }
            ).await
        }
    };

    Ok((event_return_struct_type_impls, event_filters_struct_type_impls, event_query_token))
}

fn impl_abi_constructor(contract_info_name: &str, abi_constructor: &AbiConstructor, abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let debug_api = get_api_generic_ident();
    let contract_info_ident = format_ident!("{}", contract_info_name);
    let function_name = format_ident!("deploy");
    let function_inputs = impl_endpoint_inputs(false, &abi_constructor.inputs, abi_types)?;
    let (endpoint_args_let_statements, endpoint_args_inputs) = impl_endpoint_args_for_call(&abi_constructor.inputs, abi_types)?;
    let (function_managed_outputs, function_native_outputs) = impl_endpoint_outputs(&abi_constructor.outputs, abi_types, &debug_api)?;

    let function_token = quote! {
        /// This asynchronous function encapsulates the logic for deploying a smart contract to the blockchain.
        pub async fn #function_name<Code: AsBytesValue, Executor: DeployExecutor>(_novax_deploy_data: DeployData<Code>, _novax_executor: &mut Executor, _novax_egld_value: num_bigint::BigUint, _novax_gas_limit: u64, #function_inputs) -> Result<(Address, CallResult<#function_native_outputs>), NovaXError> {
            let _novax_request_arc = crate::utils::static_request_arc::get_static_request_arc_clone();

            let mut _novax_contract = #contract_info_ident::new(&multiversx_sc::types::Address::from(<[u8;32]>::default()));

            #endpoint_args_let_statements

            let mut _novax_bytes_args: std::vec::Vec<std::vec::Vec<u8>> = vec![];
            #(#endpoint_args_inputs) *

            let _novax_code_bytes = _novax_deploy_data.code.into_bytes_value().await?;

            _novax_executor.sc_deploy::<#function_managed_outputs>(
                _novax_code_bytes,
                _novax_deploy_data.metadata,
                _novax_egld_value,
                _novax_bytes_args,
                _novax_gas_limit
            )
                .await
                .map_err(NovaXError::from)
        }
    };

    Ok(function_token)
}

// (TokenStream, TokenStream) = function_managed_outputs, function_native_outputs
fn impl_endpoint_outputs(outputs: &AbiOutputs, abi_types: &AbiTypes, api_generic: &TokenStream) -> Result<(TokenStream, TokenStream), BuildError> {
    let mut managed_outputs_idents: Vec<TokenStream> = vec![];
    let mut native_outputs_idents: Vec<TokenStream> = vec![];
    for output in outputs {
        let (managed_output, native_output) = impl_endpoint_output_for_query(output, abi_types, api_generic)?;
        managed_outputs_idents.push(managed_output);
        native_outputs_idents.push(native_output);
    }
    let (function_managed_outputs, function_native_outputs) = if outputs.is_empty() {
        (quote! {()}, quote!{()})
    } else if outputs.len() == 1 {
        let managed_output_ident = managed_outputs_idents.first().unwrap();
        let native_output_ident = native_outputs_idents.first().unwrap();
        (quote! {#managed_output_ident}, quote!{#native_output_ident})
    } else {
        let length = format_ident!("MultiValue{}", outputs.len());
        (quote! {#length<#(#managed_outputs_idents), *>}, quote! {(#(#native_outputs_idents), *)})
    };

    Ok((function_managed_outputs, function_native_outputs))
}

fn impl_event_inputs(inputs: &AbiEventInputs, abi_types: &AbiTypes, api_generic: &TokenStream) -> Result<(TokenStream, Vec<TokenStream>, Vec<TokenStream>), BuildError> {
    let mut managed_outputs_idents: Vec<TokenStream> = vec![];
    let mut native_outputs_idents: Vec<TokenStream> = vec![];
    for input in inputs {
        let (managed_output, native_output) = impl_event_input_for_query(input, abi_types, api_generic)?;
        managed_outputs_idents.push(managed_output);
        native_outputs_idents.push(native_output);
    }
    let (function_managed_outputs, function_native_outputs) = if inputs.is_empty() {
        (quote! {()}, quote!{()})
    } else {
        let length = format_ident!("MultiValue{}", inputs.len());
        (quote! {#length<#(#managed_outputs_idents), *>}, quote! {(#(#native_outputs_idents), *)})
    };

    Ok((function_managed_outputs, managed_outputs_idents, native_outputs_idents))
}

fn impl_endpoint_inputs(should_include_self: bool, abi_inputs: &AbiInputs, abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let debug_api = get_api_generic_ident();
    let mut inputs_idents: Vec<TokenStream> = vec![];
    if should_include_self {
        inputs_idents.push(quote! { &mut self })
    }

    for input in abi_inputs {
        let input_name_ident = format_ident!("{}", input.name);
        let input_managed_type_ident = get_managed_type_for_abi_type(&input.r#type, abi_types, &debug_api)?;

        inputs_idents.push(quote! { #input_name_ident: &<#input_managed_type_ident as NativeConvertible>::Native });
    }

    Ok(
        quote! {#(#inputs_idents), *}
    )
}

fn impl_endpoint_key_for_query(endpoint_name: &str, abi_inputs: &AbiInputs) -> TokenStream {
    let mut inputs_hash_idents: Vec<TokenStream> = vec![];
    for input in abi_inputs {
        let input_name_ident = format_ident!("{}", input.name);

        inputs_hash_idents.push(quote!{#input_name_ident.hash(&mut _novax_hasher);})
    }
    quote! {
        let mut _novax_hasher = DefaultHasher::new();
        _novax_contract_address_value.value.hash(&mut _novax_hasher);
        #endpoint_name.hash(&mut _novax_hasher);
        #(#inputs_hash_idents)*
        let _novax_key = _novax_hasher.finish();
    }
}

// (TokenStream, TokenStream) = (let statements, function params)
fn impl_endpoint_args_for_call(abi_inputs: &AbiInputs, all_abi_types: &AbiTypes) -> Result<(TokenStream, Vec<TokenStream>), BuildError> {
    let static_api = get_api_generic_ident();
    let mut inputs_let_idents: Vec<TokenStream> = vec![];
    let mut inputs_function_arg_idents: Vec<TokenStream> = vec![];
    for input in abi_inputs {
        let input_name_ident = format_ident!("r#{}", input.name);
        let input_managed_name_ident = format_ident!("_novax_argument_{}", input.name);
        let input_managed_type = get_managed_type_for_abi_type(&input.r#type, all_abi_types, &static_api)?;
        inputs_let_idents.push(quote! {
            let #input_managed_name_ident: #input_managed_type = #input_name_ident.to_managed();
        });
        inputs_function_arg_idents.push(quote! {
            {
                let mut _novax_top_encoded_args = ManagedVec::<#static_api, ManagedBuffer<#static_api>>::new();
                _ = #input_managed_name_ident.multi_encode(&mut _novax_top_encoded_args);

                for _novax_top_encoded_arg in _novax_top_encoded_args.into_iter() {
                    _novax_bytes_args.push(_novax_top_encoded_arg.to_boxed_bytes().into_vec());
                }
            }

        });
    }
    let let_statements_result = quote! {
        #(#inputs_let_idents)*
    };

    Ok((
        let_statements_result,
        inputs_function_arg_idents
    ))
}

// (TokenStream, TokenStream) = (managed_outputs, native_outputs)
fn impl_endpoint_output_for_query(abi_output: &AbiOutput, abi_types: &AbiTypes, api_generic: &TokenStream) -> Result<(TokenStream, TokenStream), BuildError> {
    let output_type_ident = get_managed_type_for_abi_type(&abi_output.r#type, abi_types, api_generic)?;

    Ok((
        quote! {#output_type_ident},
        quote! {<#output_type_ident as NativeConvertible>::Native}
    ))
}

fn impl_event_input_for_query(abi_event_input: &AbiEventInput, abi_types: &AbiTypes, api_generic: &TokenStream) -> Result<(TokenStream, TokenStream), BuildError> {
    let output_type_ident = get_managed_type_for_abi_type(&abi_event_input.r#type, abi_types, api_generic)?;

    Ok((
        quote! {#output_type_ident},
        quote! {<#output_type_ident as NativeConvertible>::Native}
    ))
}


fn get_managed_type_for_abi_type(abi_type_name: &str, abi_types: &AbiTypes, api_generic: &TokenStream) -> Result<TokenStream, BuildError> {
    if abi_types.get(abi_type_name).is_some() {
        let managed_name = get_native_struct_managed_name(abi_type_name);
        let managed_type_ident = format_ident!("{}", managed_name);
        Ok(
            quote! {#managed_type_ident}
        )
    } else {
        if abi_type_name.starts_with("multi") {
            panic!("bouhhh");
        }
        Ok(
            parse_abi_type_name_to_managed_ident(
                abi_type_name,
                abi_types,
                api_generic
            )?
        )
    }
}