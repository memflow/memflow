use memflow::types::{Address, PhysicalAddress};

/// Helper to convert `Address` to a `PhysicalAddress`
///
/// This will create a `PhysicalAddress` with `UNKNOWN` PageType.
#[no_mangle]
pub extern "C" fn addr_to_paddr(address: Address) -> PhysicalAddress {
    address.into()
}
