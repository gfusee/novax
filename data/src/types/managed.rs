multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use multiversx_sc_codec::TopEncodeMulti;

/// Provides a bridge from common Rust types to complex smart contract types
/// managed by the MultiversX virtual machine.
///
/// This trait facilitates the conversion from native Rust types to managed types.
/// Unlike `NativeConvertible`, this bridge establishes a many-to-many relationship,
/// allowing multiple Rust types to be converted to multiple managed types.
///
/// Implementing this trait requires specifying the managed type parameter `M` and
/// providing an implementation for the `to_managed` method, which will carry out
/// the actual conversion.
///
/// # Type Parameters
/// - `M`: The managed type to which the native Rust type will be converted,
///        constrained by the `TopEncodeMulti` trait.
///
/// # Methods
/// - `to_managed`: Performs the conversion from the native Rust type to the specified
///                 managed type.
///
/// # Examples
///
/// - `String` can be converted to `ManagedBuffer`, `TokenIdentifier`, or `ManagedAddress`.
/// - `Vec` can be converted to `ManagedVec`.
///
/// ```rust
/// # use novax_data::ManagedConvertible;
/// # use multiversx_sc::types::ManagedBuffer;
/// # use multiversx_sc_scenario::api::StaticApi;
///
/// struct YourStruct(String);
///
/// impl ManagedConvertible<ManagedBuffer<StaticApi>> for YourStruct {
///     fn to_managed(&self) -> ManagedBuffer<StaticApi> {
///         ManagedBuffer::from(&*self.0)
///     }
/// }
/// ```
pub trait ManagedConvertible<M: TopEncodeMulti> {
    /// Converts the native Rust type to the specified managed type.
    fn to_managed(&self) -> M;
}


macro_rules! managed_convertible_impl_self {
    ($($type_name:ident )+) => {
        $(
            impl ManagedConvertible<$type_name> for $type_name {
                fn to_managed(&self) -> $type_name {
                    self.clone()
                }
            }
        )+
    }
}

pub(crate) use managed_convertible_impl_self;