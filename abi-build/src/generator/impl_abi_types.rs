use convert_case::Case;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use crate::abi::result::AbiTypes;
use crate::abi::r#type::{AbiPossibleType, AbiType, AbiTypeFields, AbiTypeVariants};
use crate::errors::build_error::BuildError;
use crate::utils::get_api_generic_ident::get_api_generic_ident;
use crate::utils::get_native_struct_managed_name::get_native_struct_managed_name;
use convert_case::Casing;

pub(crate) fn impl_abi_types_mod(abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let mut abi_types_impl: Vec<TokenStream> = vec![];
    for abi_type_key in abi_types.clone().into_keys() {
        let abi_type = abi_types.get(&abi_type_key).unwrap();

        let abi_type_impl = match abi_type.r#type {
            AbiPossibleType::Struct => impl_abi_struct_type(&abi_type_key, abi_type, abi_types)?,
            AbiPossibleType::Enum => impl_abi_enum_type(&abi_type_key, abi_type, abi_types)?
        };

        abi_types_impl.push(abi_type_impl)
    }
    Ok(
        quote! {
            #(#abi_types_impl)*
        }
    )
}

fn impl_abi_enum_type(name: &String, abi_type: &AbiType, all_abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let native_name_ident = format_ident!("{}", name);
    let managed_name = get_native_struct_managed_name(name);
    let managed_name_ident = format_ident!("{}", managed_name);
    let managed_name_ident_mod = format_ident!("{}_mod", managed_name.to_case(Case::Snake));
    let variants = abi_type.variants.clone().unwrap();
    let (managed_values, native_values) = impl_abi_enum_values(&variants, all_abi_types)?;
    let to_discriminant_impl = impl_abi_enum_to_discriminant(name, abi_type);
    let native_convertible_impl = impl_abi_enum_native_convertible(&native_name_ident, &managed_name_ident, &variants)?;
    let managed_convertible_impl = impl_abi_enum_managed_convertible(&native_name_ident, &managed_name_ident, &variants)?;
    let derive_impl = impl_abi_enum_managed_derives(&variants);
    let result = quote! {
        #[allow(missing_docs)]
        mod #managed_name_ident_mod {
            use super::*;

            /// Represents the managed version of the enum `#native_name_ident` as defined in the ABI.
            ///
            /// This enum encapsulates variants that have a corresponding managed representation in the smart contract.
            /// It's typically used internally for serialization and deserialization to and from the smart contract,
            /// as well as for other operations that interact directly with the smart contract's ABI.
            #derive_impl
            pub enum #managed_name_ident {
                #managed_values
            }
        }

        use #managed_name_ident_mod::*;

        /// Represents the native version of the enum `#managed_name_ident` as defined in the ABI.
        ///
        /// This enum encapsulates variants that have a corresponding native representation in Rust.
        /// It's designed for more straightforward interaction in Rust code, and can be converted to and from its
        /// corresponding managed representation (`#managed_name_ident`) using the provided trait implementations.
        #[derive(Serialize, Deserialize, PartialEq, Hash, Clone, Debug)]
        pub enum #native_name_ident {
            #native_values
        }

        impl #native_name_ident {
            /// Constructs an instance of `#native_name_ident` from ESDT token attributes.
            ///
            /// This function attempts to decode the provided ESDT attributes into an instance of `#native_name_ident`.
            /// It is specifically designed to work with the attributes associated with ESDT tokens, which are typically
            /// encoded in a binary format.
            ///
            /// # Arguments
            /// - `attributes`: A byte slice (`&[u8]`) representing the ESDT token attributes to be decoded.
            ///
            /// # Returns
            /// - `Ok(#native_name_ident)`: Successfully decoded instance of `#native_name_ident`.
            /// - `Err(NovaXError)`: An error wrapped in `NovaXError`, specifically `CodingError::CannotDecodeEsdtAttributes`,
            ///   if the decoding process fails. This error indicates that the provided attributes could not be properly
            ///   decoded into the expected `#native_name_ident` type.
            pub fn from_esdt_attributes(attributes: &[u8]) -> Result<#native_name_ident, NovaXError> {
                let Result::Ok(decoded) = #managed_name_ident::top_decode(attributes) else {
                    return Result::Err(CodingError::CannotDecodeEsdtAttributes.into());
                };

                Result::Ok(decoded.to_native())
            }
        }

        #native_convertible_impl

        #managed_convertible_impl

        #to_discriminant_impl
    };

    Ok(result)
}

