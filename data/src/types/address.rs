use std::hash::{Hash, Hasher};
use std::ops::Deref;
use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::types::ManagedAddress;
use multiversx_sc_codec::{DecodeError, TopDecode, TopDecodeInput};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::scenario_model::AddressValue;
use multiversx_sdk::data::address::Address as SDKAddress;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;
use crate::error::AddressError;
use crate::error::DataError;
use crate::types::managed::ManagedConvertible;
use crate::types::native::NativeConvertible;

/// A struct representing a blockchain address.
/// This struct provides various utility methods for working with addresses,
/// including conversions from and to Bech32 string representations and byte arrays.
///
/// # Serialization
/// This struct is serializable with the `serde` crate.
///
/// # Cloning
/// Cloning is supported.
///
/// # Debugging
/// Debug printouts are supported.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use novax_data::Address;
/// let address = Address::from_bech32_string("erd1qqqqqqqqqqqqqpgq7ykazrzd905zvnlr88dpfw06677lxe9w0n4suz00uh").unwrap();
/// let bech32 = address.to_bech32_string().unwrap();
/// assert_eq!(bech32, "erd1qqqqqqqqqqqqqpgq7ykazrzd905zvnlr88dpfw06677lxe9w0n4suz00uh");
/// ```
#[derive(Serialize, Clone, Debug)]
pub struct Address(SDKAddress);


impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}

/// The `Address` struct provides an abstraction over a blockchain address,
/// with various utility methods for working with addresses.
impl Address {
    /// Creates an `Address` instance from a Bech32 string representation.
    ///
    /// # Parameters
    /// - `bech32`: The Bech32 string representation of the address.
    ///
    /// # Returns
    /// - An `Ok(Address)` instance if the conversion is successful.
    /// - An `Err(DataError)` if the Bech32 string is invalid.
    ///
    /// # Example
    /// ```
    /// # use novax_data::Address;
    /// let address = Address::from_bech32_string("erd1qqqqqqqqqqqqqpgq7ykazrzd905zvnlr88dpfw06677lxe9w0n4suz00uh").unwrap();
    /// ```
    pub fn from_bech32_string(bech32: &str) -> Result<Address, DataError> {
        let Ok(address) = SDKAddress::from_bech32_string(bech32) else { return Err(AddressError::InvalidBech32String.into()) };

        Ok(Address(address))
    }

    /// Creates an `Address` instance from a byte array.
    ///
    /// # Parameters
    /// - `bytes`: A byte array of length 32.
    ///
    /// # Returns
    /// - An `Address` instance.
    ///
    /// # Example
    /// ```
    /// # use novax_data::Address;
    /// let address = Address::from_bytes([0_u8; 32]);
    /// ```
    pub fn from_bytes(bytes: [u8; 32]) -> Address {
        Address(SDKAddress::from_bytes(bytes))
    }

    /// Converts the `Address` instance to a Bech32 string representation.
    ///
    /// # Returns
    /// - An `Ok(String)` containing the Bech32 string representation if successful.
    /// - An `Err(DataError)` if the conversion fails.
    ///
    /// # Example
    /// ```
    /// # use novax_data::Address;
    /// # let address = Address::from_bytes([0_u8; 32]);
    /// let bech32_string = address.to_bech32_string().unwrap();
    /// ```
    pub fn to_bech32_string(&self) -> Result<String, DataError> {
        let Ok(string) = self.0.to_bech32_string() else {
            return Err(AddressError::CannotConvertToBech32String.into())
        };

        Ok(string)
    }

    /// Converts the `Address` instance to a byte array.
    ///
    /// # Returns
    /// - A byte array of length 32.
    ///
    /// # Example
    /// ```
    /// # use novax_data::Address;
    /// # let address = Address::from_bytes([0_u8; 32]);
    /// let bytes = address.to_bytes();
    /// assert_eq!(bytes, [0_u8; 32]);
    /// ```
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}


impl Deref for Address {
    type Target = multiversx_sdk::data::address::Address;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for Address {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bytes().hash(state)
    }
}

impl<M: ManagedTypeApi> NativeConvertible for ManagedAddress<M> {
    type Native = Address;

    fn to_native(&self) -> Self::Native {
        Address(SDKAddress::from_bytes(self.to_byte_array()))
    }
}

impl NativeConvertible for Address {
    type Native = Self;

    fn to_native(&self) -> Self::Native {
        self.clone()
    }
}

impl ManagedConvertible<ManagedAddress<StaticApi>> for Address {
    fn to_managed(&self) -> ManagedAddress<StaticApi> {
        ManagedAddress::from_address(&multiversx_sc::types::Address::from(self.to_bytes()))
    }
}

impl TopDecode for Address {
    fn top_decode<I>(input: I) -> Result<Self, DecodeError> where I: TopDecodeInput {
        let bytes = ManagedAddress::<StaticApi>::top_decode(input)?.to_byte_array();
        Ok(Address(SDKAddress::from_bytes(bytes)))
    }
}

impl From<&Address> for AddressValue {
    fn from(value: &Address) -> Self {
        (&multiversx_sc::types::Address::from(value.0.to_bytes())).into()
    }
}

impl From<SDKAddress> for Address {
    fn from(value: SDKAddress) -> Self {
        Address::from_bytes(value.to_bytes())
    }
}

impl From<&SDKAddress> for Address {
    fn from(value: &SDKAddress) -> Self {
        Address::from_bytes(value.to_bytes())
    }
}

impl From<&multiversx_sc::types::Address> for Address {
    fn from(value: &multiversx_sc::types::Address) -> Self {
        Address::from_bytes(*value.as_array())
    }
}

impl From<multiversx_sc::types::Address> for Address {
    fn from(value: multiversx_sc::types::Address) -> Self {
        Address::from_bytes(*value.as_array())
    }
}

impl From<Address> for multiversx_sc::types::Address {
    fn from(value: Address) -> Self {
        multiversx_sc::types::Address::from(value.to_bytes())
    }
}

impl From<&str> for Address {
    fn from(value: &str) -> Self {
        if value.starts_with("erd1") {
            Address::from_bech32_string(value).unwrap()
        } else {
            (&AddressValue::from(value).value).into()
        }
    }
}

impl From<&String> for Address {
    fn from(value: &String) -> Self {
        From::<&str>::from(value)
    }
}

impl<'a> Deserialize<'a> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'a> {
        let string = String::deserialize(deserializer)?;

        let Ok(address) = Address::from_bech32_string(&string) else {
            return Err(D::Error::custom(format!("Cannot parse bech32 address : {string}")))
        };

        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use multiversx_sc::types::ManagedAddress;
    use multiversx_sc_scenario::api::StaticApi;
    use crate::Address;
    use crate::types::managed::ManagedConvertible;
    use crate::types::native::NativeConvertible;

    #[test]
    fn test_managed_address_to_native() {
        let expected = Address::from_bech32_string("erd1qqqqqqqqqqqqqpgq7ykazrzd905zvnlr88dpfw06677lxe9w0n4suz00uh").unwrap();
        let managed_address: ManagedAddress<StaticApi> = ManagedAddress::from(expected.to_bytes());
        let native = managed_address.to_native();

        assert_eq!(
            native.to_bytes(),
            expected.to_bytes()
        )
    }

    #[test]
    fn test_managed_address_to_managed() {
        let address = Address::from_bech32_string("erd1qqqqqqqqqqqqqpgq7ykazrzd905zvnlr88dpfw06677lxe9w0n4suz00uh").unwrap();
        let managed = address.to_managed();

        assert_eq!(
            address.to_bytes(),
            managed.to_byte_array()
        )
    }
}