use std::prelude::v1::*;

use dataview::Pod;
use std::convert::TryFrom;
use std::str;

/// Describes an offset file.
/// At compile time this crate will create a binary blob of all
/// TOML files contained in the memflow-win32/offsets/ folder
/// and merge the byte buffer directly into the build.
///
/// This byte buffer is then transmuted back into a slice of
/// Win32OffsetFile structs and parsed as a backup in case
/// no symbol store is available.
///
/// To get loaded properly this struct guarantees a certain alignment and no padding.
/// This is enforced due to a compile time assert as well as the Pod derive itself.
/// Especially in the case of cross-compilation where the target architecture
/// is different from the architecture memflow is built with this could give potential issues.
///
// # Safety
// This struct guarantees that it does not contain any padding.
#[repr(C, align(4))]
#[derive(Clone, Pod)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Win32OffsetFile {
    // Win32GUID
    #[cfg_attr(feature = "serde", serde(default))]
    pub pdb_file_name: BinaryString,
    #[cfg_attr(feature = "serde", serde(default))]
    pub pdb_guid: BinaryString,

    // Win32Version
    pub nt_major_version: u32,
    pub nt_minor_version: u32,
    pub nt_build_number: u32,

    // Architecture
    pub arch: Win32OffsetsArchitecture,

    pub offsets: Win32OffsetTable,
}

const _: [(); std::mem::size_of::<[Win32OffsetFile; 16]>()] =
    [(); 16 * std::mem::size_of::<Win32OffsetFile>()];

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Win32OffsetsArchitecture {
    X86 = 0,
    X64 = 1,
    AArch64 = 2,
}

impl std::fmt::Display for Win32OffsetsArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

unsafe impl Pod for Win32OffsetsArchitecture {}

// TODO: use const-generics here once they are fully stabilized
#[derive(Clone)]
pub struct BinaryString(pub [u8; 128]);

impl Default for BinaryString {
    fn default() -> Self {
        (&[][..]).into()
    }
}

impl<'a> From<&'a [u8]> for BinaryString {
    fn from(other: &'a [u8]) -> Self {
        let mut arr = [0; 128];

        arr[..other.len()].copy_from_slice(other);

        Self { 0: arr }
    }
}

impl<'a> TryFrom<&'a BinaryString> for &'a str {
    type Error = std::str::Utf8Error;
    fn try_from(other: &'a BinaryString) -> Result<Self, Self::Error> {
        Ok(str::from_utf8(&other.0)?
            .split_terminator('\0')
            .next()
            .unwrap())
    }
}

impl<'a> From<&'a str> for BinaryString {
    fn from(other: &'a str) -> Self {
        let mut arr = [0; 128];

        arr[..other.len()].copy_from_slice(other.as_bytes());

        Self { 0: arr }
    }
}

impl From<String> for BinaryString {
    fn from(other: String) -> Self {
        Self::from(other.as_str())
    }
}

unsafe impl Pod for BinaryString {}

#[cfg(feature = "serde")]
impl ::serde::Serialize for BinaryString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(
            <&str>::try_from(self)
                .map_err(|_| ::serde::ser::Error::custom("invalid UTF-8 characters"))?,
        )
    }
}

#[cfg(feature = "serde")]
impl<'de> ::serde::de::Deserialize<'de> for BinaryString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::de::Deserializer<'de>,
    {
        struct BinaryStringVisitor;

        impl<'de> ::serde::de::Visitor<'de> for BinaryStringVisitor {
            type Value = [u8; 128];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string containing json data")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                // unfortunately we lose some typed information
                // from errors deserializing the json string
                let mut result = [0u8; 128];

                result[..v.len()].copy_from_slice(v.as_bytes());

                Ok(result)
            }
        }

        // use our visitor to deserialize an `ActualValue`
        let inner: [u8; 128] = deserializer.deserialize_any(BinaryStringVisitor)?;
        Ok(Self { 0: inner })
    }
}

#[repr(C, align(4))]
#[derive(Debug, Clone, Pod)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct Win32OffsetTable {
    pub list_blink: u32,
    pub eproc_link: u32,

    /// Since version 3.10
    pub kproc_dtb: u32,
    /// Since version 3.10
    pub eproc_pid: u32,
    /// Since version 3.10
    pub eproc_name: u32,
    /// Since version 5.10
    pub eproc_peb: u32,
    /// Since version 3.10
    pub eproc_section_base: u32,
    /// Since version 3.10
    pub eproc_exit_status: u32,
    /// Since version 5.10
    pub eproc_thread_list: u32,
    /// Since version 5.0
    pub eproc_wow64: u32,

    /// Since version 6.2
    pub kthread_teb: u32,
    /// Since version 6.2
    pub ethread_list_entry: u32,
    /// Since version x.x
    pub teb_peb: u32,
    /// Since version x.x
    pub teb_peb_x86: u32,
}
