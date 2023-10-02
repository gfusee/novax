use base64::Engine;
use multiversx_sc::codec::TopDecodeMulti;
use crate::error::DataError;
use crate::error::UtilsError::CannotParseQueryResult;
use crate::types::native::NativeConvertible;

/// Parses and decodes a slice of base64-encoded strings obtained from a blockchain gateway query,
/// or a mocked environment, into a native Rust type.
///
/// This function first decodes the base64-encoded strings into bytes, then delegates the parsing
/// of the byte data to `parse_query_return_bytes_data`.
///
/// # Type Parameters
/// - `T`: The native Rust type to which the data should be parsed, which must implement both
///        `NativeConvertible` and `TopDecodeMulti` traits.
///
/// # Parameters
/// - `data`: A slice of base64-encoded strings representing the data to be parsed.
///
/// # Returns
/// A `Result` containing the parsed native type, or a `DataError` if parsing fails.
///
/// # Examples
///
/// ```rust
/// # use novax_data::DataError;
/// # use novax_data::parse_query_return_string_data;
/// let data = vec!["AhhxGgA="];
/// let result: Result<u64, DataError> = parse_query_return_string_data(&data);
///
/// assert_eq!(result.unwrap(), 9000000000_u64)
/// ```
pub fn parse_query_return_string_data<T: NativeConvertible + TopDecodeMulti>(
    data: &[&str]
) -> Result<T, DataError> {
    let mut data_to_parse: Vec<Vec<u8>> = vec![];
    for data in data {
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data) else {return Err(CannotParseQueryResult.into()) };
        data_to_parse.push(bytes)
    }

    parse_query_return_bytes_data(&mut data_to_parse)
}

/// Parses and decodes a vector of byte vectors obtained from a blockchain gateway query,
/// or a mocked environment, into a native Rust type.
///
/// This function utilizes the `TopDecodeMulti` trait to parse the byte data into the desired
/// native type `T`.
///
/// # Type Parameters
/// - `T`: The native Rust type to which the data should be parsed, which must implement both
///        `NativeConvertible` and `TopDecodeMulti` traits.
///
/// # Parameters
/// - `data`: A mutable reference to a vector of byte vectors representing the data to be parsed.
///
/// # Returns
/// A `Result` containing the parsed native type, or a `DataError` if parsing fails.
pub fn parse_query_return_bytes_data<T: NativeConvertible + TopDecodeMulti>(
    data: &mut Vec<Vec<u8>>
) -> Result<T, DataError> {
    let Ok(result) = T::multi_decode(data) else { return Err(CannotParseQueryResult.into()) };

    Ok(result)
}