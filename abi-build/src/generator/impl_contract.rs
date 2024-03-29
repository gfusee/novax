use convert_case::Case;
use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use crate::abi::constructor::{AbiConstructor, AbiOutputs};
use crate::abi::result::{Abi, AbiEndpoints, AbiTypes};
use crate::abi::endpoint::{AbiEndpoint, AbiInputs, AbiPossibleMutability};
use crate::abi::output::AbiOutput;
use crate::errors::build_error::BuildError;
use crate::utils::get_api_generic_ident::get_api_generic_ident;
use crate::utils::get_native_struct_managed_name::get_native_struct_managed_name;
use crate::utils::parse_abi_type_name_to_managed_ident::parse_abi_type_name_to_managed_ident;

pub(crate) fn impl_contract(mod_name: &str, abi: &Abi) -> Result<TokenStream, BuildError> {
    let mod_name_ident = format_ident!("{}", mod_name);
    let name = format_ident!("{}", abi.get_contract_name());
    let proxy_name = abi.get_proxy_name();
    let proxy_name_ident = format_ident!("{}", proxy_name);
    let proxy_mod_name_ident = format_ident!("{}", proxy_name.to_case(Case::Snake));
    let contract_info_name = format!("{name}Infos");
    let contract_info_ident = format_ident!("{}", contract_info_name);
    let call_name = format_ident!("{}", abi.get_call_name());
    let query_name = format_ident!("{}", abi.get_query_name());
    let (calls_impls, queries_impls) = impl_abi_endpoints(&contract_info_name, &abi.endpoints, &abi.types)?;
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
fn impl_abi_endpoints(contract_info_name: &str, abi_endpoints: &AbiEndpoints, abi_types: &AbiTypes) -> Result<(TokenStream, TokenStream), BuildError> {
    let mut calls_impls: Vec<TokenStream> = vec![];
    let mut queries_impls: Vec<TokenStream> = vec![];
    for endpoint in abi_endpoints {
        let endpoint_impls = impl_abi_endpoint_call_query(contract_info_name, endpoint, abi_types)?;

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

fn impl_abi_endpoint_call_query(contract_info_name: &str, abi_endpoint: &AbiEndpoint, abi_types: &AbiTypes) -> Result<(TokenStream, TokenStream), BuildError> {
    let debug_api = get_api_generic_ident();
    let contract_info_ident = format_ident!("{}", contract_info_name);
    let function_name = format_ident!("{}", abi_endpoint.name.to_case(Case::Snake));
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
        let _novax_contract_address_value: AddressValue = (&Address::from(&self.contract_address)).into();
        let mut _novax_contract = #contract_info_ident::new(&_novax_contract_address_value); // unnecessary clone when calling

        #endpoint_args_let_statements

        let _novax_contract_call = _novax_contract
            .#function_name(#endpoint_args_inputs);

        let mut _novax_tx: TypedScCall<#function_managed_outputs> = ScCallStep::new()
            .call(_novax_contract_call)
            .into();

        if self.egld_value > num_bigint::BigUint::from(0u8) {
            _novax_tx = _novax_tx
            .egld_value(self.egld_value.clone());
        }

        _novax_tx = _novax_tx
            .expect(TxExpect::ok());
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
        pub async fn #function_name(#function_inputs) -> Result<CallResult<#function_native_outputs>, NovaXError> {
            #common_token

            _novax_tx = _novax_tx
                .gas_limit(self.gas_limit);

            for _novax_transfer in &self.token_transfers {
                _novax_tx = _novax_tx
                .esdt_transfer(
                    format!("str:{}", _novax_transfer.identifier.clone()),
                    _novax_transfer.nonce.clone(),
                    _novax_transfer.amount.clone()
                );
            }

            let _novax_should_skip_deserialization = async {
                self.executor.sc_call(&mut _novax_tx).await?;

                Result::Ok::<bool, NovaXError>(self.executor.should_skip_deserialization().await)
            }.await?;

            let (_novax_result, _novax_response) = {
                let mut result = None;
                let mut opt_response = None;
                if !_novax_should_skip_deserialization {
                    if let Result::Ok(_novax_parsed_result) = _novax_tx.result::<#function_managed_outputs>() {
                        result = Some(_novax_parsed_result);
                        opt_response = Some(_novax_tx.response().clone());
                    }
                }

                (result, opt_response.unwrap_or_else(|| TxResponse::default()))
            };


             let mut _novax_call_result = CallResult {
                response: _novax_response,
                result: _novax_result.to_native()
            };

            Result::Ok(_novax_call_result)
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
        pub async fn #function_name(#function_inputs) -> Result<#function_native_outputs, NovaXError> {
            #common_token
            #endpoint_query_key
            self.caching.get_or_set_cache(
                _novax_key,
                async {
                    let result = self.executor.execute::<#function_managed_outputs>(&_novax_tx.sc_call_step).await;

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

fn impl_abi_constructor(contract_info_name: &str, abi_constructor: &AbiConstructor, abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let debug_api = get_api_generic_ident();
    let contract_info_ident = format_ident!("{}", contract_info_name);
    let function_name = format_ident!("deploy");
    let function_inputs = impl_endpoint_inputs(false, &abi_constructor.inputs, abi_types)?;
    let (endpoint_args_let_statements, endpoint_args_inputs) = impl_endpoint_args_for_call(&abi_constructor.inputs, abi_types)?;
    let (function_managed_outputs, function_native_outputs) = impl_endpoint_outputs(&abi_constructor.outputs, abi_types, &debug_api)?;

    let function_token = quote! {
        /// This asynchronous function encapsulates the logic for deploying a smart contract to the blockchain.
        /// It takes in a `DeployData` instance, an executor, a gas limit, and additional function-specific inputs
        /// to facilitate the deployment process.
        ///
        /// # Type Parameters
        ///
        /// - `Code`: This type represents the contract code and must implement the `AsBytesValue` trait to ensure
        ///   it can be properly serialized for deployment.
        /// - `Executor`: This type represents the executor that will carry out the deployment transaction and
        ///   must implement the `DeployExecutor` trait.
        ///
        /// # Parameters
        ///
        /// - `_novax_deploy_data`: An instance of `DeployData` containing the contract code and metadata necessary
        ///   for deployment.
        /// - `_novax_executor`: A mutable reference to an executor that will perform the deployment transaction.
        /// - `gas_limit`: The maximum amount of gas that can be consumed during the deployment process.
        /// - `#function_inputs`: Additional inputs required for deploying the contract, as defined in the contract's ABI.
        ///
        /// # Returns
        ///
        /// A `Result` containing a tuple with the following elements upon successful deployment:
        ///
        /// - `Address`: The address of the newly deployed contract on the blockchain.
        /// - `CallResult`: A `CallResult` instance containing the response data from the contract's deployment.
        ///
        /// Or a `NovaXError` if the deployment process fails.
        pub async fn #function_name<Code: AsBytesValue, Executor: DeployExecutor>(_novax_deploy_data: DeployData<Code>, _novax_executor: &mut Executor, gas_limit: u64, #function_inputs) -> Result<(Address, CallResult<#function_native_outputs>), NovaXError> {
            let mut _novax_contract = #contract_info_ident::new(&multiversx_sc::types::Address::from(<[u8;32]>::default()));

            #endpoint_args_let_statements

            let _novax_code_bytes = _novax_deploy_data.code.into_bytes_value().await?;

            let mut _novax_deploy_step = ScDeployStep::new()
                .call(_novax_contract.init(#endpoint_args_inputs))
                .gas_limit(gas_limit)
                .code(_novax_code_bytes)
                .code_metadata(_novax_deploy_data.metadata);

            let _novax_should_skip_deserialization = async {
                _novax_executor.sc_deploy(&mut _novax_deploy_step).await?;

                Result::Ok::<bool, NovaXError>(_novax_executor.should_skip_deserialization().await)
            }.await?;

            let (_novax_new_address, _novax_result, _novax_response) = {
                let mut result = None;
                let mut opt_response = None;
                let mut new_address = multiversx_sc::types::Address::from(<[u8;32]>::default());
                if !_novax_should_skip_deserialization {
                    if let Result::Ok(_novax_parsed_result) = _novax_deploy_step.result::<#function_managed_outputs>() {
                        let _novax_response = _novax_deploy_step.response().clone();
                        new_address = _novax_response.new_deployed_address.clone().unwrap();
                        result = Some(_novax_parsed_result);
                        opt_response = Some(_novax_response);
                    }
                }

                (Address::from(new_address), result, opt_response.unwrap_or_else(|| TxResponse::default()))
            };


            let mut _novax_call_result = CallResult {
                response: _novax_response,
                result: _novax_result.to_native()
            };

            Result::Ok((_novax_new_address, _novax_call_result))
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
fn impl_endpoint_args_for_call(abi_inputs: &AbiInputs, all_abi_types: &AbiTypes) -> Result<(TokenStream, TokenStream), BuildError> {
    let static_api = get_api_generic_ident();
    let mut inputs_let_idents: Vec<TokenStream> = vec![];
    let mut inputs_function_arg_idents: Vec<TokenStream> = vec![];
    for input in abi_inputs {
        let input_name_ident = format_ident!("r#{}", input.name);
        let input_managed_name_ident = format_ident!("_novax_{}", input.name);
        let input_managed_type = get_managed_type_for_abi_type(&input.r#type, all_abi_types, &static_api)?;
        inputs_let_idents.push(quote! {
            let #input_managed_name_ident: #input_managed_type = #input_name_ident.to_managed();
        });
        inputs_function_arg_idents.push(input_managed_name_ident.into_token_stream());
    }
    let let_statements_result = quote! {
        #(#inputs_let_idents)*
    };
    let function_inputs_result = quote! {
        #(#inputs_function_arg_idents), *
    };

    Ok((
        let_statements_result,
        function_inputs_result
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