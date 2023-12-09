use proc_macro2::{TokenStream};
use quote::{format_ident, quote};
use regex::Regex;
use crate::abi::result::AbiTypes;
use crate::errors::build_error::BuildError;
use crate::generator::generator_error::GeneratorError::TypeNotFoundForInput;
use std::backtrace::Backtrace;
use std::str::FromStr;
use crate::utils::get_native_struct_managed_name::get_native_struct_managed_name;

pub(crate) fn parse_abi_type_name_to_managed_ident(abi_type: &str, all_abi_types: &AbiTypes, api_generic: &TokenStream) -> Result<TokenStream, BuildError> {
    if let Some(sub_types) = parse_multi_type("^optional<multi<(.+)>>$", abi_type) {
        let mut sub_types_idents: Vec<TokenStream> = vec![];
        for sub_type in &sub_types {
            let sub_type_ident_to_push = parse_abi_type_name_to_managed_ident(sub_type, all_abi_types, api_generic)?;
            sub_types_idents.push(sub_type_ident_to_push);
        }

        let multi_value_ident = format_ident!("MultiValue{}", sub_types.len());

        return Ok(
            quote! { OptionalValue<#multi_value_ident<#(#sub_types_idents), *>> }
        );
    };

    if let Some(sub_types) = parse_multi_type("^variadic<multi<(.+)>>$", abi_type) {
        let variadic_regex = Regex::new(r"^variadic<(.+)>$").unwrap();
        if variadic_regex.is_match(abi_type) {
            let mut sub_types_idents: Vec<TokenStream> = vec![];
            for sub_type in &sub_types {
                let sub_type_ident_to_push = parse_abi_type_name_to_managed_ident(sub_type, all_abi_types, api_generic)?;
                sub_types_idents.push(sub_type_ident_to_push);
            }

            let multi_value_ident = format_ident!("MultiValue{}", sub_types.len());

            return Ok(
                quote! { MultiValueVec<#multi_value_ident<#(#sub_types_idents), *>> }
            )
        };
    } else {
        let variadic_regex = Regex::new(r"^variadic<(.+)>$").unwrap();
        if variadic_regex.is_match(abi_type) {
            let sub_type = variadic_regex.captures_at(abi_type, 0).unwrap().get(1).unwrap().as_str();
            let sub_type_ident = parse_abi_type_name_to_managed_ident(sub_type, all_abi_types, api_generic)?;

            return Ok(
                quote! { MultiValueEncoded<#api_generic, #sub_type_ident> }
            )
        };
    }

    let option_regex = Regex::new(r"^Option<(.+)>$").unwrap();
    if option_regex.is_match(abi_type) {
        let sub_type = option_regex.captures_at(abi_type, 0).unwrap().get(1).unwrap().as_str();
        let sub_type_ident = parse_abi_type_name_to_managed_ident(sub_type, all_abi_types, api_generic)?;

        return Ok(
            quote! { Option<#sub_type_ident> }
        )
    };

    let optional_regex = Regex::new(r"^optional<(.+)>$").unwrap();
    if optional_regex.is_match(abi_type) {
        let sub_type = optional_regex.captures_at(abi_type, 0).unwrap().get(1).unwrap().as_str();
        let sub_type_ident = parse_abi_type_name_to_managed_ident(sub_type, all_abi_types, api_generic)?;

        return Ok(
            quote! { OptionalValue<#sub_type_ident> }
        )
    };

    let list_regex = Regex::new(r"^List<(.+)>$").unwrap();
    if list_regex.is_match(abi_type) {
        let sub_type = list_regex.captures_at(abi_type, 0).unwrap().get(1).unwrap().as_str();
        let sub_type_ident = parse_abi_type_name_to_managed_ident(sub_type, all_abi_types, api_generic)?;

        return Ok(
            quote! { ManagedVec<#api_generic, #sub_type_ident> }
        )
    };

    if let Some(sub_types) = parse_multi_type("^tuple<(.+)>$", abi_type) {
        let mut sub_types_idents: Vec<TokenStream> = vec![];
        for sub_type in &sub_types {
            let sub_type_ident_to_push = parse_abi_type_name_to_managed_ident(sub_type, all_abi_types, api_generic)?;
            sub_types_idents.push(sub_type_ident_to_push);
        }

        return Ok(
            quote! { (#(#sub_types_idents), *) }
        );
    };

    let bytes_array_regex = Regex::new(r"^array([0-9]+)<u8>$").unwrap();
    if bytes_array_regex.is_match(abi_type) {
        let len = usize::from_str(bytes_array_regex.captures_at(abi_type, 0).unwrap().get(1).unwrap().as_str()).unwrap();

        return Ok(
            quote! { [u8; #len] }
        )
    };

    match abi_type {
        "bytes" => Ok(quote! {ManagedBuffer<#api_generic>}),
        "Address" => Ok(quote! {ManagedAddress<#api_generic>}),
        "TokenIdentifier" => Ok(quote! {TokenIdentifier<#api_generic>}),
        "EgldOrEsdtTokenIdentifier" => Ok(quote! {EgldOrEsdtTokenIdentifier<#api_generic>}),
        "BigUint" => Ok(quote! {BigUint<#api_generic>}),
        "BigInt" => Ok(quote! {BigInt<#api_generic>}),
        "u64" => Ok(quote! {u64}),
        "u32" => Ok(quote! {u32}),
        "u16" => Ok(quote! {u16}),
        "u8" => Ok(quote! {u8}),
        "i64" => Ok(quote! {i64}),
        "i32" => Ok(quote! {i32}),
        "i16" => Ok(quote! {i16}),
        "i8" => Ok(quote! {i8}),
        "bool" => Ok(quote! {bool}),
        "CodeMetadata" => Ok(quote! {CodeMetadata}),
        _ => {
            if all_abi_types.contains_key(abi_type) {
                let ident = format_ident!("{}", get_native_struct_managed_name(abi_type));
                Ok(quote! { #ident })
            } else {
                println!("unknown type : {abi_type}");
                println!("all types :");
                for name in all_abi_types.keys() {
                    println!("{name}");
                }
                println!("Custom backtrace: {}", Backtrace::force_capture());
                Err(TypeNotFoundForInput.into())
            }
        }
    }
}


fn parse_multi_type(regex_str: &str, text: &str) -> Option<Vec<String>> {
    let regex = Regex::new(regex_str).unwrap();
    if regex.is_match(text) {
        let full_sub_types = regex.captures_at(text, 0).unwrap().get(1).unwrap().as_str();
        let mut sub_types: Vec<String> = vec![];
        let mut generic_count = 0usize;
        let mut last_start_position = 0usize;
        let mut current_position = 0usize;
        let full_sub_types_chars = full_sub_types.chars();
        let full_sub_types_string = String::from(full_sub_types);
        for char in full_sub_types_chars {
            if char == '<' {
                generic_count += 1;
            } else if char == '>' {
                generic_count -= 1;
            }

            if char == ',' && generic_count == 0 {
                let actual_sub_type = &full_sub_types_string[last_start_position..current_position];
                sub_types.push(actual_sub_type.to_string());
                last_start_position = current_position + 1;
            }

            current_position += 1;
        }

        let last_sub_type = &full_sub_types_string[last_start_position..current_position];
        sub_types.push(last_sub_type.to_string());

        Some(sub_types)
    } else {
        None
    }
}