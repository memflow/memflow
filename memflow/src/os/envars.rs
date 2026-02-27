use crate::prelude::v1::*;

#[repr(C)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct EnvVarInfo {
    /// Variable name (e.g. "PATH")
    pub name: ReprCString,
    /// Variable value (full string)
    pub value: ReprCString,
    /// Address of the variable in target memory, if known
    pub address: Address,
    /// Architecture of the envar
    pub arch: ArchitectureIdent,
}

pub type EnvVarCallback<'a> = OpaqueCallback<'a, EnvVarInfo>;
