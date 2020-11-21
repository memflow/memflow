use memflow::types::byte_swap::ByteSwap;
use memflow_derive::*;

#[derive(ByteSwap)]
struct ByteSwapDerive {
    pub val: u32,
}

#[derive(ByteSwap)]
struct ByteSwapDeriveGeneric<T: ByteSwap> {
    pub val: T,
}

#[derive(ByteSwap)]
struct ByteSwapDeriveWhere<T>
where
    T: ByteSwap,
{
    pub val: T,
}

#[derive(ByteSwap)]
struct ByteSwapDeriveSlice {
    pub slice: [u8; 32],
}

#[derive(ByteSwap)]
struct ByteSwapDeriveStructSlice {
    pub slice: [ByteSwapDeriveSlice; 128],
}

#[derive(ByteSwap)]
struct ByteSwapDeriveStructGenericSlice<T: ByteSwap> {
    pub slice: [ByteSwapDeriveGeneric<T>; 128],
}

#[test]
pub fn compiles() {}
