multiversx_sc::imports!();
multiversx_sc::derive_imports!();

/// Provides a bridge from a complex smart contract type, managed by the
/// MultiversX virtual machine, to a common Rust type.
///
/// This trait facilitates the conversion from managed types to native Rust types.
/// The conversion is one-way and is designed to map multiple managed types to
/// a single Rust type, establishing a many-to-one relationship.
///
/// Implementing this trait requires specifying the associated `Native` type
/// and providing an implementation for the `to_native` method, which will carry
/// out the actual conversion.
///
/// # Type Parameters
/// - `Native`: The native Rust type to which the managed type will be converted.
///
/// # Methods
/// - `to_native`: Performs the conversion from the managed type to the specified
/// native Rust type.
pub trait NativeConvertible {
    /// The native Rust type to which the managed type will be converted.
    type Native;

    /// Converts the managed type to the specified native Rust type.
    fn to_native(&self) -> Self::Native;
}