fn impl_abi_enum_to_discriminant(name: &String, abi_type: &AbiType) -> TokenStream {
    let name_ident = format_ident!("{}", name);
    let mut match_cases: Vec<TokenStream> = vec![];
    let variants = abi_type.variants.clone().unwrap();
    for variant in variants {
        let variant_name_ident = format_ident!("{}", variant.name);
        let variant_discriminant = variant.discriminant;
        match_cases.push(quote! {
             #name_ident::#variant_name_ident { .. } => Result::Ok(#variant_discriminant)
        })
    }

    quote! {
        impl #name_ident {
            /// This function converts an enum variant to its discriminant (u8) representation as per the
            /// smart contract's understanding. Each variant is mapped to a unique u8 value starting from 0,
            /// incrementing by 1 for each subsequent variant.
            ///
            /// # Errors
            /// Returns `NovaXError` if the conversion fails, which could occur if the enum has a variant
            /// that is not accounted for in the conversion logic.
            pub fn to_discriminant(&self) -> Result<u8, NovaXError> {
                match self {
                    #(#match_cases), *,
                }
            }
        }
    }
}

fn impl_abi_enum_managed_derives(variants: &AbiTypeVariants) -> TokenStream {
    let mut derive_idents: Vec<Ident> = vec![];
    for derive in ["TopEncode", "TopDecode", "NestedEncode", "NestedDecode", "Clone", "Debug"] {
        let ident = format_ident!("{}", derive);
        derive_idents.push(ident);
    }

    if is_enum_fieldless(variants) {
        derive_idents.push(format_ident!("{}", "ManagedVecItem"))
    }

    quote! {#[derive(#(#derive_idents), *)]}
}

// (TokenStream, TokenStream) = (managed, native)
fn impl_abi_enum_values(variants: &AbiTypeVariants, all_abi_types: &AbiTypes) -> Result<(TokenStream, TokenStream), BuildError> {
    let debug_api = get_api_generic_ident();
    let mut native_names_idents: Vec<TokenStream> = vec![];
    let mut managed_names_idents: Vec<TokenStream> = vec![];
    for variant in variants {
        let variant_name_ident = format_ident!("{}", variant.name);
        let (managed_name_ident, native_name_ident) = if let Some(fields) = &variant.fields {
            let mut managed_field_idents: Vec<TokenStream> = vec![];
            let mut native_field_idents: Vec<TokenStream> = vec![];
            let mut has_named_fields = true;
            for field in fields {
                let managed_field_ident = &field.r#type.get_managed_type_ident(&debug_api, all_abi_types)?;
                let (managed_field_ident, native_field_ident) = if let Some(field_name_ident) = field.get_enum_field_name() {
                    let field_name_ident = format_ident!("r#{}", field_name_ident);
                    (
                        quote! {#field_name_ident: #managed_field_ident},
                        quote! {#field_name_ident: <#managed_field_ident as NativeConvertible>::Native}
                    )
                } else {
                    has_named_fields = false;
                    (
                        quote! {#managed_field_ident},
                        quote! {<#managed_field_ident as NativeConvertible>::Native}
                    )
                };

                managed_field_idents.push(managed_field_ident);
                native_field_idents.push(native_field_ident);
            }

            let (managed_fields, native_fields) = if has_named_fields {
                (
                    quote! {#variant_name_ident{#(#managed_field_idents), *}},
                    quote! {#variant_name_ident{#(#native_field_idents), *}}
                )
            } else {
                (
                    quote! {#variant_name_ident(#(#managed_field_idents), *)},
                    quote! {#variant_name_ident(#(#native_field_idents), *)}
                )
            };

            (managed_fields, native_fields)
        } else {
            let quote = quote! {#variant_name_ident};

            (quote.clone(), quote)
        };
        managed_names_idents.push(quote! {#managed_name_ident});
        native_names_idents.push(quote! {#native_name_ident});
    }

    let native_names_idents: Vec<TokenStream> = native_names_idents.into_iter()
        .map(|ident| {
            quote! {
                #[allow(missing_docs)]
                #ident
            }
        })
        .collect();

    Ok((
        quote! {#(#managed_names_idents), *},
        quote! {#(#native_names_idents), *}
    ))
}

fn impl_abi_enum_native_convertible(native_name_ident: &Ident, managed_name_ident: &Ident, variants: &AbiTypeVariants) -> Result<TokenStream, BuildError> {
    let mut match_cases: Vec<TokenStream> = vec![];

    for variant in variants {
        let variant_name_ident = format_ident!("{}", variant.name);
        if let Some(fields) = &variant.fields {
            let mut managed_side_field_name_idents: Vec<Ident> = vec![];
            let mut native_side_field_name_idents: Vec<TokenStream> = vec![];
            let mut has_named_fields = true;
            for field_index in 0..fields.len() {
                let field = fields.get(field_index).unwrap();
                let opt_field_name_ident = field.get_enum_field_name();
                if let Some(field_name_ident) = opt_field_name_ident {
                    managed_side_field_name_idents.push(field_name_ident.clone());
                    native_side_field_name_idents.push(quote!{#field_name_ident: #field_name_ident.to_native()});
                } else {
                    has_named_fields = false;
                    let field_name_ident = format_ident!("field_{}", field_index);
                    managed_side_field_name_idents.push(field_name_ident.clone());
                    native_side_field_name_idents.push(quote!{#field_name_ident.to_native()});
                }
            }
            let managed_side_all_field_name_ident = quote! {#(#managed_side_field_name_idents), *};
            let native_side_all_field_name_ident = quote! {#(#native_side_field_name_idents), *};
            let case = if has_named_fields {
                quote! {#managed_name_ident::#variant_name_ident{#managed_side_all_field_name_ident} => #native_name_ident::#variant_name_ident{#native_side_all_field_name_ident}}
            } else {
                quote! {#managed_name_ident::#variant_name_ident(#managed_side_all_field_name_ident) => #native_name_ident::#variant_name_ident(#native_side_all_field_name_ident)}
            };
            match_cases.push(case);
        } else {
            let case = quote! {#managed_name_ident::#variant_name_ident => #native_name_ident::#variant_name_ident};
            match_cases.push(case);
        }
    }

    let result = quote! {
        impl NativeConvertible for #managed_name_ident {
            type Native = #native_name_ident;

            fn to_native(&self) -> Self::Native {
                match self {
                    #(#match_cases), *
                }
            }
        }
    };

    Ok(result)
}

fn impl_abi_enum_managed_convertible(native_name_ident: &Ident, managed_name_ident: &Ident, variants: &AbiTypeVariants) -> Result<TokenStream, BuildError> {
    let mut match_cases: Vec<TokenStream> = vec![];

    for variant in variants {
        let variant_name_ident = format_ident!("{}", variant.name);
        if let Some(fields) = &variant.fields {
            let mut managed_side_field_name_idents: Vec<TokenStream> = vec![];
            let mut native_side_field_name_idents: Vec<Ident> = vec![];
            let mut has_named_fields = true;
            for field_index in 0..fields.len() {
                let field = fields.get(field_index).unwrap();
                let opt_field_name_ident = field.get_enum_field_name();
                if let Some(field_name_ident) = opt_field_name_ident {
                    managed_side_field_name_idents.push(quote!{#field_name_ident: #field_name_ident.to_managed()});
                    native_side_field_name_idents.push(field_name_ident);
                } else {
                    has_named_fields = false;
                    let field_name_ident = format_ident!("field_{}", field_index);
                    managed_side_field_name_idents.push(quote!{#field_name_ident.to_managed()});
                    native_side_field_name_idents.push(field_name_ident);
                }
            }
            let managed_side_all_field_name_ident = quote! {#(#managed_side_field_name_idents), *};
            let native_side_all_field_name_ident = quote! {#(#native_side_field_name_idents), *};
            let case = if has_named_fields {
                quote! {#native_name_ident::#variant_name_ident{#native_side_all_field_name_ident} => #managed_name_ident::#variant_name_ident{#managed_side_all_field_name_ident}}
            } else {
                quote! {#native_name_ident::#variant_name_ident(#native_side_all_field_name_ident) => #managed_name_ident::#variant_name_ident(#managed_side_all_field_name_ident)}
            };
            match_cases.push(case);
        } else {
            let case = quote! {#native_name_ident::#variant_name_ident => #managed_name_ident::#variant_name_ident};
            match_cases.push(case);
        }
    }

    let result = quote! {
        impl ManagedConvertible<#managed_name_ident> for #native_name_ident {
            fn to_managed(&self) -> #managed_name_ident {
               match self {
                    #(#match_cases), *
                }
            }
        }
    };

    Ok(result)
}

fn impl_abi_struct_type(name: &str, abi_type: &AbiType, all_abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let managed_name = get_native_struct_managed_name(name);
    let managed_name_ident = format_ident!("{}", managed_name);
    let managed_name_type_wrap_api = format_ident!("{}Type", managed_name);
    let abi_fields = abi_type.fields.clone().unwrap();
    let managed_fields_impl = impl_abi_struct_managed_fields(&abi_fields, all_abi_types)?;
    let native_name_ident = format_ident!("{}", name);
    let native_fields_impl = impl_abi_struct_native_fields(&abi_fields, all_abi_types)?;
    let native_convertible_fields_init = impl_abi_struct_native_convertible_fields(&abi_fields)?;
    let managed_convertible_fields_init = impl_abi_struct_managed_convertible_fields(&abi_fields)?;
    let derive_impls = impl_abi_struct_derive(&abi_fields, all_abi_types);
    Ok(
        quote! {
            /// Derive implementations for the structs.
            #derive_impls
            /// Represents the managed version of type `#native_name_ident` as defined in the ABI.
            ///
            /// This structure encapsulates fields that have a corresponding managed representation in the smart contract.
            /// It's typically used internally for serialization and deserialization to and from the smart contract,
            /// as well as for other operations that interact directly with the smart contract's ABI.
            pub struct #managed_name_ident {
                #managed_fields_impl
            }

            /// Represents the native version of type `#managed_name_ident` as defined in the ABI.
            ///
            /// This structure encapsulates fields that have a corresponding native representation in Rust.
            /// It's designed for more straightforward interaction in Rust code, and can be converted to and from its
            /// corresponding managed representation (`#managed_name_ident`) using the provided trait implementations.
            #[derive(Serialize, Deserialize, PartialEq, Hash, Clone, Debug)]
            pub struct #native_name_ident {
                #native_fields_impl
            }

            /// Provides a mechanism for converting a `#managed_name_ident` to its native representation (`#native_name_ident`).
            impl NativeConvertible for #managed_name_ident {
                type Native = #native_name_ident;

                /// Converts the `#managed_name_ident` to its native representation.
                ///
                /// # Returns
                /// A `#native_name_ident` instance representing the same data as the `#managed_name_ident`.
                fn to_native(&self) -> Self::Native {
                    #native_name_ident {
                        #native_convertible_fields_init
                    }
                }
            }

            /// Type alias for wrapping the managed API.
            type #managed_name_type_wrap_api = #managed_name_ident;

            impl #native_name_ident {
                /// Constructs an instance of `#native_name_ident` from ESDT token attributes.
                ///
                /// This function attempts to decode the provided ESDT attributes into an instance of `#native_name_ident`.
                /// It is specifically designed to work with the attributes associated with ESDT tokens, which are typically
                /// encoded in a binary format.
                ///
                /// # Arguments
                /// - `attributes`: A byte slice (`&[u8]`) representing the ESDT token attributes to be decoded.
                ///
                /// # Returns
                /// - `Ok(#native_name_ident)`: Successfully decoded instance of `#native_name_ident`.
                /// - `Err(NovaXError)`: An error wrapped in `NovaXError`, specifically `CodingError::CannotDecodeEsdtAttributes`,
                ///   if the decoding process fails. This error indicates that the provided attributes could not be properly
                ///   decoded into the expected `#native_name_ident` type.
                pub fn from_esdt_attributes(attributes: &[u8]) -> Result<#native_name_ident, NovaXError> {
                    let Result::Ok(decoded) = #managed_name_ident::top_decode(attributes) else {
                        return Result::Err(CodingError::CannotDecodeEsdtAttributes.into());
                    };

                    Result::Ok(decoded.to_native())
                }
            }

            /// Provides a mechanism for converting a `#native_name_ident` to its managed representation (`#managed_name_ident`).
            impl ManagedConvertible<#managed_name_ident> for #native_name_ident {
                /// Converts the `#native_name_ident` to its managed representation.
                ///
                /// # Returns
                /// A `#managed_name_ident` instance representing the same data as the `#native_name_ident`.
                fn to_managed(&self) -> #managed_name_ident {
                    #managed_name_type_wrap_api {
                       #managed_convertible_fields_init
                    }
                }
            }
        }
    )
}

fn impl_abi_struct_derive(abi_fields: &AbiTypeFields, abi_types: &AbiTypes) -> TokenStream {
    let mut derive_idents: Vec<Ident> = vec![];
    for derive in ["TopEncode", "TopDecode", "NestedEncode", "NestedDecode", "Clone", "Debug"] {
        let ident = format_ident!("{}", derive);
        derive_idents.push(ident);
    }

    let should_include_managed_vec_item = should_struct_derive_managed_vec_item(abi_fields, abi_types);

    if should_include_managed_vec_item {
        derive_idents.push(format_ident!("{}", "ManagedVecItem"));
    }

    quote! {#[derive(#(#derive_idents), *)]}
}

fn impl_abi_struct_managed_fields(abi_fields: &AbiTypeFields, abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let debug_api = get_api_generic_ident();
    let mut impls: Vec<TokenStream> = vec![];

    for field in abi_fields {
        let field_name = format_ident!("r#{}", field.name.clone());
        let field_type = field.r#type.get_managed_type_ident(&debug_api, abi_types)?;
        impls.push(
            quote! {
                #[allow(missing_docs)]
                pub #field_name: #field_type,
            }
        )
    }

    Ok(
        quote! {
            #(#impls)*
        }
    )
}

fn impl_abi_struct_native_fields(abi_fields: &AbiTypeFields, all_abi_types: &AbiTypes) -> Result<TokenStream, BuildError> {
    let debug_api = get_api_generic_ident();
    let mut impls: Vec<TokenStream> = vec![];

    for field in abi_fields {
        let field_name = format_ident!("r#{}", field.name.clone());
        let managed_type_ident = field.r#type.get_managed_type_ident(&debug_api, all_abi_types)?;
        let field_type = quote! {<#managed_type_ident as NativeConvertible>::Native};
        impls.push(
            quote! {
                #[allow(missing_docs)]
                pub #field_name: #field_type,
            }
        )
    }

    Ok(
        quote! {
            #(#impls)*
        }
    )
}

fn impl_abi_struct_native_convertible_fields(abi_fields: &AbiTypeFields) -> Result<TokenStream, BuildError> {
    let mut impls: Vec<TokenStream> = vec![];

    for field in abi_fields {
        let field_name = format_ident!("r#{}", field.name.clone());
        impls.push(
            quote! {
                #field_name: self.#field_name.to_native(),
            }
        )
    }

    Ok(
        quote! {
            #(#impls)*
        }
    )
}

fn impl_abi_struct_managed_convertible_fields(abi_fields: &AbiTypeFields) -> Result<TokenStream, BuildError> {
    let mut impls: Vec<TokenStream> = vec![];

    for field in abi_fields {
        let field_name = format_ident!("r#{}", field.name.clone());
        impls.push(
            quote! {
                #field_name: self.#field_name.to_managed(),
            }
        )
    }

    Ok(
        quote! {
            #(#impls)*
        }
    )
}

fn is_enum_fieldless(variants: &AbiTypeVariants) -> bool {
    for variant in variants {
        if let Some(fields) = &variant.fields {
            if !fields.is_empty() {
                return false;
            }
        }
    }

    true
}

fn should_struct_derive_managed_vec_item(abi_fields: &AbiTypeFields, all_abi_types: &AbiTypes) -> bool {
    for field in abi_fields {
        let Some(abi_type) = all_abi_types.get(&field.r#type.0) else { continue };
        let should_derive = match abi_type.r#type {
            AbiPossibleType::Enum => {
                let variants = abi_type.variants.clone().unwrap();

                is_enum_fieldless(&variants)
            },
            AbiPossibleType::Struct => {
                should_struct_derive_managed_vec_item(&abi_type.fields.clone().unwrap(), all_abi_types)
            }
        };

        if !should_derive {
            return false
        }
    }

    true
}